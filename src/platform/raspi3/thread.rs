use alloc::rc::Rc;
use alloc::sync::Arc;
use core::arch::asm;

use alloc::vec;

use alloc::vec::Vec;

use alloc::collections::VecDeque;

use alloc::string::String;

use alloc::boxed::Box;

use crate::aarch64::interrupt::IRQLock;
use crate::platform::platform_devices::PLATFORM;
use crate::platform::raspi3::exception::InterruptFrame;
use super::kernel_object::{
    ObjectHandle,
    KernelObject
};

#[derive(Copy, Clone, Debug)]
pub enum ThreadStatus {
    Running,
    Ready,
    Waiting(u64),
    Exited(u64),
    Joining(ThreadID),
}

pub type ThreadID = u64;

#[derive(Debug)]
// TODO: implement Drop trait
pub struct Thread<'a> {
    pub stack_pointer: IRQLock<*const u64>,
    pub parent: Option<Arc<Thread<'a>>>,
    pub status: IRQLock<ThreadStatus>,
    pub name: String,
    pub id: u64,
    pub children: IRQLock<Vec<Arc<Thread<'a>>>>,
    pub objects: IRQLock<Vec<(ObjectHandle, Box<dyn KernelObject>)>> // TODO: find a more efficient way of doing this
}

impl<'a> Thread<'a> {
    pub fn from_current() -> Self {
        Self {
            stack_pointer: IRQLock::new(0x0 as *const u64),
            parent: None,
            status: IRQLock::new(ThreadStatus::Running),
            name: String::from("Idle"),
            id: 0,
            children: IRQLock::new(vec![]),
            objects: IRQLock::new(vec![])
        }
    }

    pub fn return_to(&self) -> ! {
        unsafe {
            asm!(
                "mov sp, {sp}", sp = in(reg) *self.stack_pointer.lock()
            );

            asm!(
                "
                ldp x0, x1, [sp, 0x100]
                msr elr_el1, x0
                msr spsr_el1, x1
                ldr x1, [sp, 0x350]
                msr fpsr, x1

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

                ldp q0, q1, [sp, 0x110]
                ldp q2, q3, [sp, 0x130]
                ldp q4, q5, [sp, 0x150]
                ldp q6, q7, [sp, 0x170]
                ldp q8, q9, [sp, 0x190]
                ldp q10, q11, [sp, 0x1b0]
                ldp q12, q13, [sp, 0x1e0]
                ldp q14, q15, [sp, 0x210]
                ldp q16, q17, [sp, 0x230]
                ldp q18, q19, [sp, 0x250]
                ldp q20, q21, [sp, 0x270]
                ldp q22, q23, [sp, 0x290]
                ldp q24, q25, [sp, 0x2b0]
                ldp q26, q27, [sp, 0x2e0]
                ldp q28, q29, [sp, 0x310]
                ldp q30, q31, [sp, 0x330]

                add sp, sp, 0x360
                ldr lr, [sp], #16
                eret
                "
            );
        }

        loop {}
    }

    /// Unsafe if the stack pointer is not accurate
    /// TODO: for memory safety, shyould this require a mutable ref to self?
    fn set_return_value(&self, value: u64) {
        unsafe {
            let frame = &mut *(*self.stack_pointer.lock() as *mut InterruptFrame);
            frame.regs[0] = value;
        }
    }
}

pub struct Scheduler<'a> {
    pub current_thread: Arc<Thread<'a>>,
    pub threads: Vec<Arc<Thread<'a>>>,
    pub thread_queue: VecDeque<Arc<Thread<'a>>>,
    pub waiting_threads: Vec<Arc<Thread<'a>>>,
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Self {
        let current_thread = Arc::new(Thread::from_current());

        Self {
            current_thread: current_thread.clone(),
            threads: vec![current_thread],
            thread_queue: VecDeque::new(),
            waiting_threads: vec![],
        }
    }

    pub fn add_thread(&mut self, thread: Thread<'a>) {
        let thread = Arc::new(thread);
        self.thread_queue.push_back(Arc::clone(&thread));

        self.current_thread.children.lock().push(thread.clone());

        self.threads.push(thread);

        /*for thread in &self.threads {
            crate::println!(
                "{} (Strong count {}) @ {:#?}: {:?}",
                &thread.name,
                Rc::strong_count(thread),
                Rc::as_ptr(thread),
                *thread.status.lock()
            );
        }*/

        //crate::println!("\n\n\n");
    }

    pub fn update_waits(&mut self) {
        let time = PLATFORM.get_timer().get_micros();

        for thread in self.threads.iter_mut() {
            if let ThreadStatus::Waiting(timeout) = *thread.status.lock() {
                if timeout < time {
                    *thread.status.lock() = ThreadStatus::Ready;
                    self.thread_queue.push_back(Arc::clone(thread));
                }
            }
        }
    }

    pub fn update_current(&mut self, frame: &InterruptFrame) {
        *self.current_thread.stack_pointer.lock() = frame as *const InterruptFrame as *const u64;
    }

    pub fn choose_thread(&mut self) -> Arc<Thread<'a>> {
        *self.current_thread.status.lock() = ThreadStatus::Ready;

        self.thread_queue
            .push_back(Arc::clone(&self.current_thread));

        let new_thread = self
            .thread_queue
            .pop_front()
            .expect("Unable to find thread to schedule");

        self.current_thread = Arc::clone(&new_thread);

        *new_thread.status.lock() = ThreadStatus::Running;

        return Arc::clone(&new_thread);
    }

    pub fn return_to_current(&self) {
        self.current_thread.return_to();
    }

    pub fn set_current_stack_pointer(&mut self, pointer: *const u64) {
        *self.current_thread.stack_pointer.lock() = pointer;
    }

    pub fn schedule(&mut self) {
        /*crate::println!("\n\n\nScheduling \n\n");
        crate::println!("Threads:");
        for thread in &self.threads {
            crate::println!(
                "{} (Strong count {}) @ {:#?}: {:?}",
                &thread.name,
                Rc::strong_count(thread),
                Rc::as_ptr(thread),
                *thread.status.lock()
            );
        }

        crate::println!("Queue:");
        for thread in &self.thread_queue {
            crate::println!(
                "{} (Strong count {}) @ {:#?}: {:?}",
                &thread.name,
                Rc::strong_count(thread),
                Rc::as_ptr(thread),
                *thread.status.lock()
            );
        }
        crate::println!(
            "Cloning! {:?}. With strong count {}",
            &self.current_thread,
            Rc::strong_count(&self.current_thread)
        );*/
        let former_thread = Arc::clone(&self.current_thread);
        //crate::println!("Cloned!");
        *former_thread.status.lock() = ThreadStatus::Ready;
        self.thread_queue.push_back(former_thread);

        //crate::println!("Old pushed");

        let new_thread = self.thread_queue.pop_front().expect("No threads on queue");
        *new_thread.status.lock() = ThreadStatus::Running;
        self.current_thread = new_thread;
    }

    pub fn exit_current_thread(&mut self, code: u64) {
        let dying_thread = Arc::clone(&self.current_thread);
        *dying_thread.status.lock() = ThreadStatus::Exited(code);

        let dying_thread_index = self
            .threads
            .iter()
            .position(|thread| Arc::ptr_eq(thread, &dying_thread))
            .expect("Dying thread is not listed as a thread.");
        self.threads.remove(dying_thread_index);

        let dying_thread_id = dying_thread.id;

        self.threads.iter().for_each(|thread| {
            if let ThreadStatus::Joining(id) = *thread.status.lock() {
                if id == dying_thread_id {
                    unsafe {
                        let frame = &mut *(*thread.stack_pointer.lock() as *mut InterruptFrame);

                        frame.regs[0] = code;

                        self.thread_queue.push_back(thread.clone());
                    }
                }
            }
        });

        /*if let Some(ref parent) = dying_thread.parent {
            let children = &mut parent.children.lock();

            children.retain(|child| child.id == dying_thread_id);
        }*/

        self.current_thread = self.thread_queue.pop_front().expect("No threads on queue");
    }

    pub fn delay_current_thread(&mut self, delay: u64) {
        let thread_to_delay = Arc::clone(&self.current_thread);

        *thread_to_delay.status.lock() = ThreadStatus::Waiting(delay);

        self.waiting_threads.push(thread_to_delay);

        let new_thread = self.thread_queue.pop_front().expect("No threads on queue");

        *new_thread.status.lock() = ThreadStatus::Running;

        self.current_thread = new_thread;
    }

    pub fn get_next_thread_wakeup(&self) -> Option<u64> {
        self.waiting_threads
            .iter()
            .map(|thread| match *thread.status.lock() {
                ThreadStatus::Waiting(wake_time) => wake_time,
                _ => panic!("Non waiting thread on waiting thread queue"),
            })
            .min()
    }

    pub fn wake_sleeping(&mut self) {
        let current_time = PLATFORM.get_timer().get_micros();

        self.waiting_threads.retain(|thread| {
            let wake_time = match *thread.status.lock() {
                ThreadStatus::Waiting(wake_time) => wake_time,
                _ => panic!("Non waiting thread on waiting thread queue"),
            };

            if wake_time <= current_time {
                *thread.status.lock() = ThreadStatus::Ready;

                self.thread_queue.push_back(Arc::clone(&thread));

                return false;
            } else {
                return true;
            }
        })
    }

    pub fn set_current_thread_return(&mut self, value: u64) {
        let frame = *self.current_thread.stack_pointer.lock() as *mut InterruptFrame;

        unsafe {
            let frame = &mut *frame;
            frame.regs[0] = value;
        }
    }

    pub fn join_current_thread(&mut self, thread_id: ThreadID) {
        let child_thread_index = self
            .current_thread
            .children
            .lock()
            .iter()
            .position(|child| child.id == thread_id)
            .expect(alloc::format!("Waiting on child that doesn't exist {}", thread_id).as_str());

        let child_thread_status = *self.current_thread.children.lock()[child_thread_index]
            .status
            .lock();

        if let ThreadStatus::Exited(exit_code) = child_thread_status {
            self.current_thread
                .children
                .lock()
                .remove(child_thread_index);

            self.current_thread.set_return_value(exit_code);
        } else {
            self.current_thread.status.lock();
            *self.current_thread.status.lock() = ThreadStatus::Joining(thread_id);

            self.current_thread = self.thread_queue.pop_front().expect("No threads on queue");
        }
    }

    pub fn yield_current_thread(&mut self) {
        let yielding_thread = self.current_thread.clone();

        self.thread_queue.push_back(yielding_thread);

        self.current_thread = self.thread_queue.pop_front().expect("No threads on queue");
    }

    pub fn add_object_to_current_thread(&self, object: Box<dyn KernelObject>, id: ObjectHandle) {
        self.current_thread.objects.lock().push((id, object));
    }

    pub fn remove_object_from_current_thread(&self, handle: ObjectHandle) {
        // TODO: error handling?
        // TODO: will this call drop?

        self.current_thread.objects.lock().retain(|(id, _)|
            *id != handle
        );
    }
}
