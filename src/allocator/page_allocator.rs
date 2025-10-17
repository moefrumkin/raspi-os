use core::slice;

use crate::{allocator::align, utils::bit_array::BitArray};

pub const PAGE_SIZE: usize = 65536;

pub type Page = [u8; PAGE_SIZE];

#[repr(C)]
pub struct PageAllocator<'a> {
    free_list: &'a mut [BitArray<usize>],
    pages: &'a mut [Page],
}

pub struct PageRef {
    pub page: *mut Page,
    pub page_number: usize,
}

impl<'a> PageAllocator<'a> {
    pub fn allocate_page(&mut self) -> Option<PageRef> {
        for i in 0..self.free_list.len() {
            // TODO: check edge cases to make sure we don't go over the number of pages allocated
            for j in 0..64 {
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

        let page_start = align(start + bytes_in_free_list, 0x10_000);

        let number_of_pages = ((start + bytes) - page_start) / PAGE_SIZE;

        unsafe {
            let free_list =
                slice::from_raw_parts_mut(start as *mut BitArray<usize>, number_of_blocks / 64);

            let pages = slice::from_raw_parts_mut(page_start as *mut Page, number_of_pages);

            Self { free_list, pages }
        }
    }
}
