#[allow(non_snake_case)]
mod MPIDR_EL1 {
    pub struct Register {}

    impl Register {
        crate::sysreg_read!(u64, "MPIDR_EL1");
    }
}

pub static MPIDR_EL1: MPIDR_EL1::Register = MPIDR_EL1::Register {};
