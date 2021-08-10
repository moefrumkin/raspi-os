#[cfg(not(test))]
pub use board::*;

#[cfg(test)]
pub use test::*;

#[cfg(not(test))]
mod board {
    pub const MMIO_START: usize = 0x3F000000;

    //TODO: switch order of operands
    pub fn write_at_offset(data: u32, offset: usize) {
        unsafe {
            core::ptr::write_volatile((MMIO_START + offset) as *mut u32, data);
        }
    }

    pub fn read_at_offset(offset: usize) -> u32 {
        unsafe {
            core::ptr::read_volatile((MMIO_START + offset) as *const u32)
        }
    }
}

#[cfg(test)]
mod test {
    static mut storage: &'static mut [u32] = &mut [0; 0x00200050];
    
    pub fn write_at_offset(data: u32, offset: usize) {
        unsafe {
            storage[offset] = data;
        }
    }

    pub fn read_at_offset(offset: usize) -> u32 {
        unsafe {
            storage[offset]
        }
    }
}