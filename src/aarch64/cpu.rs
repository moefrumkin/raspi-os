use super::registers::MPIDR_EL1;

pub fn core_id() -> u64 {
    MPIDR_EL1.read() & 0b11
}

pub fn wait_for_cycles(cycles: u64) {
    for i in 0..cycles {
        unsafe {
            asm!("nop");
        }
    }
}