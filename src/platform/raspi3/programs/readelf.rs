use crate::aarch64::cpu;
use crate::elf::{ELF64Header, ProgramHeader};
use crate::platform::platform_devices::PLATFORM;
use crate::print;
use crate::println;
use alloc::vec;
use alloc::vec::Vec;

pub extern "C" fn readelf(_: usize) {
    println!("Running Exit.elf");

    PLATFORM.exec("USERS./MOE./EXIT.ELF");

    cpu::exit_thread(0);
}
