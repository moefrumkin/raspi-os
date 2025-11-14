use super::thread::ThreadID;
use alloc::rc::Rc;
use alloc::sync::Arc;
use alloc::vec;
use core::{
    cell::{Ref, RefCell},
    time::Duration,
};

use crate::{
    aarch64::{
        interrupt::IRQLock,
        syscall::{Syscall, SyscallArgs},
    },
    allocator::{
        id_allocator::IDAllocator,
        page_allocator::{self, PageAllocator, PAGE_SIZE},
    },
    platform::{
        framebuffer::FrameBuffer,
        platform_devices::{get_platform, PLATFORM},
        raspi3::exception::InterruptFrame,
        thread::{Scheduler, Thread, ThreadStatus},
    },
};

use alloc::boxed::Box;
use alloc::string::String;

pub const TICK: u32 = 1_000;

pub struct Kernel<'a> {
    pub scheduler: Scheduler<'a>,
    pub page_allocator: RefCell<PageAllocator<'a>>,
    pub thread_id_allocator: IDAllocator,
}

impl<'a> Kernel<'a> {
    pub fn with_page_allocator(page_allocator: RefCell<PageAllocator<'a>>) -> Self {
        Self {
            scheduler: Scheduler::new(),
            page_allocator,
            thread_id_allocator: IDAllocator::new(),
        }
    }

    pub fn create_thread(&mut self, entry: usize, args: SyscallArgs) {
        let page_ref = self
            .page_allocator
            .borrow_mut()
            .allocate_page()
            .expect("Unable to Allocate Page");

        let stack_pointer;
        let name;

        unsafe {
            let page = page_ref.page;

            let page64 = page as *mut u64;

            let mut sp = PAGE_SIZE / 8;

            sp -= 106; // TODO: use size_of instead of a magic number

            let frame = &mut *(page64.offset(sp as isize) as *mut InterruptFrame);

            frame.regs[0] = args[2] as u64;

            frame.elr = entry as u64;
            frame.spsr = 0b101; // EL1 with SP_EL1h

            stack_pointer = IRQLock::new(page64.offset(sp as isize) as *const u64);

            name = String::from(&*(args[1] as *mut String));
        }

        let id = self.thread_id_allocator.allocate_id();

        self.scheduler.add_thread(Thread {
            stack_pointer,
            parent: Some(self.scheduler.current_thread.clone()),
            status: IRQLock::new(ThreadStatus::Ready),
            name,
            id,
            children: IRQLock::new(vec![]),
        });

        self.scheduler.set_current_thread_return(id);
    }

    pub fn handle_syscall(&mut self, number: usize, args: SyscallArgs) {
        let syscall = Syscall::from_u64(number as u64).expect("Invalid Syscall Number");
        match syscall {
            Syscall::Thread => self.create_thread(args[0], args),
            Syscall::Exit => self.exit_current_thread(args[0] as u64),
            Syscall::Wait => self.delay_current_thread(args[0] as u64),
            Syscall::Join => self.join_current_thread(args[0] as ThreadID),
            Syscall::Yield => self.scheduler.yield_current_thread(),
        }
    }

    pub fn tick(&mut self) {
        self.scheduler.wake_sleeping();
        self.scheduler.schedule();
    }

    pub fn get_return_thread(&mut self) -> Arc<Thread<'a>> {
        self.scheduler.choose_thread()
    }

    pub fn return_from_exception(&self) {
        self.scheduler.return_to_current();
    }

    pub fn save_current_frame(&mut self, frame: &mut InterruptFrame) {
        self.scheduler
            .set_current_stack_pointer(frame as *const InterruptFrame as *const u64);
    }

    pub fn exit_current_thread(&mut self, code: u64) {
        self.scheduler.exit_current_thread(code);
    }

    pub fn delay_current_thread(&mut self, delay: u64) {
        // TODO: what is wake up time is before the current time because of the time the computations take?
        let current_time = PLATFORM.get_timer().get_micros();
        let delay_end = current_time + delay;

        self.scheduler.delay_current_thread(delay_end);

        //let new_timeout = self.scheduler.get_next_thread_wakeup().unwrap() - current_time;

        //PLATFORM.get_timer().set_timeout(new_timeout as u32);
    }

    pub fn join_current_thread(&mut self, thread_id: ThreadID) {
        self.scheduler.join_current_thread(thread_id);
    }
}
