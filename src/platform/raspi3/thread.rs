use alloc::rc::Rc;
use core::arch::asm;

use alloc::vec;

use alloc::vec::Vec;

use alloc::collections::VecDeque;

use alloc::string::String;

use alloc::boxed::Box;

use crate::aarch64::interrupt::IRQLock;
use crate::platform::platform_devices::PLATFORM;
use crate::platform::raspi3::exception::InterruptFrame;

#[derive(Copy, Clone, Debug)]
pub enum ThreadStatus {
    Running,
    Ready,
    Waiting(u64),
    Dead,
}

#[derive(Debug)]
pub struct Thread<'a> {
    pub stack_pointer: IRQLock<*const u64>,
    pub parent: Option<&'a Thread<'a>>,
    pub status: IRQLock<ThreadStatus>,
    pub name: String,
}

impl<'a> Thread<'a> {
    pub fn from_current() -> Self {
        Self {
            stack_pointer: IRQLock::new(0x0 as *const u64),
            parent: None,
            status: IRQLock::new(ThreadStatus::Running),
            name: String::from("Idle"),
        }
    }

    pub fn return_to(&self) -> ! {
        unsafe {
            asm!(
                "mov sp, {sp}", sp = in(reg) *self.stack_pointer.lock()
            );

            asm!(
                "
                ldr x0, [sp, 0x100]
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
                // ldp x31, x0, [sp, 0x100]
                add sp, sp, 0x110
                ldr lr, [sp], #16
                msr daifclr, 0b10 // Enable Interrupts
                eret
                "
            );
        }

        loop {}
    }
}

pub struct Scheduler<'a> {
    pub current_thread: Rc<Thread<'a>>,
    pub threads: Vec<Rc<Thread<'a>>>,
    pub thread_queue: VecDeque<Rc<Thread<'a>>>,
    pub waiting_threads: Vec<Rc<Thread<'a>>>,
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Self {
        let current_thread = Rc::new(Thread::from_current());

        Self {
            current_thread: current_thread.clone(),
            threads: vec![current_thread],
            thread_queue: VecDeque::new(),
            waiting_threads: vec![],
        }
    }

    pub fn add_thread(&mut self, thread: Thread<'a>) {
        let thread = Rc::new(thread);
        self.thread_queue.push_back(thread.clone());
        self.threads.push(thread);
    }

    pub fn update_waits(&mut self) {
        let time = PLATFORM.get_timer().get_micros();

        for thread in self.threads.iter_mut() {
            if let ThreadStatus::Waiting(timeout) = *thread.status.lock() {
                if timeout < time {
                    *thread.status.lock() = ThreadStatus::Ready;
                    self.thread_queue.push_back(Rc::clone(thread));
                }
            }
        }
    }

    pub fn update_current(&mut self, frame: &InterruptFrame) {
        *self.current_thread.stack_pointer.lock() = frame as *const InterruptFrame as *const u64;
    }

    pub fn choose_thread(&mut self) -> Rc<Thread<'a>> {
        *self.current_thread.status.lock() = ThreadStatus::Ready;

        self.thread_queue.push_back(self.current_thread.clone());

        let new_thread = self
            .thread_queue
            .pop_front()
            .expect("Unable to find thread to schedule");

        self.current_thread = new_thread.clone();

        *new_thread.status.lock() = ThreadStatus::Running;

        return new_thread.clone();
    }

    pub fn return_to_current(&self) {
        self.current_thread.return_to();
    }

    pub fn set_current_stack_pointer(&mut self, pointer: *const u64) {
        *self.current_thread.stack_pointer.lock() = pointer;
    }

    pub fn schedule(&mut self) {
        self.thread_queue.push_back(self.current_thread.clone());

        self.current_thread = self.thread_queue.pop_front().expect("No threads on queue");
    }

    pub fn exit_current_thread(&mut self) {
        let dying_thread = self.current_thread.clone();
        *dying_thread.status.lock() = ThreadStatus::Dead;

        let dying_thread_index = self
            .threads
            .iter()
            .position(|thread| Rc::ptr_eq(thread, &dying_thread))
            .expect("Dying thread is not listed as a thread.");

        self.threads.remove(dying_thread_index);

        self.current_thread = self.thread_queue.pop_front().expect("No threads on queue");
    }
}
