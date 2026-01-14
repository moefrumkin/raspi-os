use core::slice;

use crate::{allocator::align, utils::bit_array::BitArray};

pub const PAGE_SIZE: usize = 4096;

pub type Page = [u8; PAGE_SIZE];

#[repr(C)]
#[derive(Debug)]
pub struct PageAllocator<'a> {
    free_list: &'a mut [BitArray<usize>],
    pages: &'a mut [Page],
}

/// A reference to an allocated page
#[derive(Debug)]
pub struct PageRef {
    pub page: *mut Page,
    pub page_number: usize,
}

/// A stack pointer in an allocated page
pub struct StackPointer {
    sp: *mut u8,
}

impl<'a> PageAllocator<'a> {
    pub fn allocate_page(&mut self) -> Option<PageRef> {
        //TODO: skipping first page now because of possible stack underflow
        for i in 1..self.free_list.len() {
            // TODO: check edge cases to make sure we don't go over the number of pages allocated
            for j in 0..4 {
                // TODO n * j is hacky to prevent overflow
                let j = 16 * j;
                let is_allocated = self.free_list[i].get_bit(j);

                if is_allocated == 0 {
                    let block_number = 64 * i + j;
                    self.free_list[i] = self.free_list[i].set_bit(j, 1);
                    return Some(PageRef {
                        page: &mut self.pages[block_number] as *mut Page,
                        page_number: 64 * i + j,
                    });
                }
            }
        }

        return None;
    }

    pub fn free_page(&mut self, page: &PageRef) {
        let list_block = page.page_number / 64;
        let block_offset = page.page_number % 64;

        self.free_list[list_block] = self.free_list[list_block].set_bit(block_offset, 0);
    }

    pub const fn with_start_and_length(start: usize, bytes: usize) -> Self {
        let number_of_blocks = bytes / (PAGE_SIZE + 1);
        let bytes_in_free_list = number_of_blocks / 8;

        let page_start = align(start + bytes_in_free_list, 0x1000);

        let number_of_pages = ((start + bytes) - page_start) / PAGE_SIZE;

        unsafe {
            let free_list =
                slice::from_raw_parts_mut(start as *mut BitArray<usize>, number_of_blocks / 64);

            let pages = slice::from_raw_parts_mut(page_start as *mut Page, number_of_pages);

            Self { free_list, pages }
        }
    }
}

impl PageRef {
    /// Get the stack pointer that starts at the top of this page
    pub fn get_initial_stack_pointer(&self) -> StackPointer {
        // TODO: where to put unsafeness?
        StackPointer::from(unsafe { (self.page as *mut u8).offset(PAGE_SIZE as isize) })
    }
}

impl StackPointer {
    pub fn from(sp: *mut u8) -> Self {
        Self { sp }
    }

    /// Pushes a value to the stack and returns the new stack pointer
    pub fn push<T>(&mut self, value: T) -> Self {
        let ptr = unsafe { (self.sp as *mut T).offset(-1) };

        unsafe { *ptr = value };

        Self::from(ptr as *mut u8)
    }

    pub fn get(&self) -> *const u64 {
        self.sp as *const u64
    }
}
