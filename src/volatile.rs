use alloc::alloc::{Global, Allocator};
use core::alloc::Layout;
use core::ptr::NonNull;
use core::ops::{Index, IndexMut, Deref, DerefMut};

pub struct AlignedBuffer<T> {
    start: *mut T,
    layout: Layout, // TODO: we really shouldn't need to save the layout
    elements: usize
}

impl<T> AlignedBuffer<T> {
    pub fn with_length_align(length: usize, align: usize) -> Self {
        let size = length * Self::element_size();
        let layout = Layout::from_size_align(size, align).unwrap();

        let start = 
            Global.allocate(layout);

        if start.is_err() {
            panic!("Unable to allocated");
        }

        let start = start.unwrap();

        Self {
            start: start.as_mut_ptr() as *mut T,
            layout,
            elements: length
        }
    }

    fn element_size() -> usize {
        size_of::<T>()
    }

    pub fn len(&self) -> usize {
        self.elements
    }

    // TODO: what should the ownership implications of this be?
    pub fn as_ptr(&self) -> *const T {
        self.start
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

impl<T> Deref for AlignedBuffer<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe {
            core::slice::from_raw_parts(self.start, self.len())
        }
    }
}

impl<T> DerefMut for AlignedBuffer<T> {
    fn deref_mut(&mut self) -> &mut[T] {
        unsafe {
            core::slice::from_raw_parts_mut(self.start, self.len())
        }
    }
}

// TODO define ownership semantics
#[repr(transparent)]
pub struct Volatile<T> {
    value: T
}

impl<T> Volatile<T> {
    // TODO: should this use a conversion trait?
    pub fn from_ptr(value: &T) -> &Self {
        unsafe {
            (value as *const T as *const Volatile<T>).as_ref().unwrap()
        }
    }

    pub fn from_mut_ptr(value: &mut T) -> &mut Self {
        unsafe {
            (value as *mut T as *mut Volatile<T>).as_mut().unwrap()
        }
    }
}

// TODO: implement dereferencing?
impl <T: Copy> Volatile<T> {
    pub fn get(&self) -> T {
        unsafe {
            core::ptr::read_volatile(&self.value)
        }
    }

    pub fn set(&mut self, value: T) {
        unsafe {
            core::ptr::write_volatile(&mut self.value, value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volatile()  {
        let mut num = 9;

        let ptr = &mut num;

        let volatile_ptr = Volatile::from_mut_ptr(ptr);

        assert_eq!(volatile_ptr.get(), 9);

        volatile_ptr.set(16);

        assert_eq!(volatile_ptr.get(), 16);

        assert_eq!(num, 16);
    }
}
