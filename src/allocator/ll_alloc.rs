use core::mem;
use core::fmt::{Debug, Formatter, Error};
use alloc::alloc::{GlobalAlloc, Layout};
use crate::sync::SpinMutex;

#[derive(Debug)]
pub struct LinkedListAllocator {
    free_list: FreeBlock,
    size: usize
}

unsafe impl GlobalAlloc for SpinMutex<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::expand_to_min(layout);

        let mut allocator = self.lock();

        if let Some(block) = allocator.find(size, align) {
            block as *const FreeBlock as *mut u8
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = LinkedListAllocator::expand_to_min(layout);

        self.lock().free(ptr as usize, size);
    }
}

impl Debug for SpinMutex<LinkedListAllocator> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.lock().fmt(f)
    }
}

impl LinkedListAllocator {
    #[allow(dead_code)]
    pub const fn new() -> Self {
        Self {
            free_list: FreeBlock {size: 0, next: None},
            size: 0
        }
    }

    #[allow(dead_code)]
    pub fn init(&mut self, start: usize, size: usize) {
        if size == 0 {
            panic!("Heap must have non-zero size");
        }

        self.size = size;
        self.free(start, size);

        let block_ptr = start as *mut FreeBlock;

        unsafe {
            let val = FreeBlock::from_components(size, None);
            block_ptr.write_volatile(val);
            //self.free_list = None; 
            //self.size = size;
        }
    }

    //TODO coalesce neighboring blocks
    fn free(&mut self, start: usize, size: usize) {
        if super::align(start, mem::align_of::<FreeBlock>()) != start {
            panic!("Incompatible memory alignment of freed block");
        }

        if size < mem::size_of::<FreeBlock>() {
            panic!("Block too small to free");
        }
        
        //No actual computation, just a cast
        let block_ptr = start as *mut FreeBlock;

        //This is where the fun begins
        unsafe {
            //create a free block at the start location
            let val = FreeBlock::from_components(size, self.free_list.next.take());
            block_ptr.write_volatile(val);

            self.free_list.next = Some(&mut *block_ptr);
        }
    }

    /// finds a free block that satisfies the size and alignment requirements
    /// will divide a block to find a free block
    unsafe fn find(&mut self, size: usize, align: usize) -> Option<&FreeBlock> {
        let mut current = &mut (self.free_list) as *mut FreeBlock;

        while let Some(block) = (*current).next {
            if let Ok((result, next_block)) = Self::try_fit(&mut *block, size, align) {
                //TODO: there should be a more elegant way of expressing this
                (*current).next = next_block;
                return Some(result);
            } else {
                current = block;
            }
        }

        None
    }

    /// Finds an appropriate start within a block for a given alignment and size
    /// If a block is found, it is returned
    /// A block may be allocated from within an existing block. In this case, it creates a sub
    /// block and relinks the remaining blocks appropriately
    /// Returns the freed block and the block to link the previous block to
    unsafe fn try_fit(block: &mut FreeBlock, size: usize, align: usize) -> Result<(&FreeBlock, Option<*mut FreeBlock>), ()> {
        let start = super::align(block.start(), align);
        let end = start + size;

        // TODO: is there a more efficient sequence of calculations here?
        let start_offset = start - block.start();
        let end_offset = block.end() - end;

        if end > block.end() {
            return Err(());
        }

        let next_block: Option<*mut FreeBlock>;

        if start_offset > 0 {
            if start_offset < mem::size_of::<FreeBlock>() {
                return Err(());
            } else {
                block.size = start_offset;
                next_block = Some(block as *mut FreeBlock);
            }
        } else {
            next_block = block.next;
        }

        if end_offset > 0 {
            if end_offset < mem::size_of::<FreeBlock>() {
                return Err(());
            } else {
                *((end) as *mut FreeBlock) = FreeBlock{size: end_offset, next: block.next};
                block.next = Some((end) as *mut FreeBlock);
            }
        }

        Ok((&*(start as *const FreeBlock), next_block))
    }

    /// Expands a layout to the minimum size required to fit a FreeBlock instance 
    fn expand_to_min(layout: Layout) -> (usize, usize) {
        if layout.align() == 0 {
            panic!("Layout must have non zero alignment");
        }

        if layout.align() & (layout.align() << 1) != 0 {
            panic!("Layout must have power of two alignment");
        }
        let layout = layout
            .align_to(mem::align_of::<FreeBlock>())
            //.expect("Unable to fit layout to minimum size")
            .unwrap()
            .pad_to_align();
        
            ( layout.size().max(mem::size_of::<FreeBlock>()), layout.align() )
    }
}

#[derive(Debug)]
struct FreeBlock {
    size: usize,
    next: Option<*mut FreeBlock>
}

impl FreeBlock {
    #[allow(dead_code)]
    const fn new(size: usize) -> Self {
        Self {
            size,
            next: None
        }
    }

    fn from_components(size: usize, next: Option<*mut FreeBlock>) -> Self {
        Self {
            size,
            next
        }
    }

    fn start(&self) -> usize {
        self as *const Self as usize
    }

    fn end(&self) -> usize {
        self.start() + self.size
    }
}

#[cfg(test)]
mod tests {
    use std::alloc::Layout;
    use super::*;
    use crate::sync::SpinMutex;

    static mut HEAP: [u8; 4096] = [0; 4096];

    #[test]
    fn allocate() {
        unsafe {
            let alloc = SpinMutex::new(LinkedListAllocator::new());
            alloc.lock().init(HEAP.as_ptr() as usize, HEAP.len());

            assert_eq!(alloc.alloc(Layout::from_size_align(4096, 1).unwrap()), HEAP.as_mut_ptr());

            alloc.dealloc(HEAP.as_mut_ptr(), Layout::from_size_align(4096, 1).unwrap());

            assert_eq!(alloc.alloc(Layout::from_size_align(65536, 256).unwrap()), std::ptr::null_mut());

            assert_eq!(alloc.alloc(Layout::from_size_align(256, 16).unwrap()), HEAP.as_mut_ptr());
        }
    }

    #[test]
    fn find_start() {
        let block = FreeBlock::from_components(1024, None);

        assert_eq!(LinkedListAllocator::find_start(&block, 56, 8), Ok(&block as *const FreeBlock as usize));
        assert_eq!(LinkedListAllocator::find_start(&block, 2048, 8), Err(()));
    }

    #[test]
    fn expand_to_min() {
        let size = mem::size_of::<FreeBlock>();
        let align = mem::align_of::<FreeBlock>();
        assert_eq!(LinkedListAllocator::expand_to_min(Layout::from_size_align(8, 1).unwrap()), (size, align));
        assert_eq!(LinkedListAllocator::expand_to_min(Layout::from_size_align(160, 4).unwrap()), (160, align));
        assert_eq!(LinkedListAllocator::expand_to_min(Layout::from_size_align(4, 16).unwrap()), (size, 16));
        assert_eq!(LinkedListAllocator::expand_to_min(Layout::from_size_align(160, 32).unwrap()), (160, 32));
        
    }
}
