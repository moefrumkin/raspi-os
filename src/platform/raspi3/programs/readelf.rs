use crate::aarch64::cpu;
use crate::elf::{ELF64Header, ProgramHeader};
use crate::print;
use crate::println;
use alloc::vec;
use alloc::vec::Vec;

pub extern "C" fn readelf(_: usize) {
    println!("Opening object");

    let fixup_handle = cpu::open_object("USERS./MOE./EXIT.ELF");

    println!("Opened with handle: {}", fixup_handle);

    let mut buffer: [u8; 8192] = [b'\0'; 8192];

    println!("Reading from file");
    let bytes_read = cpu::read_object(fixup_handle, &mut buffer);

    println!("{} bytes read", bytes_read);
    println!("Parsing Elf");

    let header = ELF64Header::try_from(&buffer[0..bytes_read]).expect("Error parsing elf");

    let mut pheaders: Vec<ProgramHeader> = vec![];

    let pheader_start = header.program_header_offset;

    println!("Pheader has offset: {}", pheader_start);

    for i in 0..header.program_header_number {
        let pheader_offset = pheader_start + ((header.program_header_entry_size * i) as u64);

        let phdr = unsafe {
            let buffer_offset = buffer.as_ptr().offset(pheader_offset as isize);

            *(buffer_offset as *const ProgramHeader)
        };

        println!("Header: {:?}\n", phdr);

        pheaders.push(phdr);
    }

    println!("{:?}", header);

    cpu::close_object(fixup_handle);

    cpu::exit_thread(0);
}
