use core::ptr;

#[cfg(not(test))]
//global_asm!(include_str!("start.s"));

/// QEMU start function
/// Writes "Booting on qemu" to UART
#[naked]
#[no_mangle]
pub extern "C" fn _start() {
    unsafe {
        asm!("ldr x30, =LD_STACK_PTR", "mov sp, x30");
    }

    const UART0: *mut u8 = 0x0900_0000 as *mut u8;
    let out_str = b"Booting on qemu\n";
    for byte in out_str {
        unsafe {
            ptr::write_volatile(UART0, *byte);
        }
    }
}
