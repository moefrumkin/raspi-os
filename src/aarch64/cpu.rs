use super::registers::MPIDR_EL1;

/// Returns the id of the cpu core as reported by the arm MPIDR_EL1 system register
#[allow(dead_code)]
pub fn core_id() -> u64 {
    MPIDR_EL1.read() & 0xff
}

#[allow(dead_code)]
pub fn wait_for_cycles(cycles: u64) {
    for _ in 0..cycles {
        unsafe {
            asm!("nop");
        }
    }
}