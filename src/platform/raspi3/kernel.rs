use core::cell::{Ref, RefCell};

use crate::{
    aarch64::syscall::SyscallArgs,
    allocator::page_allocator::{self, PageAllocator},
    platform::thread::Scheduler,
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

    pub fn create_thread(&mut self, entry: fn() -> ()) {}

    pub fn handle_syscall(&mut self, number: usize, args: SyscallArgs) {
        crate::println!("Received system call {}, {:#?}", number, args);
    }
}
