//! Structures and functions to represent and manipulate OS threads

use alloc::sync::Arc;
use core::arch::asm;

use alloc::vec;

use alloc::vec::Vec;

use alloc::string::String;

use super::kernel_object::{KernelObject, ObjectHandle};
use crate::aarch64::interrupt::IRQLock;
use crate::aarch64::{mmu, syscall};
use crate::allocator::page_allocator::PAGE_SIZE;
use crate::elf::{ELF64Header, ProgramHeader, ProgramType};
use crate::platform::page_table::PageTable;
use crate::platform::platform_devices::PLATFORM;
use crate::platform::raspi3::exception::InterruptFrame;

/// The status of a given thread
#[derive(Copy, Clone, Debug)]
pub enum ThreadStatus {
    /// The thread is current running
    Running,
    /// The thread is not running but it unblocked and scheduled to run
    Ready,
    /// The thread is sleeping until a clock time
    Sleeping(u64),
    /// The thread has exited with the given status
    Exited(u64),
    /// The thread is waiting for a given thread to exit
    Joining(ThreadID),
}

/// Each thread is identified with a unique id
pub type ThreadID = u64;

#[derive(Debug)]
// TODO: implement Drop trait
pub struct Thread<'a> {
    /// The thread's stack pointer. This is only valid for a non-running thread
    pub stack_pointer: IRQLock<*const u64>,
    pub parent: Option<Arc<Thread<'a>>>,
    pub status: IRQLock<ThreadStatus>,
    pub name: String,
    pub id: u64,
    pub children: IRQLock<Vec<Arc<Thread<'a>>>>,
    pub objects: IRQLock<Vec<(ObjectHandle, Arc<dyn KernelObject>)>>, // TODO: find a more efficient way of doing this
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

    /// Return control to this thread
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

    /// Set the return value
    /// Unsafe if the stack pointer is not accurate
    /// TODO: for memory safety, shyould this require a mutable ref to self?
    pub fn set_return_value(&self, value: u64) {
        unsafe {
            let frame = &mut *(*self.stack_pointer.lock() as *mut InterruptFrame);
            frame.gp_registers[0] = value;
        }
    }

    pub fn set_stack_pointer(&self, sp: *const u64) {
        *self.stack_pointer.lock() = sp;
    }

    pub fn add_object(&self, object: Arc<dyn KernelObject>, id: ObjectHandle) {
        self.objects.lock().push((id, object));
    }

    pub fn remove_object(&self, handle: ObjectHandle) {
        self.objects.lock().retain(|(id, _)| *id != handle);
    }

    pub fn get_object(&self, handle: ObjectHandle) -> Option<Arc<dyn KernelObject>> {
        let objects = self.objects.lock();

        for i in 0..objects.len() {
            let (id, o) = &objects[i];

            if *id == handle {
                return Some(o.clone());
            }
        }

        return None;
    }

    /// Load and run a user-mode program on this thread
    pub fn exec(&self, program: &str) {
        let handle = syscall::open(program);

        let mut buffer: [u8; 840] = [b'\0'; 840];

        let bytes_read = syscall::read(handle, &mut buffer);

        let header = ELF64Header::try_from(&buffer[0..bytes_read]).expect("Error parsing elf");

        let entry_address = header.program_entry_address;

        let mut pheaders = vec![];

        let pheader_start = header.program_header_offset;

        for i in 0..header.program_header_number {
            let pheader_offset = pheader_start + ((header.program_header_entry_size * i) as u64);

            let phdr = unsafe {
                let buffer_offest = buffer.as_ptr().offset(pheader_offset as isize);

                *(buffer_offest as *const ProgramHeader)
            };

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

            let mut offset_in_page = vaddr & 0xFFF;

            for _ in 0..number_of_pages {
                if !self.user_table.lock().is_addr_mapped(virtual_address) {
                    let page = PLATFORM.allocate_zeroed_page();

                    self.user_table
                        .lock()
                        .map_user_address(virtual_address, page.page as u64);
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
