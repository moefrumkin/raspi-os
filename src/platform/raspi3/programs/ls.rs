use crate::aarch64::cpu;
use crate::println;
use crate::print;

pub extern "C" fn ls(_: usize) {
    println!("Opening object");

    let fixup_handle = cpu::open_object("/LOST.TXT");

    println!("Opened with handle: {}", fixup_handle);

    let mut buffer: [u8; 8192] = [b'\0'; 8192];

    println!("Reading from file");
    let bytes_read = cpu::read_object(fixup_handle, &mut buffer);

    for i in 0..512 {
        print!("{}", buffer[i] as char);
    }
    print!("\n");

    println!("{} bytes read", bytes_read);

    cpu::close_object(fixup_handle);

    cpu::exit_thread(0);
}