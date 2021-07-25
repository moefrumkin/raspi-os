use crate::aarch64::cpu;

use super::gpu;
use super::gpio;
use super::timer;

#[no_mangle]
pub extern "C" fn _start() {
    if cpu::core_id() == 0 {
        let bss_start: usize;
        let bss_end: usize;

        unsafe{
            asm! {
                "ldr {}, =BSS_START",
                "ldr {}, =BSS_END",
                out(reg) bss_start,
                out(reg) bss_end
            }
        }

        let mut addr = bss_start;
        while(addr < bss_end) {
            unsafe {
                core::ptr::write_volatile(addr as *mut u64, 0);
            }
            addr += 8;
        }

        unsafe {
            asm!(
                "ldr x30, =LD_STACK_PTR",
                "mov sp, x30"
            )
        }
    }

    loop {}
}
