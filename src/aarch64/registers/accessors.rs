#[macro_export]
macro_rules! sysreg_read {
    ($size:ty, $name:tt) => {
        #[allow(dead_code)]
        pub fn read(&self) -> $size {
            let value;
            unsafe {
                asm!(concat!("mrs {}, ", $name), out(reg) value);
            }
            value
        }
    };
}

#[macro_export]
macro_rules! sysreg_write {
    ($size:ty, $name:tt) => {
        #[allow(dead_code)]
        pub fn write(&self, value: $size) {
            unsafe {
                asm!(concat!("msr ", $name, ", {}"), in(reg) value);
            }
        }
    };
}
