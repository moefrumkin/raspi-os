use core::arch::asm;

use alloc::vec;

use alloc::vec::Vec;

use crate::platform::raspi3::exception::InterruptFrame;

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

    pub fn return_to(&self) -> ! {
        unsafe {
            asm!(
                "mov sp, {sp}", sp = in(reg) &self.stack_pointer
            );

            asm!(
                "
                ldp x21, x0, [sp, 0x100]
                msr elr_el1, x0
                ldp x0, x1, [sp, 0x0]
                ldp x2, x3, [sp, 0x10]
                ldp x4, x5, [sp, 0x20]
                ldp x6, x7, [sp, 0x30]
                ldp x8, x9, [sp, 0x40]
                ldp x10, x11, [sp, 0x50]
                ldp x12, x13, [sp, 0x60]
                ldp x14, x15, [sp, 0x70]
                ldp x16, x17, [sp, 0x90]
                ldp x18, x19, [sp, 0xa0]
                ldp x20, x21, [sp, 0xb0]
                ldp x22, x23, [sp, 0xc0]
                ldp x24, x25, [sp, 0xd0]
                ldp x26, x27, [sp, 0xe0]
                ldp x28, x29, [sp, 0xf0]
                // ldp x30, xzr, [sp, #160]
                ldp x31, x0, [sp, 0x100]
                add sp, sp, 0x110"
            );

            asm!("ldr lr, [sp], #16");
        }

        loop {}
    }
}

pub struct Scheduler<'a> {
    current_thread: Thread<'a>,
    threads: Vec<Thread<'a>>,
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Self {
        Self {
            current_thread: Thread::from_current(),
            threads: vec![],
        }
    }

    pub fn add_thread(&mut self, thread: Thread<'a>) {
        self.threads.push(thread);
    }

    pub fn update_current(&mut self, frame: &InterruptFrame) {
        self.current_thread.stack_pointer = frame as *const InterruptFrame as *const u64;
    }
}
