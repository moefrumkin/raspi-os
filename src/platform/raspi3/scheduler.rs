use alloc::sync::Arc;

use alloc::vec;

use alloc::vec::Vec;

use alloc::collections::VecDeque;

use super::kernel_object::ObjectHandle;
use crate::platform::platform_devices::PLATFORM;
use crate::platform::raspi3::exception::InterruptFrame;
use crate::platform::thread::{Thread, ThreadID, ThreadStatus};

#[derive(Debug)]
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

    /// Update scheduling decisions
    pub fn schedule(&mut self) {
        let former_thread = Arc::clone(&self.current_thread);
        *former_thread.status.lock() = ThreadStatus::Ready;
        self.thread_queue.push_back(former_thread);

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

        *thread_to_delay.status.lock() = ThreadStatus::Sleeping(delay);

        self.waiting_threads.push(thread_to_delay);

        let new_thread = self.thread_queue.pop_front().expect("No threads on queue");

        *new_thread.status.lock() = ThreadStatus::Running;

        self.current_thread = new_thread;
    }

    /// Wake all threads that are sleeping
    pub fn wake_sleeping(&mut self) {
        let current_time = PLATFORM.get_timer().get_micros();

        self.waiting_threads.retain(|thread| {
            let wake_time = match *thread.status.lock() {
                ThreadStatus::Sleeping(wake_time) => wake_time,
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

    pub fn read(&mut self, handle: ObjectHandle, buffer: &mut [u8]) {
        let mut return_value = 0;

        {
            let objects = self.current_thread.objects.lock();

            for i in 0..objects.len() {
                let (id, o) = &objects[i];

                if *id == handle {
                    return_value = o.read(buffer);
                    break;
                }
            }
        }

        self.current_thread.set_return_value(return_value as u64);
    }

    pub fn write(&mut self, handle: ObjectHandle, buffer: &mut [u8]) {
        let mut return_value = 0;
        {
            let objects = self.current_thread.objects.lock();

            for i in 0..objects.len() {
                let (id, o) = &objects[i];

                if *id == handle {
                    return_value = o.write(buffer);
                    break;
                }
            }
        }

        self.current_thread.set_return_value(return_value as u64);
    }
}
