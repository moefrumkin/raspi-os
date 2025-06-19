use alloc::alloc::{Global, Allocator};
use core::alloc::Layout;
use core::ptr::NonNull;
use core::ops::{Index, IndexMut};

struct AlignedBuffer<T> {
    start: *mut T,
    layout: Layout
}

impl<T> AlignedBuffer<T> {
    pub fn with_length_align(length: usize, align: usize) -> Self {
        let layout = Layout::from_size_align(length, align).unwrap();

        let start = 
            Global.allocate(layout);

        if start.is_err() {
            panic!("Unable to allocated");
        }

        let start = start.unwrap();

        Self {
            start: start.as_mut_ptr() as *mut T,
            layout
        }
    }

    pub fn len(&self) -> usize {
        self.layout.size()
    }
}

impl<T> Drop for AlignedBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            Global.deallocate(NonNull::new(self.start as *mut u8).unwrap(), self.layout);
        }
    }
}

impl<T> Index<usize> for AlignedBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        if index >= self.len() {
            panic!("Out of bounds error");
        }

        unsafe {
            self.start.offset(index as isize).as_ref().unwrap()
        }
    }
}

impl <T> IndexMut<usize> for AlignedBuffer<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        if index >= self.len() {
            panic!("Out of bounds error");
        }

        unsafe {
            self.start.offset(index as isize).as_mut().unwrap()
        }
    }
}
