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

impl Syscall {
    pub fn from_u64(value: u64) -> Option<Self> {
        match value {
            0x1 => Some(Syscall::Thread),
            0x2 => Some(Syscall::Exit),
            0x3 => Some(Syscall::Wait),
            0x4 => Some(Syscall::Join),
            0x5 => Some(Syscall::Yield),

            0x6 => Some(Syscall::Open),
            0x7 => Some(Syscall::Close),
            0x8 => Some(Syscall::Read),
            0x9 => Some(Syscall::Write),
            0xa => Some(Syscall::Exec),
            _ => None,
        }
    }
}
