//! Systems calls provide an interface for kernel functionality.
//! Kernels threads may also use system calls just like user threads.
//! This crate defines the system call numbers and wrapper functions to issue system calls from the kernel.

use alloc::string::String;
use core::arch::asm;

pub enum Syscall {
    Thread = 0x1,
    Exit = 0x2,
    Wait = 0x3,
    Join = 0x4,
    Yield = 0x5,

    Open = 0x6,
    Close = 0x7,
    Read = 0x8,
    Write = 0x9,

    Exec = 0xa,
}

pub type SyscallArgs = [usize; 3];

impl TryFrom<u64> for Syscall {
    type Error = &'static str;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0x1 => Ok(Syscall::Thread),
            0x2 => Ok(Syscall::Exit),
            0x3 => Ok(Syscall::Wait),
            0x4 => Ok(Syscall::Join),
            0x5 => Ok(Syscall::Yield),

            0x6 => Ok(Syscall::Open),
            0x7 => Ok(Syscall::Close),
            0x8 => Ok(Syscall::Read),
            0x9 => Ok(Syscall::Write),
            0xa => Ok(Syscall::Exec),
            _ => Err("Invalid Syscall Number"),
        }
    }
}

// Wrappers to invoke system calls from the kernel.
// These functions are marked as extern "C" to ensure they follow the calling convention

/// Produces the assembly instruction to issue the system call.
macro_rules! syscall {
    ($number: expr) => {
        unsafe {
            asm!("svc {}", const $number as usize);
        }
    };
}

/// Produces a wrapper function for a syscall using a specific number of arguments and a specific return type
macro_rules! wrap_syscall {
    ($number: expr, $name: ident,
        ($($arg:ident: $type:ty),*) $(-> $return_type: ty)?
    ) => {
        #[allow(unused_variables)]
        pub extern "C" fn $name($($arg: $type)*) $(-> $return_type)? {
            syscall!($number);

            $(
                return_x0() as $return_type
            )?
        }
    };
}

pub fn create_thread<T>(f: extern "C" fn(arg: T) -> (), name: String, arg: usize) -> u64 {
    start_thread(f, &name, arg)
}

/// Returns the value in x0.
/// This is useful since the rust compiler doesn't know that syscalls return their value in x0
/// This function should be optimized away
#[inline(always)]
fn return_x0() -> u64 {
    let x0: u64;

    unsafe {
        asm!("mov {}, x0", out(reg) x0);
    }

    x0
}

extern "C" fn start_thread<T>(
    _f: extern "C" fn(arg: T) -> (),
    _name: *const String,
    _arg: usize,
) -> u64 {
    syscall!(Syscall::Thread);

    return_x0()
}

wrap_syscall!(Syscall::Exit, exit, (code: u64));

wrap_syscall!(Syscall::Wait, sleep, (micros: u64));

wrap_syscall!(Syscall::Join, join, (thread_id: u64) -> u64);

wrap_syscall!(Syscall::Yield, yield_thread, ());

pub fn open(name: &str) -> u64 {
    // We calculate these values before the asm block just in case .as_ptr or .len changes the values of x0 or x1 in ways we wouldn't want
    // TODO: is this necessary based on the semantics of the asm macro?
    let name_ptr = name.as_ptr();
    let name_size = name.len();

    unsafe {
        asm!("
            mov x0, {}
            mov x1, {}
        ",
            in(reg) name_ptr,
            in(reg) name_size
        )
    }

    syscall!(Syscall::Open);

    return_x0()
}

wrap_syscall!(Syscall::Close, close, (handle: u64));

pub fn read(handle: u64, buffer: &mut [u8]) -> usize {
    let buffer_ptr = buffer.as_ptr() as usize;
    let buffer_len = buffer.len() as usize;

    unsafe {
        asm!(
            "mov x0, {}
            mov x1, {}
            mov x2, {}",
            in(reg) handle,
            in(reg) buffer_ptr,
            in(reg) buffer_len
        );
    }

    syscall!(Syscall::Read);

    return_x0() as usize
}

pub fn write(handle: u64, buffer: &[u8]) -> usize {
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

    syscall!(Syscall::Write);

    return_x0() as usize
}
