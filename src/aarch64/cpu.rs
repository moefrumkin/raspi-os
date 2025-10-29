use alloc::boxed::Box;
use alloc::string::String;
use core::arch::asm;

use crate::{aarch64::syscall::Syscall, read, write};

/// Returns the id of the cpu core as reported by the arm MPIDR_EL1 system register
#[allow(dead_code)]
pub fn core_id() -> usize {
    (read!("MPIDR_EL1") & 0xff) as usize
}

/// Returns the execution level when called
#[allow(dead_code)]
pub fn el() -> usize {
    (read!("CurrentEL") & 0b1100) >> 2
}

pub fn nop() {
    unsafe {
        asm!("nop");
    }
}

#[allow(dead_code)]
pub fn wait_for_cycles(cycles: u64) {
    for _ in 0..cycles {
        unsafe {
            asm!("nop");
        }
    }
}

pub fn eret() {
    unsafe {
        asm!("eret");
    }
}

pub fn instruction_buffer() {
    unsafe {
        asm!("isb");
    }
}

pub fn data_buffer() {
    unsafe {
        asm!("dsb sy");
    }
}

pub fn create_thread<T>(function: extern "C" fn(arg: T) -> (), name: String, arg: usize) {
    start_thread(function, &name, arg);
}

pub extern "C" fn start_thread<T>(
    _function: extern "C" fn(arg: T) -> (),
    _name: *const String,
    _arg: usize,
) {
    unsafe {
        asm!("svc {}", const Syscall::Thread as usize);
    }
}

pub extern "C" fn exit_thread() {
    unsafe {
        asm!("svc {}", const Syscall::Exit as usize);
    }
}

pub extern "C" fn sleep(_micros: u64) {
    unsafe {
        asm!("svc {}", const Syscall::Wait as usize);
    }
}
