#[cfg(not(test))]
global_asm!(include_str!("start.s"));

#[no_mangle]
pub fn main() {
    unsafe {
        core::ptr::write_volatile(0x3f200008 as *mut u32, 0x08);
        core::ptr::write_volatile(0x3f20001c as *mut u32, 0x200000);
    }
}