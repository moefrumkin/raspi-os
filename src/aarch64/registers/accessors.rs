#[macro_export]
macro_rules! sysreg_read {
    ($size:ty, $name:tt) => {
        #[inline]
        pub fn read(&self) -> $size {
            let value;
            unsafe {
                asm!(concat!("mrs {}, ", $name), out(reg) value);
                //llvm_asm!(concat!("mrs $0, ", $name) :"=r"(value) ::: "volatile");
            }
            value
        }
    };
}

#[macro_export]
macro_rules! sysreg_write {
    ($size:ty, $name:tt) => {
        #[inline]
        pub fn write(&self, value: $size) {
            unsafe {
                //llvm_asm!(concast!("msr ", $name, ", $0") :: "r"(value) ::: "volatile")
            }
        }
    };
}
