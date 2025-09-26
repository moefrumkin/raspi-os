use crate::bitfield;
use crate::volatile::Volatile;

pub enum InterruptType {
    TimerInterrupt,
}

#[repr(C)]
pub struct InterruptRegisters {
    irq_basic_pending: Volatile<IRQSource>,
    irq_pending_1: Volatile<InterruptBlock1>,
    irq_pending_2: Volatile<u32>,
    fiq_control: Volatile<u32>,
    enable_irq_1: Volatile<InterruptBlock1>,
    enable_irq_2: Volatile<u32>,
    enable_basic_irqs: Volatile<u32>,
    disable_irq_1: Volatile<InterruptBlock1>,
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

    pub fn get_interrupt_type(&self) -> Option<InterruptType> {


        None
    }
}

bitfield! {
    IRQSource(u32) {
        arm_timer_irq: 0-0,
        arm_mailbox_irq: 1-1,
        arm_doorbell_0_irq: 2-2,
        arm_doorbell_1_irq: 3-3,
        block_1_irq: 8-8,
        block_2_irq: 9-9
    }
}

// TODO: Could the interrupt blocks be merged into 1 64 bit block?
bitfield! {
    InterruptBlock1(u32) {
        system_timer_match_3: 3-3,
        usb_controller: 9-9,
        auxiliary_device_interrupt: 29-29
    }
}

bitfield! {
    BasicInterruptBlock(u32) {
        arm_timer: 0-0
    }
}