use crate::aarch64::cpu;
use super::{gpio::{Pin, Mode, Pull}, mmio};

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


pub fn init() {
    //Enable UART
    mmio::write_at_offset(
        mmio::read_at_offset(AUX_ENABLE as usize) | 1,
        AUX_ENABLE as usize
    );

    //Disable Tx and Rx
    mmio::write_at_offset(0, AUX_MU_CNTL as usize);

    //Set data format to 8 bit
    mmio::write_at_offset(0b11, AUX_MU_LCR as usize);

    //Set rts line high
    mmio::write_at_offset(0, AUX_MU_MCR as usize);

    //Disable interrupts
    mmio::write_at_offset(0, AUX_MU_IER as usize);

    //Clear fifo bits
    mmio::write_at_offset(0b11000110, AUX_MU_IIR as usize);

    //Set baud rate to 115,200
    mmio::write_at_offset(270, AUX_MU_BAUD as usize);

    let tx = Pin::new(14).unwrap();
    let rx = Pin::new(15).unwrap();

    tx.set_mode(Mode::AF5);
    rx.set_mode(Mode::AF5);
    
    //enable Tx and Rx
    mmio::write_at_offset(3, AUX_MU_CNTL as usize);
}

pub fn send_char(c: char) {
    while mmio::read_at_offset(AUX_MU_LSR as usize) & 0b100000 == 0 {
        unsafe {
            asm!("nop");
        }
    }
    mmio::write_at_offset(c as u32, AUX_MU_IO as usize);
}

pub fn send_str(s: &str) {
    for c in s.chars() {
        send_char(c);
    }
}

pub fn read() -> Result<char, ()> {
    while mmio::read_at_offset(AUX_MU_LSR as usize) & 0b1 == 0 {
        unsafe {
            asm!("nop");
        }
    }

    core::char::from_u32(mmio::read_at_offset(AUX_MU_IO as usize)).ok_or(())
}