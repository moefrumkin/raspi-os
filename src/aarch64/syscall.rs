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

macro_rules! syscall {
    ($number: expr) => {
        unsafe {
            asm!("svc {}", const $number as usize);
        }
    };
}

pub fn create_thread<T>(f: extern "C" fn(arg: T) -> (), name: String, arg: usize) -> u64 {
    start_thread(f, &name, arg)
}

// Should be optimized away
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

pub extern "C" fn exit(_code: u64) {
    syscall!(Syscall::Exit)
}

pub extern "C" fn sleep(_micros: u64) {
    syscall!(Syscall::Wait);
}

pub extern "C" fn join(_thread_id: u64) -> u64 {
    syscall!(Syscall::Join);

    return_x0()
}

pub extern "C" fn yield_thread() {
    syscall!(Syscall::Yield);
}

pub fn open(name: &str) -> u64 {
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

pub extern "C" fn close(_handle: u64) {
    syscall!(Syscall::Close);
}

pub fn read(handle: u64, buffer: &mut [u8]) -> usize {
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
