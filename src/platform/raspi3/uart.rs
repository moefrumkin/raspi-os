use super::{
    gpio::{GPIOController, Mode, Pin},
    mmio::MMIOController,
};

const UART_BASE_OFFSET: u32 = 0x215000;

const AUX_ENABLE: u32 = UART_BASE_OFFSET + 0x4;

const AUX_MU_IO: u32 = UART_BASE_OFFSET + 0x40;
const AUX_MU_IER: u32 = UART_BASE_OFFSET + 0x44;
const AUX_MU_IIR: u32 = UART_BASE_OFFSET + 0x48;
const AUX_MU_LCR: u32 = UART_BASE_OFFSET + 0x4c;
const AUX_MU_MCR: u32 = UART_BASE_OFFSET + 0x50;
const AUX_MU_LSR: u32 = UART_BASE_OFFSET + 0x54;
const AUX_MU_CNTL: u32 = UART_BASE_OFFSET + 0x60;
const AUX_MU_BAUD: u32 = UART_BASE_OFFSET + 0x68;

#[allow(dead_code)]
pub struct UARTController<'a> {
    gpio: &'a GPIOController<'a>,
    mmio: &'a MMIOController,
}

impl<'a> UARTController<'a> {
    pub fn init(gpio: &'a GPIOController, mmio: &'a MMIOController) -> Self {
        //Enable UART
        mmio.write_at_offset(
            mmio.read_at_offset(AUX_ENABLE as usize) | 1,
            AUX_ENABLE as usize,
        );

        //Disable Tx and Rx
        mmio.write_at_offset(0, AUX_MU_CNTL as usize);

        //Set data format to 8 bit
        mmio.write_at_offset(0b11, AUX_MU_LCR as usize);

        //Set rts line high
        mmio.write_at_offset(0, AUX_MU_MCR as usize);

        //Disable interrupts
        mmio.write_at_offset(0, AUX_MU_IER as usize);

        //Clear fifo bits
        mmio.write_at_offset(0b11000110, AUX_MU_IIR as usize);

        //Set baud rate to 115,200
        mmio.write_at_offset(270, AUX_MU_BAUD as usize);

        let tx = Pin::new(14).unwrap();
        let rx = Pin::new(15).unwrap();

        gpio.set_mode(tx, Mode::AF5);
        gpio.set_mode(rx, Mode::AF5);

        //enable Tx and Rx
        mmio.write_at_offset(3, AUX_MU_CNTL as usize);

        UARTController { gpio, mmio }
    }

    pub fn putc(&self, c: char) {
        while self.mmio.read_at_offset(AUX_MU_LSR as usize) & 0b100000 == 0 {
            unsafe {
                asm!("nop");
            }
        }
        self.mmio.write_at_offset(c as u32, AUX_MU_IO as usize);
    }

    pub fn write_hex(&self, n: usize) {
        self.putc('0');
        self.putc('x');

        for c in (0..=60).step_by(4) {
            let mut n = (n >> (60 - c)) & 0b1111;

            n += if n > 9 { 0x37 } else { 0x30 };
            self.putc(n as u8 as char);
        }
    }

    pub fn write(&self, s: &str) {
        for c in s.chars() {
            self.putc(c);
        }
    }

    pub fn writeln(&self, s: &str) {
        for c in s.chars() {
            self.putc(c);
        }
        self.putc('\n');
        self.putc('\r');
    }

    #[allow(dead_code)]
    pub fn read(&self) -> Result<char, ()> {
        while self.mmio.read_at_offset(AUX_MU_LSR as usize) & 0b1 == 0 {
            unsafe {
                asm!("nop");
            }
        }

        core::char::from_u32(self.mmio.read_at_offset(AUX_MU_IO as usize)).ok_or(())
    }
}
