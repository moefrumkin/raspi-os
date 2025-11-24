use crate::aarch64::cpu;
use crate::println;

pub extern "C" fn ls(_: usize) {
    println!("Opening object");

    let handle = cpu::open_object("/");

    println!("Opened with handle: {}", handle);

    let handle = cpu::open_object("/FIXUP.DAT");

    println!("Opened with handle: {}", handle);

    cpu::exit_thread(0);
}