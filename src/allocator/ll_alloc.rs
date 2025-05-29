use core::mem;
use core::fmt::{Debug, Formatter, Error};
use alloc::alloc::{GlobalAlloc, Layout};
use crate::sync::SpinMutex;

#[derive(Debug)]
pub struct LinkedListAllocator {
    free_list: FreeBlock,
    size: usize
}

#[derive(Debug)]
pub struct AllocatorStats {
    pub free_space: usize,
    pub blocks: usize
}

impl Default for AllocatorStats {
    fn default() -> Self {
        Self {
            free_space: 0,
            blocks: 0
        }
    }
}

unsafe impl GlobalAlloc for SpinMutex<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let allocator = &mut self.lock();

        allocator.allocate(layout)
            .map_or(core::ptr::null_mut(), |b| b as *const FreeBlock as *mut u8)

    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.lock().free(ptr as usize, layout.size());
    }
}

impl SpinMutex<LinkedListAllocator> {
    pub fn stats(&self) -> AllocatorStats {
        let mut stats = AllocatorStats {
            free_space: 0,
            blocks: 0
        };

        let first_block = &self.lock().free_list;          

        return first_block.next.as_ref().map_or_else(AllocatorStats::default, |b| b.stats());
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

    // Return the smallest block larger than the size and of the correct alignment
    fn allocate(&mut self, layout: Layout) -> Option<&mut FreeBlock> {
        let (size, align) = Self::expand_to_min(layout);
        if let Ok((free, next)) = self.free_list.fit_in_block(size, align) {
            Some(free)
        } else {
            None
        }
    }


    //TODO coalesce neighboring blocks
    fn free(&mut self, start: usize, size: usize) {
        if start % mem::align_of::<FreeBlock>() != 0 {
            panic!("Incompatible memory alignment of freed block. Block address: {:x}, needs alignment {}", start, mem::align_of::<FreeBlock>());
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

   /// Finds an appropriate start within a block for a given alignment and size
    /// If a block is found, it is returned
    /// A block may be allocated from within an existing block. In this case, it creates a sub
    /// block and relinks the remaining blocks appropriately
    /// Returns the freed block and boolean whether the caller needs to relink the blocks
    fn fit_in_block(&mut self, size: usize, align: usize) -> Result<(&mut FreeBlock, Option<&mut FreeBlock>), ()> {
        let start = super::align(self.start(), align);
        let end = start + size;

        // TODO: is there a more efficient sequence of calculations here?
        let start_offset = start - self.start();

        if start_offset > 0 {
            if start_offset < mem::size_of::<FreeBlock>() {
                return Err(());
            } else {
                self.partition(start_offset).expect("Failed to partition block");
                let next = self.next.take().expect("Block improperly partitioned");

                if let Ok((freed_block, next_block)) = next.fit_in_block(size, align) {
                    self.next = next_block;
                    return Ok((freed_block, Some(self)));
                } else {
                    return Err(());
                }
            }
        }

        if end > self.end() {
            if let Some(next) = self.next.take() {
                if let Ok((free_block, next)) = next.fit_in_block(size, align) {
                    self.next = next;
                    return Ok((free_block, Some(self)));
                }
            }

            return Err(());
        }

        let end_offset = self.end() - end;

        if end_offset > 0 {
            if end_offset < mem::size_of::<FreeBlock>() {
                return Err(());
            } else {
                self.partition(size + start_offset).expect("Failed to partition block");
                let next = self.next.take();
                return Ok((self, next));
            }
        }

        Err(())
    }

    /// Split the given block into two blocks
    /// Assumes that there is enough space to partition the blocks
    fn partition(&mut self, size: usize) -> Result<(), ()> {
        let new_block: &mut FreeBlock = unsafe {
            &mut *(self as *mut FreeBlock).offset(size as isize)
        };

        *new_block = FreeBlock {
            size: (*self).size - size,
            next: (*self).next.take()
        };

        (*self).size = size;
        (*self).next = Some(new_block);

        Ok(())
    }

    fn stats(&self) -> AllocatorStats {
        let mut stats = self.next.as_ref().map_or_else(AllocatorStats::default, |next| next.stats());

        stats.blocks += 1;
        stats.free_space += self.size;

        stats
    }
} 



#[cfg(test)]
mod tests {
    use std::alloc::Layout;
    use super::*;
    use crate::sync::SpinMutex;

    //TODO Check alignment
    #[repr(align(16))]
    struct Heap {
        memory: [u8; 4096]
    }

    static ALIGNED_HEAP: Heap = Heap{
        memory: [0; 4096]
    };

    static mut HEAP: [u8; 4096] = ALIGNED_HEAP.memory;

    fn initialize_allocator() -> SpinMutex<LinkedListAllocator> {
        let alloc = SpinMutex::new(LinkedListAllocator::new());
        unsafe {
            let ptr = HEAP.as_ptr() as usize;
            assert_eq!(super::super::align(ptr, mem::align_of::<FreeBlock>()), ptr, "The heap pointer is misaligned");
            alloc.lock().init(HEAP.as_ptr() as usize, HEAP.len());
        }
        return alloc
    }

    #[test]
    fn test_initialize_allocator() {
        initialize_allocator();
    }

    #[test]
    fn allocate() {
        unsafe {
            let alloc = SpinMutex::new(LinkedListAllocator::new());
            alloc.lock().init(HEAP.as_ptr() as usize, HEAP.len());

            let layout = Layout::from_size_align(4096, 1024).unwrap();

            let ptr = alloc.alloc(layout);

            assert_ne!(ptr, std::ptr::null_mut(), "Failed to allocated whole heap");

            assert_eq!(ptr as usize % layout.align(), 0,  "Allocated block has incorrect allignment. Pointer: {:p}, required alignment is {}", ptr, layout.align());

            let stats = alloc.stats();

            assert_eq!(stats.free_space, 0, "After allocation, expected 0 bytes left, actual is {}", stats.free_space);
        }
    }

    #[test]
    fn expand_to_min() {
        let size = mem::size_of::<FreeBlock>();
        let align = mem::align_of::<FreeBlock>();
        assert_eq!(LinkedListAllocator::expand_to_min(Layout::from_size_align(8, 1).unwrap()), (size, align));
        assert_eq!(LinkedListAllocator::expand_to_min(Layout::from_size_align(160, 4).unwrap()), (160, align));
        assert_eq!(LinkedListAllocator::expand_to_min(Layout::from_size_align(4, 16).unwrap()), (size, 16));
        assert_eq!(LinkedListAllocator::expand_to_min(Layout::from_size_align(160, 32).unwrap()), (160, 32));
        assert_eq!(LinkedListAllocator::expand_to_min(Layout::from_size_align(4096, 1024).unwrap()), (4096, 1024));
    }
}
