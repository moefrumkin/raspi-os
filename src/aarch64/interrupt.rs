use core::arch::asm;

pub fn enable_irq() {
    unsafe {
        asm!(
            "msr daifclr, 0b10"
        );
    }
}

pub fn disable_irq() {
    unsafe {
        asm!(
            "msr daifset, 0b10"
        )
    }
}