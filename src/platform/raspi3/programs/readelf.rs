use crate::aarch64::syscall;
use crate::platform::platform_devices::PLATFORM;
use crate::println;

pub extern "C" fn readelf(_: usize) {
    println!("Running Exit.elf");

    PLATFORM.exec("file:USERS./MOE./EXIT.ELF");

    syscall::exit(0);
}
