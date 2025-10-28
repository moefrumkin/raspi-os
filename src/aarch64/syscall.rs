pub enum Syscall {
    Thread = 0x1,
    Exit = 0x2,
    Wait = 0x3,
}

pub type SyscallArgs = [usize; 3];

impl Syscall {
    pub fn from_u64(value: u64) -> Option<Self> {
        match value {
            0x1 => Some(Syscall::Thread),
            0x2 => Some(Syscall::Exit),
            0x3 => Some(Syscall::Wait),
            _ => None,
        }
    }
}
