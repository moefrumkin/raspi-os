use super::thread::ThreadID;
use alloc::sync::Arc;
use alloc::vec;
use core::{slice, str};

use crate::{
    aarch64::{
        interrupt::IRQLock,
        syscall::{Syscall, SyscallArgs},
    },
    allocator::{
        id_allocator::IDAllocator,
        page_allocator::{PageAllocator, PageRef},
    },
    filesystem::fat32::{FAT32DirectoryEntry, FAT32Filesystem},
    platform::{
        kernel_object::{FileObject, ObjectHandle, Stdio},
        page_table::PageTable,
        platform_devices::PLATFORM,
        raspi3::exception::InterruptFrame,
        scheduler::Scheduler,
        thread::{Thread, ThreadStatus},
    },
};

use alloc::string::String;

pub const TICK: u32 = 1_000;

#[derive(Debug)]
pub struct Kernel<'a> {
    pub scheduler: Scheduler<'a>,
    pub page_allocator: IRQLock<PageAllocator<'a>>,
    pub thread_id_allocator: IDAllocator,
    pub object_id_allocator: IDAllocator,
    filesystem: Arc<IRQLock<FAT32Filesystem<'a>>>, // TODO: should this be here or on the platform?
}

impl<'a> Kernel<'a> {
    pub fn with_page_allocator_and_filesystem(
        page_allocator: IRQLock<PageAllocator<'a>>,
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

    // TODO: should be able to eliminate
    pub fn allocate_page(&mut self) -> PageRef {
        self.page_allocator
            .lock()
            .allocate_page()
            .expect("Error allocationg page")
    }

    fn get_current_thread(&self) -> &Arc<Thread<'a>> {
        &self.scheduler.current_thread
    }

    /// Processess a system call with the given number and arguments. Does not return control to a thread but may change scheduling depending on the system call.
    /// TODO: add pointer validity checks
    pub fn handle_syscall(&mut self, number: usize, args: SyscallArgs) {
        let syscall = Syscall::try_from(number as u64).expect("Invalid Syscall Number");
        match syscall {
            Syscall::Thread => self.create_thread(args[0], args),
            Syscall::Exit => self.scheduler.exit_current_thread(args[0] as u64),
            Syscall::Wait => self
                .scheduler
                .delay_current_thread(PLATFORM.get_timer().get_micros() + args[0] as u64),
            Syscall::Join => self.scheduler.join_current_thread(args[0] as ThreadID),
            Syscall::Yield => self.scheduler.yield_current_thread(),
            Syscall::Open => {
                let name = unsafe { str::from_raw_parts(args[0] as *const u8, args[1]) };
                self.open_object(name);
            }
            Syscall::Close => self.get_current_thread().remove_object(args[0] as u64),
            Syscall::Read => {
                let handle = args[0] as ObjectHandle;
                let buffer = unsafe { slice::from_raw_parts_mut(args[1] as *mut u8, args[2]) };
                self.read(handle, buffer);
            }
            Syscall::Write => {
                let handle = args[0] as ObjectHandle;
                let buffer = unsafe { slice::from_raw_parts_mut(args[1] as *mut u8, args[2]) };
                self.write(handle, buffer);
            }
            _ => panic!("Unsupported Syscall"),
        }
    }

    pub fn exec(&mut self, program_name: &str) {
        self.get_current_thread().exec(program_name);
    }

    /// Handles a system tick:
    /// 1. Wakes sleeping threads
    /// 2. Updates scheduling decisions
    ///
    /// Does not return to a thread
    pub fn tick(&mut self) {
        self.scheduler.wake_sleeping();
        self.scheduler.schedule();
    }

    pub fn get_return_thread(&mut self) -> Arc<Thread<'a>> {
        self.scheduler.choose_thread()
    }

    pub fn return_from_exception(&self) {
        self.get_current_thread().return_to();
    }

    pub fn save_frame(&mut self, frame: &mut InterruptFrame) {
        self.get_current_thread()
            .set_stack_pointer(frame as *const InterruptFrame as *const u64);
    }

    pub fn readfile(&self, entry: FAT32DirectoryEntry, buffer: &mut [u8]) -> usize {
        self.filesystem.lock().read_file(entry, buffer)
    }

    // Syscall helpers
    fn open_object(&mut self, name: &str) {
        let mut split = name.split(":");
        let prefix = split.next().unwrap();

        if prefix == "file" {
            let path = split.next().unwrap();
            let entry = self.filesystem.lock().search_item(path);

            if let Some(entry) = entry {
                let id = self.object_id_allocator.allocate_id();

                self.get_current_thread()
                    .add_object(Arc::new(FileObject::from_entry(entry)), id);

                self.get_current_thread().set_return_value(id);
            } else {
                self.get_current_thread().set_return_value(0);
            }
        } else if prefix == "stdio" {
            let id = self.object_id_allocator.allocate_id();
            self.get_current_thread()
                .add_object(Arc::new(Stdio::new()), id);
            self.get_current_thread().set_return_value(id);
        }
    }

    fn create_thread(&mut self, entry: usize, args: SyscallArgs) {
        let page_ref = self
            .page_allocator
            .lock()
            .allocate_page()
            .expect("Unable to Allocate Page");

        let mut sp = page_ref.get_initial_stack_pointer();

        let mut initial_frame = InterruptFrame::with_kernel_entry(entry as u64);
        initial_frame.set_arg(args[2] as u64);

        sp = sp.push(0 as u64).push(0 as u64);
        let sp = sp.push(initial_frame);

        let top_page: u64 = 0xFFFF_FFFF_FFFF_F000;
        let stack_phys_page = (page_ref.page as u64) & !(0xFFF) & (0xFFFF_FFFF_FFFF);

        let sp = sp.get() as u64 | top_page;

        let stack_pointer = IRQLock::new(sp as *const u64);

        let name;
        unsafe {
            name = String::from(&*(args[1] as *mut String));
        }

        let id = self.thread_id_allocator.allocate_id();

        let mut kernel_table = PageTable::new_kernel();

        kernel_table.map_user_address(top_page, stack_phys_page);

        self.scheduler.add_thread(Thread {
            stack_pointer,
            parent: Some(self.scheduler.current_thread.clone()),
            status: IRQLock::new(ThreadStatus::Ready),
            name,
            id,
            children: IRQLock::new(vec![]),
            objects: IRQLock::new(vec![]),
            kernel_table: IRQLock::new(kernel_table), // Currently all kernel threads have the same mapping
            user_table: IRQLock::new(PageTable::new_unmapped()),
        });

        self.get_current_thread().set_return_value(id);
    }

    fn read(&self, handle: ObjectHandle, buffer: &mut [u8]) {
        let thread = self.get_current_thread();

        let object = thread.get_object(handle);

        if let Some(o) = object {
            let len = o.read(self, buffer);
            thread.set_return_value(len as u64);
        } else {
            // TODO: kill thread
        }
    }

    fn write(&self, handle: ObjectHandle, buffer: &mut [u8]) {
        let thread = self.get_current_thread();

        let object = thread.get_object(handle);

        if let Some(o) = object {
            let len = o.write(self, buffer);
            thread.set_return_value(len as u64);
        } else {
            // TODO: kill thread
        }
    }
}
