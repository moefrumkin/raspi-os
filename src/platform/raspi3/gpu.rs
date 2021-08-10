//! This module provides an interface to the raspberry pi's built in gpu

/*use super::mmio;

const MBOX_BASE_OFFSET: usize = 0x0000b880;

pub fn init() -> Result<FBInfo, ()>{
    let fb_info = FBInfo {
        width: 1920,
        height: 1080,
        v_width: 1920,
        v_height: 1080,
        pitch: 0,
        bit_depth: 16,
        x_offset: 0,
        y_offset: 0,
        ptr: 0,
        size: 0,
    };

    let gpu_channel = Channel::new(0).unwrap();

    unsafe {
        let fbi_ptr = &fb_info as *const FBInfo;
        gpu_channel.write(fbi_ptr as u32 + 0x40000000)
    }

    if gpu_channel.read() == 0 {Ok(fb_info)} else {Err(())}
}

fn wait_util_ready() {
    loop {
        //wait until mailbox is ready
        let status = MailboxRegister::Status.read();
        if status & 0x80000000 == 0 {
            break;
        }
    }
}

pub struct Channel{
    number: u32,
}

impl Channel {
    const CHANNELS: u32 = 7;

    pub fn new(number: u32) -> Result<Self, ()> {
        if number < Channel::CHANNELS {Ok(Channel { number: number} )} else { Err(()) }
    }

    pub fn read(&self) -> u32 {
        loop {
            wait_util_ready();

            let value = mmio::read_at_offset(MBOX_BASE_OFFSET + MailboxRegister::Read as usize);

            if value & 0b1111 == self.number {
                return value;
            }
        }
    }

    pub fn write(&self, value: u32) {
        wait_util_ready();

        mmio::write_at_offset(value + self.number, MBOX_BASE_OFFSET + MailboxRegister::Write as usize)
    }
}

pub enum MailboxRegister {
    Read = 0,
    Poll= 16,
    Sender = 20,
    Status = 24,
    Configuration = 28,
    Write = 32
}

impl MailboxRegister {
    pub fn read(self) -> u32 {
        mmio::read_at_offset(MBOX_BASE_OFFSET + self as usize)
    }
}

#[repr(align(4))]
pub struct FBInfo {
    width: u32,
    height: u32,
    v_width: u32,
    v_height: u32,
    pitch: u32,
    bit_depth: u32,
    x_offset: u32,
    y_offset: u32,
    ptr: u32,
    size: u32,
}

impl FBInfo {
    pub fn draw(&self) {
        let color: u16 = 0xff;
        loop {
            let mut current_pxl = self.ptr;
            for y in 0..self.v_height {
                for x in 0..self.v_width {
                    unsafe {
                        core::ptr::write_volatile((current_pxl as *mut u16), color);
                    }
                    current_pxl += 2
                }
            }
        }
    }
}*/