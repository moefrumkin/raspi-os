pub enum Syscall {
    Thread = 0x1,
    Exit = 0x2,
}

pub type SyscallArgs = [usize; 3];

impl Syscall {
    pub fn from_u64(value: u64) -> Option<Self> {
        match value {
            1 => Some(Syscall::Thread),
            2 => Some(Syscall::Exit),
            _ => None,
        }
    }
}
