use core::ptr;

global_asm!(include_str!("start.s"));

/// QEMU start function
/// Writes "Booting on qemu" to UART
#[no_mangle]
pub extern "C" fn start() {
    const UART0: *mut u8 = 0x0900_0000 as *mut u8;
    let out_str = b"Booting on qemu\n";
    for byte in out_str {
        unsafe {
            ptr::write_volatile(UART0, *byte);
        }
    }
}