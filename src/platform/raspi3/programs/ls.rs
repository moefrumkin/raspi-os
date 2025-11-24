use crate::aarch64::cpu;
use crate::println;

pub extern "C" fn ls(_: usize) {
    println!("Opening object");

    let root_handle = cpu::open_object("/");

    println!("Opened with handle: {}", root_handle);

    let fixup_handle = cpu::open_object("/FIXUP.DAT");

    println!("Opened with handle: {}", fixup_handle);

    cpu::close_object(root_handle);
    cpu::close_object(fixup_handle);

    cpu::exit_thread(0);
}