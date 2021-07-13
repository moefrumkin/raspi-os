mod sp {
    pub struct Register {}

    impl Register {
        crate::sysreg_write!(u64, "sp");
    }
}

pub static SP: sp::Register = sp::Register {};
