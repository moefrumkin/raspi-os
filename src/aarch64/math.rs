use core::arch::global_asm;

global_asm!(include_str!("math.s"));

extern "C" {
    pub fn sqrt_asm(f: f64) -> f64;
    pub fn abs_asm(f: f64) -> f64;
    pub fn round_asm(f: f64) -> isize;
    pub fn sin_asm(f: f64) -> f64;
    pub fn cos_asm(f: f64) -> f64;
}

pub fn sqrt(f: f64) -> f64 {
    unsafe {
        sqrt_asm(f)
    }
}

pub fn abs(f: f64) -> f64 {
    unsafe {
        abs_asm(f)
    }
}

pub fn round(f: f64) -> isize {
    unsafe {
        round_asm(f)
    }
}

pub fn sin(f: f64) -> f64 {
    unimplemented!()
}

pub fn cos(f: f64) -> f64 {
    unimplemented!()
}