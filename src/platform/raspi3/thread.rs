use alloc::vec;

use alloc::vec::Vec;

pub enum ThreadStatus {
    Running,
    Ready,
}

pub struct Thread<'a> {
    pub stack_pointer: *const u64,
    pub parent: Option<&'a Thread<'a>>,
    pub status: ThreadStatus,
}

impl<'a> Thread<'a> {
    pub fn from_current() -> Self {
        Self {
            stack_pointer: 0x0 as *const u64,
            parent: None,
            status: ThreadStatus::Running,
        }
    }
}

pub struct Scheduler<'a> {
    threads: Vec<Thread<'a>>,
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Self {
        Self {
            threads: vec![Thread::from_current()],
        }
    }

    pub fn add_thread(&mut self, thread: Thread<'a>) {
        self.threads.push(thread);
    }
}
