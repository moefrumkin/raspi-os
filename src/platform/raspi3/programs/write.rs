use crate::aarch64::cpu::{self, close_object, exit_thread, write_object};
use crate::platform::platform_devices::PLATFORM;
use crate::println;

pub extern "C" fn write(_: usize) {
    println!("Running Write.elf");

    PLATFORM.exec("file:USERS./MOE./WRITE.ELF");

    cpu::exit_thread(0);
}
