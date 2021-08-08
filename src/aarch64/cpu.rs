use super::registers::MPIDR_EL1;

<<<<<<< HEAD
pub fn core_id() -> u64 {
    MPIDR_EL1.read() & 0b11
=======
/// Returns the id of the cpu core as reported by the arm MPIDR_EL1 system register
pub fn core_id() -> u64 {
    MPIDR_EL1.read() & 0xff
}

/// Initializes a region of memory to the initial value. Start is included and end is excluded.
pub fn init_region(start: *mut usize, end: *mut usize, init_val: usize) {
    while start < end {
        unsafe {
            core::ptr::write_volatile(start, init_val);
            *start = *start.offset(1);
        }
    }
>>>>>>> blink
}

pub fn wait_for_cycles(cycles: u64) {
    for i in 0..cycles {
        unsafe {
            asm!("nop");
        }
    }
}