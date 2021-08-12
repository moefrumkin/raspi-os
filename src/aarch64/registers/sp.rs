mod sp {
    pub struct Register {}

    #[allow(dead_code)]
    impl Register {
        pub fn write(&self, value: usize) {
            unsafe {
                asm!("mov sp, {}", in(reg) value);
            }
        }
    }
}

#[allow(dead_code)]
pub static SP: sp::Register = sp::Register {};
