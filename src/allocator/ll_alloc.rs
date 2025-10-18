use crate::aarch64::{cpu, interrupt};
use crate::sync::SpinMutex;
use alloc::alloc::{GlobalAlloc, Layout};
use core::fmt::{Debug, Error, Formatter};
use core::mem;

#[derive(Debug)]
pub struct LinkedListAllocator {
    /// A linked list of free blocks
    free_list: FreeBlock,

    /// The amont of memory managed by this allocator
    size: usize,

    stats: AllocatorStats,
}

#[derive(Debug, Clone, Copy)]
pub struct AllocatorStats {
    pub free_space: usize,
    pub blocks: usize,
    pub allocs: usize,
    pub frees: usize,
}

impl AllocatorStats {
    pub const fn new() -> Self {
        Self {
            free_space: 0,
            blocks: 0,
            allocs: 0,
            frees: 0,
        }
    }
}

impl Default for AllocatorStats {
    fn default() -> Self {
        Self {
            free_space: 0,
            blocks: 0,
            allocs: 0,
            frees: 0,
        }
    }
}

unsafe impl GlobalAlloc for SpinMutex<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let allocator = &mut self.lock();

        let allocation = allocator
            .allocate(layout)
            .map_or(core::ptr::null_mut(), |b| b as *const FreeBlock as *mut u8);

        allocation
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.lock().free(ptr as usize, layout.size());
    }
}

impl SpinMutex<LinkedListAllocator> {
    pub fn stats(&self) -> AllocatorStats {
        self.lock().stats
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
            // The first block is a sentinel
            free_list: FreeBlock {
                size: 0,
                next: None,
            },
            size: 0,
            stats: AllocatorStats::new(),
        }
    }

    #[allow(dead_code)]
    pub fn init(&mut self, start: usize, size: usize) {
        if size == 0 {
            panic!("Heap must have non-zero size");
        }

        self.size = size;
        self.free(start, size);

        //TODO: find some less awful way to do this
        self.stats.frees -= 1;

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
        self.stats.allocs += 1;
        let (size, align) = Self::expand_to_min(layout);
        // TODO: is it safe to discard next?
        if let Ok((free, _)) = self.free_list.fit_in_block(size, align) {
            Some(free)
        } else {
            None
        }
    }

    //TODO coalesce neighboring blocks
    fn free(&mut self, start: usize, size: usize) {
        self.stats.frees += 1;

        // TODO: should be a single source of truth for expanding blocks
        let size = size.max(mem::size_of::<FreeBlock>());

        if start % mem::align_of::<FreeBlock>() != 0 {
            panic!("Incompatible memory alignment of freed block. Block address: {:x}, needs alignment {}", start, mem::align_of::<FreeBlock>());
        }

        if size < mem::size_of::<FreeBlock>() {
            panic!("Block too small to free: {:#x}, {}", start, size);
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

        (
            layout.size().max(mem::size_of::<FreeBlock>()),
            layout.align(),
        )
    }
}

#[derive(Debug)]
struct FreeBlock {
    size: usize,
    next: Option<&'static mut FreeBlock>,
}

impl FreeBlock {
    #[allow(dead_code)]
    const fn new(size: usize) -> Self {
        Self { size, next: None }
    }

    fn from_components(size: usize, next: Option<&'static mut FreeBlock>) -> Self {
        Self { size, next }
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
    fn fit_in_block(
        &mut self,
        size: usize,
        align: usize,
    ) -> Result<(&mut FreeBlock, Option<&mut FreeBlock>), ()> {
        // The first address in this block with the required alignment
        // Candidate for offset of start of allocated area
        let start_candidate = super::align(self.start(), align);

        let start_offset_candidate = start_candidate - self.start();

        let end_offset_candidate = start_offset_candidate + size;

        let end_candidate = start_candidate + end_offset_candidate;

        // Does the proposed region fit in this block
        let fits = end_candidate <= self.end();

        let trim_start = start_offset_candidate > 0;
        let trim_end = end_candidate < self.end();

        let trim_start_fits = start_offset_candidate >= mem::size_of::<FreeBlock>();
        let trim_end_fits = end_offset_candidate >= mem::size_of::<FreeBlock>();

        let can_allocate =
            fits && !(trim_start && !trim_start_fits) && !(trim_end && !trim_end_fits);

        if !can_allocate {
            if let Some(next) = self.next.take() {
                if let Ok((free_block, next)) = next.fit_in_block(size, align) {
                    self.next = next;
                    return Ok((free_block, Some(self)));
                }
            }

            return Err(());
        }

        if trim_end {
            self.partition(end_offset_candidate);
        }

        if trim_start {
            self.partition(start_offset_candidate);
            let next = self.next.take().expect("Block improperly partitioned");

            if let Ok((freed_block, next_block)) = next.fit_in_block(size, align) {
                self.next = next_block;
                return Ok((freed_block, Some(self)));
            } else {
                return Err(());
            }
        }

        let next = self.next.take();
        return Ok((self, next));
    }

    /// Split the given block into two blocks
    /// Assumes that there is enough space to partition the blocks
    fn partition(&mut self, size: usize) -> Result<(), ()> {
        let new_block: &mut FreeBlock;

        unsafe {
            // As u8 so offset is in bytes
            let ptr = (self as *mut FreeBlock as *mut u8).offset(size as isize);
            new_block = &mut *(ptr as *mut FreeBlock)
        };

        *new_block = FreeBlock {
            size: (*self).size - size,
            next: (*self).next.take(),
        };

        (*self).size = size;
        (*self).next = Some(new_block);

        Ok(())
    }

    fn stats(&self) -> AllocatorStats {
        let mut stats = self
            .next
            .as_ref()
            .map_or_else(AllocatorStats::default, |next| next.stats());

        stats.blocks += 1;
        stats.free_space += self.size;

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::SpinMutex;
    use std::alloc::Layout;

    const HEAP_SIZE: usize = 4096;

    //TODO Check alignment
    #[repr(C, align(4096))]
    struct Heap {
        memory: [u64; HEAP_SIZE],
    }

    static ALIGNED_HEAP: Heap = Heap { memory: [0; 4096] };

    fn initialize_allocator(heap: Heap) -> SpinMutex<LinkedListAllocator> {
        let alloc = SpinMutex::new(LinkedListAllocator::new());
        let ptr = heap.memory.as_ptr() as usize;
        assert_eq!(
            super::super::align(ptr, mem::align_of::<FreeBlock>()),
            ptr,
            "The heap pointer is misaligned"
        );
        alloc.lock().init(ptr, 8 * heap.memory.len());
        return alloc;
    }

    #[test]
    fn test_initialize_allocator() {
        let heap = Heap { memory: [0; 4096] };

        initialize_allocator(heap);
    }

    #[test]
    fn test_allocate_small() {
        let heap = Heap { memory: [0; 4096] };

        let alloc = initialize_allocator(heap);

        let layout = Layout::from_size_align(128, 128).unwrap();

        unsafe {
            let ptr = alloc.alloc(layout);

            assert_ne!(ptr, std::ptr::null_mut(), "Failed to allocate\n");

            assert_eq!(
                ptr as usize % layout.align(),
                0,
                "Allocated block has incorrect alignment. Pointer: {:p}, required alignment is {}",
                ptr,
                layout.align()
            );
        }
    }

    #[test]
    fn test_allocate_large() {
        unsafe {
            let heap = Heap { memory: [0; 4096] };

            let alloc = initialize_allocator(heap);

            let layout = Layout::from_size_align(4096, 1024).unwrap();

            let ptr = alloc.alloc(layout);

            assert_ne!(ptr, std::ptr::null_mut(), "Failed to allocated whole heap");

            assert_eq!(
                ptr as usize % layout.align(),
                0,
                "Allocated block has incorrect allignment. Pointer: {:p}, required alignment is {}",
                ptr,
                layout.align()
            );
        }
    }

    #[test]
    fn test_allocate_many() {
        const ITERATIONS: usize = 64;

        let heap = Heap { memory: [0; 4096] };

        unsafe {
            let alloc = initialize_allocator(heap);

            let layout = Layout::from_size_align(64, 64).unwrap();

            let mut allocations: [*mut u8; ITERATIONS] = [std::ptr::null_mut(); ITERATIONS];

            for i in 0..ITERATIONS {
                let ptr = alloc.alloc(layout);

                assert_ne!(ptr, std::ptr::null_mut(), "Failed to allocate\n");

                assert_eq!(ptr as usize % layout.align(), 0,  "Allocated block has incorrect allignment. Pointer: {:p}, required alignment is {}", ptr, layout.align());

                allocations[i] = ptr;
            }

            let stats = alloc.stats();

            assert_eq!(stats.allocs, ITERATIONS);

            assert_eq!(stats.frees, 0);
        }
    }

    #[test]
    fn test_allocate_and_free() {
        const ITERATIONS: usize = 64;
        let heap = Heap { memory: [0; 4096] };

        unsafe {
            let alloc = initialize_allocator(heap);

            let layout = Layout::from_size_align(64, 64).unwrap();

            let mut allocations: [*mut u8; ITERATIONS] = [std::ptr::null_mut(); ITERATIONS];

            for i in 0..ITERATIONS {
                let ptr = alloc.alloc(layout);

                assert_ne!(ptr, std::ptr::null_mut(), "Failed to allocate\n");

                assert_eq!(ptr as usize % layout.align(), 0,  "Allocated block has incorrect allignment. Pointer: {:p}, required alignment is {}", ptr, layout.align());

                allocations[i] = ptr;
            }

            for i in 0..ITERATIONS {
                alloc.dealloc(allocations[i], layout);
            }

            for i in 0..ITERATIONS {
                let ptr = alloc.alloc(layout);

                assert_ne!(
                    ptr,
                    std::ptr::null_mut(),
                    "Failed to allocate, iteration {}\n",
                    i
                );

                assert_eq!(ptr as usize % layout.align(), 0,  "Allocated block has incorrect allignment. Pointer: {:p}, required alignment is {}", ptr, layout.align());

                allocations[i] = ptr;
            }
        }
    }

    #[test]
    fn test_interleaved() {
        const ITERATIONS: usize = 16;
        const ALLOCS_PER_ITER: usize = 3;

        let heap = Heap { memory: [0; 4096] };

        unsafe {
            let alloc = initialize_allocator(heap);

            let layout = Layout::from_size_align(64, 64).unwrap();

            let mut allocations: [*mut u8; ITERATIONS * ALLOCS_PER_ITER] =
                [std::ptr::null_mut(); ITERATIONS * ALLOCS_PER_ITER];

            for i in 0..ITERATIONS {
                for j in 0..ALLOCS_PER_ITER {
                    let ptr = alloc.alloc(layout);

                    assert_ne!(
                        ptr,
                        std::ptr::null_mut(),
                        "Failed to allocate, {:?}, at iteration {}, allocation {}\n",
                        alloc,
                        i,
                        j
                    );

                    assert_eq!(ptr as usize % layout.align(), 0,  "Allocated block has incorrect allignment. Pointer: {:p}, required alignment is {}", ptr, layout.align());

                    allocations[i * ALLOCS_PER_ITER + j] = ptr;
                }

                if i > 0 {
                    let prev = i - 1;
                    for j in 0..ALLOCS_PER_ITER {
                        alloc.dealloc(allocations[prev * ALLOCS_PER_ITER + j], layout);
                    }
                }
            }

            let last = ITERATIONS - 1;
            for j in 0..ALLOCS_PER_ITER {
                alloc.dealloc(allocations[last * ALLOCS_PER_ITER + j], layout);
            }

            let stats = alloc.stats();

            assert_eq!(stats.allocs, ITERATIONS * ALLOCS_PER_ITER);

            assert_eq!(stats.frees, ITERATIONS * ALLOCS_PER_ITER);
        }
    }

    #[test]
    fn expand_to_min() {
        let size = mem::size_of::<FreeBlock>();
        let align = mem::align_of::<FreeBlock>();
        assert_eq!(
            LinkedListAllocator::expand_to_min(Layout::from_size_align(8, 1).unwrap()),
            (size, align)
        );
        assert_eq!(
            LinkedListAllocator::expand_to_min(Layout::from_size_align(160, 4).unwrap()),
            (160, align)
        );
        assert_eq!(
            LinkedListAllocator::expand_to_min(Layout::from_size_align(4, 16).unwrap()),
            (size, 16)
        );
        assert_eq!(
            LinkedListAllocator::expand_to_min(Layout::from_size_align(160, 32).unwrap()),
            (160, 32)
        );
        assert_eq!(
            LinkedListAllocator::expand_to_min(Layout::from_size_align(4096, 1024).unwrap()),
            (4096, 1024)
        );
    }
}
