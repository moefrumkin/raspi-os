use core::ptr;

use crate::{platform::{emmc::EMMCRegisters, gpio::GPIORegisters, interrupt::InterruptRegisters, mailbox::MailboxRegisters, mini_uart::MiniUARTRegisters}, sync::SpinMutex};

use super::{
    timer::TimerRegisters
};


unsafe extern "C" {
    unsafe static MMIO_START: usize;
    unsafe static mut TIMER_REGISTERS: TimerRegisters;
    unsafe static mut INTERRUPT_REGISTERS: InterruptRegisters;
    unsafe static mut GPIO_REGISTERS: GPIORegisters;
    unsafe static mut MAILBOX_REGISTERS: MailboxRegisters;
    unsafe static mut EMMC_REGISTERS: EMMCRegisters;
    unsafe static mut MINI_UART_REGISTERS: MiniUARTRegisters;
}

//const MMIO_START: usize = 0x3F00_0000;

const LENGTH: usize = 0x00FFFFFF;

const TIMER_REGISTER_OFFSET: usize = 0x3000;
const MAILBOX_REIGSTER_OFFSET: usize = 0xB880;
const GPIO_REGISTER_OFFSET: usize = 0x20_0000;
const EMMC_REGISTER_OFFSET: usize = 0x30_0000;
const MINI_UART_REGISTER_OFFSET: usize = 0x21_5000;

const unsafe fn to_mut_mmio_registers<T>(offset: usize) -> &'static mut T
{
    &mut *(((MMIO_START) + offset) as *mut T)
}

pub const fn get_timer_registers() -> &'static mut TimerRegisters {
    unsafe {
        &mut TIMER_REGISTERS
    }
}

pub const fn get_emmc_registers() -> &'static mut EMMCRegisters {
    unsafe {
        &mut EMMC_REGISTERS
    }
}

pub const fn get_gpio_registers() -> &'static mut GPIORegisters {
    unsafe {
        &mut GPIO_REGISTERS
    }
}

pub const fn get_miniuart_registers() -> &'static mut MiniUARTRegisters {
    unsafe {
        &mut MINI_UART_REGISTERS
    }
}

pub const fn get_mailbox_registers() -> &'static mut MailboxRegisters {
    unsafe {
        &mut MAILBOX_REGISTERS
    }
}

pub const fn get_interrupt_registers() -> &'static mut InterruptRegisters {
    unsafe {
        &mut INTERRUPT_REGISTERS
    }
}

/*pub struct MMIOController {
    start: usize,
    length: usize,
}

impl MMIOController {
    pub const fn new() -> Self {
        MMIOController {
            start: START,
            length: LENGTH,
        }
    }
}

impl Default for MMIOController {
    fn default() -> Self {
        Self::new()
    }
}*/