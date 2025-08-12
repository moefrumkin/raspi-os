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
        let layout = Layout::from_size_align(size, align).expect("Error creating layout");

        let start = 
            Global.allocate(layout);

        if start.is_err() {
            panic!("Unable to allocate");
        }

        let start = start.expect("Error allocating aligned buffer");

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

    fn as_slice(&self) -> &[T] {
        unsafe {
            core::slice::from_raw_parts(self.start as *const T, self.len())
        }
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(self.start, self.len())
        }
    }
}

impl<T> Drop for AlignedBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            Global.deallocate(NonNull::new(self.start as *mut u8).expect("Error freeing aligned buffer"), self.layout);
        }
    }
}

impl<T, Idx> Index<Idx> for AlignedBuffer<T>
where Idx:
core::slice::SliceIndex<[T]> {
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl <T, Idx> IndexMut<Idx> for AlignedBuffer<T>
where
    Idx: core::slice::SliceIndex<[T]> {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}

impl<T> Deref for AlignedBuffer<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
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
            (value as *const T as *const Volatile<T>).as_ref().expect("Error converting pointer to volatile pointer")
        }
    }

    pub fn from_mut_ptr(value: &mut T) -> &mut Self {
        unsafe {
            (value as *mut T as *mut Volatile<T>).as_mut().expect("Error converting pointer to mutable volatile pointer")
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

    pub fn map(&mut self, f: fn(T) -> T) {
        self.set(f(self.get()))
    }

    pub fn map_closure(&mut self, f: &dyn Fn(T) -> T) {
        self.set(f(self.get()))
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
