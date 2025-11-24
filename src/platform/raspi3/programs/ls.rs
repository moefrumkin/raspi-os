use crate::aarch64::cpu;
use crate::println;

pub extern "C" fn ls(_: usize) {
    println!("Opening object");

    let handle = cpu::open_object("hello");

    println!("Opened with handle: {}", handle);

    cpu::exit_thread(0);
}