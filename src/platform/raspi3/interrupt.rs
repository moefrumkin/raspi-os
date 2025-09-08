use crate::bitfield;
use crate::volatile::Volatile;

pub enum Interrupt {

}

#[repr(C)]
pub struct InterruptRegisters {
    irq_basic_pending: Volatile<u32>,
    irq_pending_1: Volatile<u32>,
    irq_pending_2: Volatile<u32>,
    fiq_control: Volatile<u32>,
    enable_irq_1: Volatile<InterruptBlock1>,
    enable_irq_2: Volatile<u32>,
    enable_basic_irqs: Volatile<u32>,
    disable_irq_1: Volatile<u32>,
    disable_irq_2: Volatile<u32>,
    disable_basic_irqs: Volatile<u32>
}

impl InterruptRegisters {
    const INTERRUPT_REGISTERS_BASE: usize = 0x3F00_B200;

    fn get() -> &'static mut Self {
        unsafe {
            &mut *{Self::INTERRUPT_REGISTERS_BASE as *mut Self}
        }
    }
}

pub struct InterruptController<'a> {
    registers: &'a mut InterruptRegisters
}

impl<'a> InterruptController<'a> {
    pub fn new() -> Self {
        Self {
            registers: InterruptRegisters::get()
        }
    }

    pub fn enable_timer_interrupt_3(&mut self) {
        self.registers.enable_irq_1.map(|interrupt_block|
            interrupt_block.set_system_timer_match_3(1)
        );
    }

    pub fn enable_mini_uart_interrupt(&mut self) {
        self.registers.enable_irq_1.map(|interrupt_block|
            interrupt_block.set_auxiliary_device_interrupt(1)
        );
    }
}

bitfield! {
    InterruptBlock1(u32) {
        system_timer_match_3: 3-3,
        auxiliary_device_interrupt: 29-29
    }
}