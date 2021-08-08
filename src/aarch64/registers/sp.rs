mod sp {
    pub struct Register {}

    impl Register {
        pub fn write(&self, value: usize) {
            unsafe {
                asm!("mov sp, {}", in(reg) value);
            }
        }
    }
}

pub static SP: sp::Register = sp::Register {};
