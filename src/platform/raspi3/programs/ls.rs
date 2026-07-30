use crate::aarch64::syscall;
use crate::elf::ELF64Header;
use crate::println;

pub extern "C" fn ls(_: usize) {
    println!("Opening object");

    let fixup_handle = syscall::open("file:USERS./MOE./EXIT.ELF");

    println!("Opened with handle: {}", fixup_handle);

    let mut buffer: [u8; 8192] = [b'\0'; 8192];

    println!("Reading from file");
    let bytes_read = syscall::read(fixup_handle, &mut buffer);

    let header = ELF64Header::try_from(&buffer[0..bytes_read]).expect("Error parsing elf");

    println!("{} bytes read", bytes_read);

    println!("{:?}", header);

    syscall::close(fixup_handle);

    syscall::exit(0);
}
