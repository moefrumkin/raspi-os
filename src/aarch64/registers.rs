#[macro_export]
macro_rules! read {
    ($sysreg: literal) => {
        unsafe { 
            let value: usize;
            asm!(concat!("mrs {}, " , $sysreg), out(reg) value);
            value
        }
    }
}

#[macro_export]
macro_rules! write {
    ($sysreg: literal, $value: expr) => {
        unsafe {
            asm!(concat!("msr ", $sysreg, ", {}"), in(reg) $value);
        }
    };
}