use crate::aarch64::syscall;
use crate::platform::platform_devices::PLATFORM;
use crate::println;

pub extern "C" fn write(_: usize) {
    println!("Running Write.elf");

    PLATFORM.exec("file:USERS./MOE./WRITE.ELF");

    syscall::exit(0);
}
