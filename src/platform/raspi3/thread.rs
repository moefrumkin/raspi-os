use alloc::rc::Rc;
use alloc::sync::Arc;
use core::arch::asm;

use alloc::vec;

use alloc::vec::Vec;

use alloc::collections::VecDeque;

use alloc::string::String;

use alloc::boxed::Box;

use super::kernel_object::{KernelObject, ObjectHandle};
use crate::aarch64::interrupt::IRQLock;
use crate::aarch64::{cpu, mmu};
use crate::allocator::page_allocator::PAGE_SIZE;
use crate::elf::{ELF64Header, ProgramHeader, ProgramType};
use crate::platform::page_table::PageTable;
use crate::platform::platform_devices::PLATFORM;
use crate::platform::raspi3::exception::InterruptFrame;

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
    pub objects: IRQLock<Vec<(ObjectHandle, Box<dyn KernelObject>)>>, // TODO: find a more efficient way of doing this
    pub kernel_table: IRQLock<PageTable>,
    pub user_table: IRQLock<PageTable>,
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
            objects: IRQLock::new(vec![]),
            kernel_table: IRQLock::new(PageTable::from(mmu::get_kernel_table())),
            user_table: IRQLock::new(PageTable::from(mmu::get_user_table())),
        }
    }

    pub fn return_to(&self) -> ! {
        unsafe {
            let user_table = self.user_table.lock().get_ttbr();

            // See the Armv8-A address translation manual
            asm!("msr ttbr0_el1, {ttbr0}", ttbr0 = in(reg) user_table);
            asm!("dsb ishst");
            //asm!("tlbi alle1");
            asm!("dsb ish", "isb");

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

    pub fn exec(&self, program: &str) {
        let handle = cpu::open_object(program);

        let mut buffer: [u8; 840] = [b'\0'; 840];

        crate::println!("Reading program {}", program);

        let bytes_read = cpu::read_object(handle, &mut buffer);

        crate::println!("Parsing ELF");

        let header = ELF64Header::try_from(&buffer[0..bytes_read]).expect("Error parsing elf");

        let entry_address = header.program_entry_address;

        crate::println!("Header {:#?}", header);

        crate::println!("Entry: {:#x}", entry_address);

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

        for pheader in pheaders
            .iter()
            .filter(|header| header.program_type == ProgramType::Loadable)
        {
            let vaddr = pheader.virtual_address;
            let offset = pheader.offset;

            let file_size = pheader.file_size;
            let memory_size = pheader.memory_size;

            let start_page = vaddr & !(0xFFF);
            let end_page = (vaddr + memory_size) & !(0xFFF);

            let number_of_pages = (start_page - end_page) / (PAGE_SIZE as u64) + 1;

            // Page we are currently reading
            let mut current_page = start_page;

            // Offset in the read buffer
            let mut buffer_offset = offset;

            // Address of page we are writing to
            let mut virtual_address = current_page;

            // Bytes left to read
            let mut to_read = file_size;

            crate::println!("Number of pages: {}", number_of_pages);

            let mut offset_in_page = vaddr & 0xFFF;

            crate::println!("OFfset in page: {}", offset_in_page);

            for _ in 0..number_of_pages {
                crate::println!("Vaddr: {:#x}", virtual_address);
                if !self.user_table.lock().is_addr_mapped(virtual_address) {
                    let page = PLATFORM.allocate_zeroed_page();

                    self.user_table
                        .lock()
                        .map_user_address(virtual_address, page.page as u64);
                    crate::println!("Mapping {:#x} to {:#x}", virtual_address, page.page as u64);
                    // TODO: data and istruction buffer?

                    // TODO currect buffering?
                    let read_length = core::cmp::min(to_read, PAGE_SIZE as u64 - offset_in_page);

                    for i in offset_in_page..offset_in_page + read_length {
                        unsafe {
                            (*(page.page as *mut [u8; PAGE_SIZE]))[i as usize] =
                                buffer[buffer_offset as usize];
                        }

                        buffer_offset += 1
                    }

                    to_read -= read_length;

                    virtual_address += PAGE_SIZE as u64;
                    current_page += PAGE_SIZE as u64;

                    offset_in_page = 0;
                }
            }
        }

        // Could we do this earlier?
        //cpu::close_object(handle);

        let spsr_el1 = 0;

        crate::println!("Entry: {:#x}", entry_address);

        let stack_page = PLATFORM.allocate_zeroed_page();
        let sp = (0x80_000 /*(stack_page.page as usize)*/ + PAGE_SIZE - 8) & (0xFFFF_FFFF_FFFF);

        // TODO: how to choose base stack pointer
        self.user_table
            .lock()
            .map_user_address(0x80_000, stack_page.page as u64);

        unsafe {
            asm!("dsb ishst");
            //asm!("tlbi alle1");
            asm!("dsb ish", "isb");
        }

        unsafe {
            asm!("msr spsr_el1, {0:x}", in (reg) spsr_el1);
            asm!("msr elr_el1, {0:x}", in (reg) entry_address);
            asm!("msr sp_el0, {}", in (reg) sp);
            asm!("eret");
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

        self.current_thread
            .objects
            .lock()
            .retain(|(id, _)| *id != handle);
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

        self.set_current_thread_return(return_value as u64);
    }
}
