pub mod ll_alloc;

pub fn align(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1) 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align() {
        assert_eq!(align(0x1, 1), 0x1);
        assert_eq!(align(0x100, 16), 0x100);
        assert_eq!(align(0x1, 8), 0x8);
        assert_eq!(align(0x9, 8), 0x10);
    }
}