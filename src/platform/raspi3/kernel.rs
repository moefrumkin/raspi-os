use core::{
    cell::{Ref, RefCell},
    time::Duration,
};

use crate::{
    aarch64::syscall::{Syscall, SyscallArgs},
    allocator::page_allocator::{self, PageAllocator, PAGE_SIZE},
    platform::{
        platform_devices::get_platform,
        raspi3::exception::InterruptFrame,
        thread::{Scheduler, Thread, ThreadStatus},
    },
};

pub struct Kernel<'a> {
    pub scheduler: Scheduler<'a>,
    pub page_allocator: RefCell<PageAllocator<'a>>,
}

impl<'a> Kernel<'a> {
    pub fn with_page_allocator(page_allocator: RefCell<PageAllocator<'a>>) -> Self {
        Self {
            scheduler: Scheduler::new(),
            page_allocator,
        }
    }

    pub fn create_thread(&mut self, entry: usize) {
        let page_ref = self
            .page_allocator
            .borrow_mut()
            .allocate_page()
            .expect("Unable to Allocate Page");

        let stack_pointer;

        unsafe {
            let page = page_ref.page;

            let page64 = page as *mut u64;

            let mut sp = PAGE_SIZE / 8;

            sp -= 34;

            page64.offset(sp as isize).write(entry as u64);

            stack_pointer = sp as *const u64;
        }

        self.scheduler.add_thread(Thread {
            stack_pointer,
            parent: None,
            status: ThreadStatus::Ready,
        });
    }

    pub fn handle_syscall(&mut self, number: usize, args: SyscallArgs) {
        crate::println!("Received system call {}, {:#?}", number, args);

        if number == Syscall::Thread as usize {
            self.create_thread(args[0]);
        }
    }

    pub fn tick(&mut self, frame: &InterruptFrame) {
        let timer = get_platform().get_timer();
        //self.scheduler.update_current(frame);
        crate::println!("Tick!: {:?}", Duration::from_micros(timer.get_micros()));
    }
}
