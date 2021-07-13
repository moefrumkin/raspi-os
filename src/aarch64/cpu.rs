use super::registers::MPIDR_EL1;

pub fn wait_for_cycles(cycles: u64) {
    for i in 0..cycles {
        unsafe {
            asm!("nop");
        }
    }
}
