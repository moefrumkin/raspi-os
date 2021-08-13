use core::mem;
use alloc::alloc::{GlobalAlloc, Layout};
use crate::sync::SpinMutex;

pub struct LinkedListAllocator {
    free_list: FreeBlock
}

unsafe impl GlobalAlloc for SpinMutex<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::expand_to_min(layout);

        let mut allocator = self.lock();

        if let Some((region, start)) = allocator.find(size, align) {
            let end = start + size;
            let trim = region.end() - end;

            if trim > 0 {
                allocator.free(region.end(), trim);
            }
            start as *mut u8
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = LinkedListAllocator::expand_to_min(layout);

        self.lock().free(ptr as usize, size);
    }
}

impl LinkedListAllocator {
    #[allow(dead_code)]
    pub const fn new() -> Self {
        Self {
            free_list: FreeBlock::new(0)
        }
    }

    #[allow(dead_code)]
    pub fn init(&mut self, start: usize, size: usize) {
        if size == 0 {
            panic!("Head must have non-zero size");
        }
        self.free(start, size);
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
            block_ptr.write(FreeBlock::from_components(size, self.free_list.next.take()));

            //update free list
            self.free_list.next = Some(&mut *block_ptr);
        }
    }

    /// finds a free block that satisfies the size and alignment requirements
    fn find(&mut self, size: usize, align: usize) -> Option<(&'static mut FreeBlock, usize)> {
        let mut current = &mut self.free_list;

        while let Some(ref mut block) = current.next {
            if let Ok(start) = Self::find_start(block, size, align) {
                let next = block.next.take();
                let result = Some((
                    current.next.take().expect("The current block does not point to another block"),
                    start
                ));
                current.next = next;
                return result;
            } else {
                current = current.next.as_mut().expect("Unable to mutably access next block");
            }
        }

        None
    }

    /// Finds an appropriate start within a block for a given alignment and size
    fn find_start(block: &FreeBlock, size: usize, align: usize) -> Result<usize, ()> {
        let start = super::align(block.start(), align);
        let end = start + size;

        if end > block.end() {
            return Err(());
        }

        let trim = block.end() - end;

        if trim > 0 && trim < mem::size_of::<FreeBlock>() {
            return Err(());
        }

        Ok(start)
    }

    /// Expands a layout to the minimum size required to fit a FreeBlock instance 
    fn expand_to_min(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<FreeBlock>())
            .expect("Unable to fit layout to minimum size")
            .pad_to_align();
        
            ( layout.size().max(mem::size_of::<FreeBlock>()), layout.align() )
    }
}

struct FreeBlock {
    size: usize,
    next: Option<&'static mut FreeBlock>
}

impl FreeBlock {
    #[allow(dead_code)]
    const fn new(size: usize) -> Self {
        Self {
            size,
            next: None
        }
    }

    fn from_components(size: usize, next: Option<&'static mut FreeBlock>) -> Self {
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