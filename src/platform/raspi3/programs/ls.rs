use crate::aarch64::cpu;
use crate::println;

pub extern "C" fn ls(_: usize) {
    println!("Opening object");

    let fixup_handle = cpu::open_object("/FIXUP.DAT");

    println!("Opened with handle: {}", fixup_handle);

    let mut buffer: [char; 1024] = ['\0'; 1024];

    println!("Reading from file");
    cpu::read_object(fixup_handle, &mut buffer);

    cpu::close_object(fixup_handle);

    cpu::exit_thread(0);
}