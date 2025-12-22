use super::thread::ThreadID;
use alloc::rc::Rc;
use alloc::sync::Arc;
use alloc::vec;
use core::{
    cell::{Ref, RefCell},
    slice, str,
    time::Duration,
};

use crate::{
    aarch64::{
        cpu,
        interrupt::IRQLock,
        mmu,
        syscall::{Syscall, SyscallArgs},
    },
    allocator::{
        id_allocator::IDAllocator,
        page_allocator::{self, Page, PageAllocator, PageRef, PAGE_SIZE},
    },
    elf::{ELF64Header, ProgramHeader},
    filesystem::{
        self,
        fat32::{FAT32DirectoryEntry, FAT32Filesystem},
    },
    platform::{
        framebuffer::FrameBuffer,
        kernel_object::FileObject,
        page_table::PageTable,
        platform_devices::{get_platform, PLATFORM},
        raspi3::exception::InterruptFrame,
        thread::{Scheduler, Thread, ThreadStatus},
    },
};

use alloc::boxed::Box;
use alloc::string::String;

use super::kernel_object::ObjectHandle;

pub const TICK: u32 = 1_000;

pub struct Kernel<'a> {
    pub scheduler: Scheduler<'a>,
    pub page_allocator: RefCell<PageAllocator<'a>>,
    pub thread_id_allocator: IDAllocator,
    pub object_id_allocator: IDAllocator,
    filesystem: Arc<IRQLock<FAT32Filesystem<'a>>>, // TODO: should this be here or on the platform?
}

impl<'a> Kernel<'a> {
    pub fn with_page_allocator_and_filesystem(
        page_allocator: RefCell<PageAllocator<'a>>,
        filesystem: IRQLock<FAT32Filesystem<'a>>,
    ) -> Self {
        Self {
            scheduler: Scheduler::new(),
            page_allocator,
            thread_id_allocator: IDAllocator::new(),
            object_id_allocator: IDAllocator::new(),
            filesystem: Arc::new(filesystem),
        }
    }

    pub fn allocate_page(&mut self) -> PageRef {
        self.page_allocator
            .borrow_mut()
            .allocate_page()
            .expect("Error allocationg page")
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
            objects: IRQLock::new(vec![]),
            kernel_table: self.scheduler.current_thread.kernel_table, // Currently all kernel threads have the same mapping
            user_table: PageTable::new_unmapped(),
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
            Syscall::Open => {
                self.open_object(unsafe { str::from_raw_parts(args[0] as *const u8, args[1]) })
            }
            Syscall::Close => self
                .scheduler
                .remove_object_from_current_thread(args[0] as u64),
            Syscall::Read => self.read_object(args[0] as u64, unsafe {
                slice::from_raw_parts_mut(args[1] as *mut u8, args[2])
            }),
            _ => panic!("Unsupported Syscall"),
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

    pub fn open_object(&mut self, name: &str) {
        let entry = self.filesystem.lock().search_item(name);

        if let Some(entry) = entry {
            let id = self.object_id_allocator.allocate_id();

            self.scheduler
                .add_object_to_current_thread(Box::new(FileObject::from_entry(entry)), id);

            self.scheduler.set_current_thread_return(id);
        } else {
            self.scheduler.set_current_thread_return(0);
        }
    }

    pub fn read_object(&mut self, handle: ObjectHandle, buffer: &mut [u8]) {
        self.scheduler.read(handle, buffer);
    }

    pub fn read(&self, entry: FAT32DirectoryEntry, buffer: &mut [u8]) -> usize {
        self.filesystem.lock().read_file(entry, buffer)
    }

    pub fn exec(&mut self, program: &str) {
        let handle = cpu::open_object(program);

        let mut buffer: [u8; 8192] = [b'\0'; 8192];

        crate::println!("Reading program {}", program);

        let bytes_read = cpu::read_object(handle, &mut buffer);

        crate::println!("Parsing ELF");

        let header = ELF64Header::try_from(&buffer[0..bytes_read]).expect("Error parsing elf");

        let mut pheaders = vec![];

        let pheader_start = header.program_header_offset;

        for i in 0..header.program_header_number {
            let pheader_offset = pheader_start + ((header.program_header_entry_size * i) as u64);

            let phdr = unsafe {
                let buffer_offest = buffer.as_ptr().offset(pheader_offset as isize);

                *(buffer_offest as *const ProgramHeader)
            };

            crate::println!("Header: {:?}\n", phdr);

            pheaders.push(phdr);
        }

        for ref pheader in pheaders {}

        cpu::close_object(handle);
    }
}
