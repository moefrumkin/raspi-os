#[cfg(not(test))]
pub use board::*;

#[cfg(test)]
pub use test::*;

#[cfg(not(test))]
mod board {
    pub const GPIO_START: usize = 0x3F000000;

    pub fn write_at_offset(data: u32, offset: usize) {
        unsafe {
            *((GPIO_START + offset) as *mut u32) = data;
        }
    }

    pub fn read_at_offset(offset: usize) -> u32 {
        unsafe {
            *((GPIO_START + offset) as *const u32)
        }
    }
}

#[cfg(test)]
mod test {
    static mut storage: &'static mut [u32] = &mut [0; 44];
    
    pub fn write_at_offset(data: u32, offset: usize) {
        unsafe {
            storage[offset] = data;
            println!("writing {:#034b} to {}", data, offset);
            println!("{:?}", storage);
        }
    }

    pub fn read_at_offset(offset: usize) -> u32 {
        unsafe {
            storage[offset]
        }
    }
}