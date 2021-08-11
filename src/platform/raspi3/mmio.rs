#[cfg(not(test))]
pub use board::*;

#[cfg(test)]
pub use test::*;

const START: usize = 0x3F000000;
const LENGTH: usize = 0x00FFFFFF;

pub struct MMIOController {
    start: usize,
    length: usize,
}

impl Default for MMIOController {

    fn default() -> Self {
        MMIOController {
            start: START,
            length: LENGTH
        }
    }
}

#[cfg(not(test))]
mod board {
    use super::MMIOController;

    impl MMIOController {
        //TODO: switch order of operands
        pub fn write_at_offset(&self, data: u32, offset: usize) {
            unsafe {
                core::ptr::write_volatile((self.start + offset) as *mut u32, data);
            }
        }

        pub fn read_at_offset(&self, offset: usize) -> u32 {
            unsafe {
                core::ptr::read_volatile((self.start + offset) as *const u32)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::MMIOController;

    static mut storage: &'static mut [u32] = &mut [0;super::LENGTH];
    
    impl MMIOController {
        pub fn write_at_offset(&self, data: u32, offset: usize) {
            unsafe {
                storage[offset] = data;
            }
        }

        pub fn read_at_offset(&self, offset: usize) -> u32 {
            unsafe {
                storage[offset]
            }
        }
    }
}