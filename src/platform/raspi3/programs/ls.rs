use crate::aarch64::cpu;
use crate::elf::ELF64Header;
use crate::println;
use crate::print;

pub extern "C" fn ls(_: usize) {
    println!("Opening object");

    let fixup_handle = cpu::open_object("BIN./EXIT.ELF");

    println!("Opened with handle: {}", fixup_handle);

    let mut buffer: [u8; 8192] = [b'\0'; 8192];

    println!("Reading from file");
    let bytes_read = cpu::read_object(fixup_handle, &mut buffer);

    let header = ELF64Header::try_from(&buffer[0..bytes_read]).expect("Error parsing elf");
    
    println!("{} bytes read", bytes_read);

    println!("{:?}", header);

    cpu::close_object(fixup_handle);

    cpu::exit_thread(0);
}