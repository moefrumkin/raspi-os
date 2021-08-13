use super::uart::UARTController;
use crate::ALLOCATOR;
use alloc::vec;
use alloc::vec::Vec;

#[cfg(not(test))]
global_asm!(include_str!("start.s"));

extern "C" {
    static HEAP_START: usize;
}

/// QEMU start function
/// Writes "Booting on qemu" to UART
#[naked]
#[no_mangle]
pub extern "C" fn main() {
    let uart = UARTController::new(0x0900_0000);

    uart.writeln("UART Connection Initialized");

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, 100);
    }

    uart.writeln("Allocator Initialized");

    let vec: Vec<u8> = vec![49, 50, 51];

    for n in vec {
        uart.putc(n as char);
    }
}
