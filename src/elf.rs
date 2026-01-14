#[repr(C)]
#[derive(Debug)]
pub struct ELF64Header {
    elf_identification: ELFIdentification,
    object_file_type: ObjectFileType,
    e_machine: u16,
    e_version: u32,
    pub program_entry_address: u64, // Address to first transfer execution to
    pub program_header_offset: u64,
    section_header_offset: u64,
    e_flags: u32,
    pub elf_header_size: u16,
    pub program_header_entry_size: u16,
    pub program_header_number: u16,
    section_header_entry_size: u16,
    section_header_entry_num: u16,
    string_table_entry_number: u16,
}

#[repr(C)]
#[derive(Debug)]
pub struct ELFIdentification {
    magic_number: [u8; 4],
    file_class: ELFFileClass,
    data_encoding: u8,
    file_version: u8,
    abi_identification: u8,
    abi_version: u8,
    padding: [u8; 7],
}

#[repr(u16)]
#[derive(Debug)]
enum ObjectFileType {
    None = 0x0,
    RelocatableFile = 0x1,
    ExecutableFile = 0x2,
    SharedObjectFile = 0x3,
    CoreFile = 0x4,
}

#[repr(u8)]
#[derive(Debug)]
pub enum ELFFileClass {
    Invalid = 0x0,
    Class32 = 0x1,
    Class64 = 0x2,
}

impl ELFIdentification {
    const MAGIC_NUMBER: [u8; 4] = [0x7f, b'E', b'L', b'F'];
}

// TODO: Could we implement this all as transmutations with checks? See the file system
impl TryFrom<&[u8]> for ELF64Header {
    type Error = &'static str;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() < core::mem::size_of::<ELF64Header>() {
            return Err("File not large enough to contain header");
        }

        let elf_identification: ELFIdentification = buffer[0..16].try_into()?;

        let object_file_type: ObjectFileType =
            u16::from_le_bytes(buffer[16..18].try_into().expect("Uhh")).try_into()?;

        Ok(Self {
            elf_identification,
            object_file_type,
            e_machine: u16::from_le_bytes(buffer[18..20].try_into().unwrap()),
            e_version: u32::from_le_bytes(buffer[20..24].try_into().unwrap()),
            program_entry_address: u64::from_le_bytes(buffer[24..32].try_into().unwrap()),
            program_header_offset: u64::from_le_bytes(buffer[32..40].try_into().unwrap()),
            section_header_offset: u64::from_le_bytes(buffer[40..48].try_into().unwrap()),
            e_flags: u32::from_le_bytes(buffer[48..52].try_into().unwrap()),
            elf_header_size: u16::from_le_bytes(buffer[52..54].try_into().unwrap()),
            program_header_entry_size: u16::from_le_bytes(buffer[54..56].try_into().unwrap()),
            program_header_number: u16::from_le_bytes(buffer[56..58].try_into().unwrap()),
            section_header_entry_size: u16::from_le_bytes(buffer[58..60].try_into().unwrap()),
            section_header_entry_num: u16::from_le_bytes(buffer[60..62].try_into().unwrap()),
            string_table_entry_number: u16::from_le_bytes(buffer[62..64].try_into().unwrap()),
        })
    }
}

impl TryFrom<&[u8]> for ELFIdentification {
    type Error = &'static str;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() < core::mem::size_of::<ELFIdentification>() {
            return Err("File not larger enough to contain identification");
        }

        if buffer[0..4] != Self::MAGIC_NUMBER {
            return Err("Magic number not found");
        }

        let file_class: ELFFileClass = buffer[4].try_into()?;

        Ok(Self {
            magic_number: Self::MAGIC_NUMBER,
            file_class: file_class,
            data_encoding: buffer[5],
            file_version: buffer[6],
            abi_identification: buffer[7],
            abi_version: buffer[8],
            padding: [0; 7],
        })
    }
}

impl TryFrom<u16> for ObjectFileType {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(ObjectFileType::None),
            0x1 => Ok(ObjectFileType::RelocatableFile),
            0x2 => Ok(ObjectFileType::ExecutableFile),
            0x3 => Ok(ObjectFileType::SharedObjectFile),
            0x4 => Ok(ObjectFileType::CoreFile),
            _ => Err("Invalid object file type"),
        }
    }
}

impl TryFrom<u8> for ELFFileClass {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(ELFFileClass::Invalid),
            0x1 => Ok(ELFFileClass::Class32),
            0x2 => Ok(ELFFileClass::Class64),
            _ => Err("Invalid elf file class"),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProgramHeader {
    pub program_type: ProgramType,
    flags: u32,
    pub offset: u64,
    pub virtual_address: u64,
    physical_address: u64,
    pub file_size: u64,
    pub memory_size: u64,
    alignment: u64,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramType {
    Ignored = 0x0,
    Loadable = 0x1,
    Dynamic = 0x2,
    Interpreter = 0x3,
    Note = 0x4,
    Shlib = 0x5,
    PHeader = 0x6,
    ThreadLocalStorage = 0x7,
}
