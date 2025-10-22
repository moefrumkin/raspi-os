use alloc::rc::Rc;
use core::{
    cell::{Ref, RefCell},
    time::Duration,
};

use crate::{
    aarch64::{
        interrupt::IRQLock,
        syscall::{Syscall, SyscallArgs},
    },
    allocator::page_allocator::{self, PageAllocator, PAGE_SIZE},
    platform::{
        framebuffer::FrameBuffer,
        platform_devices::get_platform,
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
}

impl<'a> Kernel<'a> {
    pub fn with_page_allocator(page_allocator: RefCell<PageAllocator<'a>>) -> Self {
        Self {
            scheduler: Scheduler::new(),
            page_allocator,
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

            sp -= 34; // TODO: use size_of instead of a magic number

            let frame = &mut *(page64.offset(sp as isize) as *mut InterruptFrame);

            frame.regs[0] = args[2] as u64;

            frame.elr = entry as u64;

            crate::println!("Frame: {:?}", frame);

            stack_pointer = IRQLock::new(page64.offset(sp as isize) as *const u64);

            name = Box::from_raw(args[1] as *mut String)
        }

        self.scheduler.add_thread(Thread {
            stack_pointer,
            parent: None,
            status: IRQLock::new(ThreadStatus::Ready),
            name,
        });
    }

    pub fn handle_syscall(&mut self, number: usize, args: SyscallArgs) {
        if number == Syscall::Thread as usize {
            self.create_thread(args[0], args);
        }
    }

    pub fn tick(&mut self, frame: &InterruptFrame) {
        self.scheduler.update_current(frame);
    }

    pub fn get_return_thread(&mut self) -> Rc<Thread<'a>> {
        self.scheduler.choose_thread()
    }

    pub fn update_frame(&mut self, frame: &mut InterruptFrame) {
        self.scheduler
            .set_current_stack_pointer(frame as *const InterruptFrame as *const u64);
    }
}
