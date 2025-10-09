pub enum Syscall {
    Thread = 0x1,
}

pub type SyscallArgs = [usize; 3];
