use crate::{platform::{emmc::EMMCRegisters, gpio::GPIORegisters, mini_uart::MiniUARTRegisters}, sync::SpinMutex};

use super::{
    timer::TimerRegisters
};


const START: usize = 0x3F000000;
const LENGTH: usize = 0x00FFFFFF;

const TIMER_REGISTER_OFFSET: usize = 0x3000;
const GPIO_REGISTER_OFFSET: usize = 0x20_0000;
const EMMC_REGISTER_OFFSET: usize = 0x30_0000;
const MINI_UART_REGISTER_OFFSET: usize = 0x21_5000;

const unsafe fn to_mut_mmio_registers<T>(offset: usize) -> &'static mut T
{
    &mut *((START + offset) as *mut T)
}

pub const fn get_timer_registers() -> &'static mut TimerRegisters {
    unsafe {
        to_mut_mmio_registers(TIMER_REGISTER_OFFSET)
    }
}

pub const fn get_emmc_registers() -> &'static mut EMMCRegisters {
    unsafe {
        to_mut_mmio_registers(EMMC_REGISTER_OFFSET)
    }
}

pub const fn get_gpio_registers() -> &'static mut GPIORegisters {
    unsafe {
        to_mut_mmio_registers(GPIO_REGISTER_OFFSET)
    }
}

pub const fn get_miniuart_registers() -> &'static mut MiniUARTRegisters {
    unsafe {
        to_mut_mmio_registers(MINI_UART_REGISTER_OFFSET)
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