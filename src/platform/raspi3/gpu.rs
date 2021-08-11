use super::mmio::MMIOController;
use super::uart::UARTController;

const MBOX_BASE_OFFSET: u32 = 0xB880;
const MBOX_READ: u32 = MBOX_BASE_OFFSET + 0x0;
const MBOX_POLL: u32 = MBOX_BASE_OFFSET + 0x10;
const MBOX_SENDER: u32 = MBOX_BASE_OFFSET + 0x14;
const MBOX_STATUS: u32 = MBOX_BASE_OFFSET + 0x18;
const MBOX_CONFIG: u32 = MBOX_BASE_OFFSET + 0x1c;
const MBOX_WRITE: u32 = MBOX_BASE_OFFSET + 0x20;
const MBOX_RESPONSE: u32 = 0x80000000;
const MBOX_FULL: u32 = 0x80000000;
const MBOX_EMPTY: u32 = 0x40000000;

const MBOX_REQUEST: u32 = 0;

const MBOX_CH_POWER: u32 = 0;
const MBOX_CH_FB: u32 = 1;
const MBOX_CH_VUART: u32 = 2;
const MBOX_CH_VCHIQ: u32 = 3;
const MBOX_CH_LEDS: u32 = 4;
const MBOX_CH_BTNS: u32 = 5;
const MBOX_CH_TOUCH: u32 = 6;
const MBOX_CH_COUNT: u32 = 7;
const MBOX_CH_PROP: u32 = 8;

const MBOX_TAG_SETPOWER: u32 = 0x28001;
const MBOX_TAG_SETCLKRATE: u32 = 0x38002;

const MBOX_TAG_SETPHYWH: u32 = 0x48003;
const MBOX_TAG_SETVIRTWH: u32 = 0x48004;
const MBOX_TAG_SETVIRTOFF: u32 = 0x48009;
const MBOX_TAG_SETDEPTH: u32 = 0x48005;
const MBOX_TAG_SETPXLORDR: u32 = 0x48006;
const MBOX_TAG_GETFB: u32 = 0x40001;
const MBOX_TAG_GETPITCH: u32 = 0x40008;

const MBOX_TAG_LAST: u32 = 0;

pub static mut MBOX: MailboxData = MailboxData {
    data: [0u32; 36]
};

#[repr(C)]
#[repr(align(16))]
pub struct MailboxData {
    pub data: [u32; 36]
}

impl MailboxData {
    pub fn get(&self, i: isize) -> u32 {
        unsafe {
            self.ptr().offset(i).read_volatile()
        }
    }

    pub fn set(&self, i: isize, value: u32) {
        unsafe {
            self.ptr().offset(i).write_volatile(value);
        }
    }

    pub fn ptr(&self) -> *mut u32 {
        unsafe {
            self.data.as_ptr() as *mut u32
        }
    }
}

pub unsafe fn fn_init(mmio: &MMIOController, uart: &UARTController) {
    MBOX.set(0, 35 * 4);
    MBOX.set(1, MBOX_REQUEST);

    MBOX.set(2, MBOX_TAG_SETPHYWH);
    MBOX.set(3, 8);
    MBOX.set(4, 8);
    MBOX.set(5, 1920);
    MBOX.set(6, 1080);

    MBOX.set(7, MBOX_TAG_SETVIRTWH);
    MBOX.set(8, 8);
    MBOX.set(9, 8);
    MBOX.set(10, 1920);
    MBOX.set(11, 1080);

    MBOX.set(12, MBOX_TAG_SETVIRTOFF);
    MBOX.set(13, 8);
    MBOX.set(14, 8);
    MBOX.set(15, 0);
    MBOX.set(16, 0);

    MBOX.set(17, MBOX_TAG_SETDEPTH);
    MBOX.set(18, 4);
    MBOX.set(19, 4);
    MBOX.set(20, 32);

    MBOX.set(21, MBOX_TAG_SETPXLORDR);
    MBOX.set(22, 4);
    MBOX.set(23, 4);
    MBOX.set(24, 1);

    MBOX.set(25, MBOX_TAG_GETFB);
    MBOX.set(26, 8);
    MBOX.set(27, 8);
    MBOX.set(28, 4096);
    MBOX.set(29, 0);
    
    MBOX.set(30, MBOX_TAG_GETPITCH);
    MBOX.set(31, 4);
    MBOX.set(32, 4);
    MBOX.set(33, 0);

    MBOX.set(34, MBOX_TAG_LAST);

    unsafe {
        mbox_call(MBOX_CH_PROP as u8, &mmio, &uart);
        uart.write_hex(MBOX.get(28) as usize);
    }
}

pub unsafe fn mbox_call(ch: u8, mmio: &MMIOController, uart: &UARTController) -> u32 {
    uart.writeln("Performing mbox_call");

    let mbox_ptr = MBOX.ptr() as usize;

    uart.write("Using Mailbox Pointer: ");
    uart.write_hex(mbox_ptr);
    uart.write(", ");

    let r: u32 = ((mbox_ptr & !0b1111) | (ch & 0b1111) as usize) as u32;

    uart.write_hex(r as usize);
    uart.writeln("");

    while mmio.read_at_offset(MBOX_STATUS as usize) & MBOX_FULL != 0 {
        unsafe { asm!("nop"); }
    }

    uart.writeln("Sending Pointer");

    mmio.write_at_offset(r, MBOX_WRITE as usize);

    loop {
        while (mmio.read_at_offset(MBOX_STATUS as usize) & MBOX_EMPTY) != 0 {
            unsafe { asm!("nop"); }
        }

        uart.write("MBOX value is: ");
        let read_status = mmio.read_at_offset(MBOX_READ as usize);
        uart.write_hex(read_status as usize);
        uart.writeln("");

        if r == read_status {
            uart.writeln("Recieved Response");
            return if MBOX.get(1) == MBOX_RESPONSE {1} else {0};
        }
    }
}

pub unsafe fn draw_pixel(x: u32, y: u32, color: u32) {
    let fb = MBOX.get(28) & 0x3FFFFFFF;
    let pitch = MBOX.get(33);
    core::ptr::write_volatile((fb + (4 * x) + (pitch * y)) as *mut u32, color);
}

pub unsafe fn draw_stuff() {

    for x in 0..1920 {
        for y in 0..1080 {
            let red = x & 0xff;
            let blue = y & 0xff;
            let green = 0x66;
            let color = (red << 16) + (blue << 8) + green;
            draw_pixel(x, y, color);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_mbox() {
        let MBOX = super::MailboxData {
            data: [0u32; 36]
        };
        println!("Hello {:?}: {:?}", &MBOX.ptr(), &MBOX.data);
        MBOX.set(0, 9);
        MBOX.set(31, 298);
        
        assert_eq!(MBOX.get(0), 9);
        assert_eq!(MBOX.get(31), 298);

        println!("Hello {:?}: {:?}", &MBOX.ptr(), &MBOX.data);
    }
}