pub mod ll_alloc;

pub fn align(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1) 
}