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

pub fn create_thread<T>(function: extern "C" fn(arg: T) -> (), name: String, arg: usize) -> u64 {
    start_thread(function, &name, arg)
}

pub extern "C" fn start_thread<T>(
    _function: extern "C" fn(arg: T) -> (),
    _name: *const String,
    _arg: usize,
) -> u64 {
    unsafe {
        asm!("svc {}", const Syscall::Thread as usize);
    }

    let thread_id: u64;
    unsafe {
        asm!("mov {}, x0", out(reg) thread_id);
    }

    thread_id
}

pub extern "C" fn exit_thread(_code: u64) {
    unsafe {
        asm!("svc {}", const Syscall::Exit as usize);
    }
}

pub extern "C" fn sleep(_micros: u64) {
    unsafe {
        asm!("svc {}", const Syscall::Wait as usize);
    }
}

pub extern "C" fn join_thread(_thread_id: u64) -> u64 {
    unsafe {
        asm!("svc {}", const Syscall::Join as usize);
    }

    let return_code: u64;

    unsafe {
        asm!("mov {}, x0", out(reg) return_code);
    }

    return_code
}

pub extern "C" fn yield_thread() {
    unsafe {
        asm!("svc {}", const Syscall::Yield as usize);
    }
}

pub fn open_object(name: &str) -> u64 {
    let ptr = name.as_ptr();
    let size = name.len();

    let object_handle: u64;

    unsafe {
        asm!("
            mov x0, {}
            mov x1, {}
        ", 
        in(reg) ptr,
        in(reg) size
    );

    asm!("svc {}", const Syscall::Open as usize);

    asm!("mov {}, x0", out(reg) object_handle);
    }

    object_handle
}

pub fn close_object(handle: u64) {
    unsafe {
        asm!("mov x0, {}", in(reg) handle);

        asm!("svc {}", const Syscall::Close as usize);
    }
}

pub fn read_object(handle: u64, buffer: &mut [char]) -> usize {
    unsafe {
        asm!(
            "mov x0, {}
            mov x1, {}
            mov x2, {}",
            in(reg) handle,
            in(reg) buffer.as_ptr() as usize,
            in(reg) buffer.len()
        );
    }

    unsafe {
        asm!("svc {}", const Syscall::Read as usize);
    }

    let bytes_read;

    unsafe {
        asm!("mov {}, x0", out(reg) bytes_read);
    }

    bytes_read
}