use super::emmc::EMMCController;
use core::fmt;
use crate::bitfield;

#[repr(transparent)]
pub struct Sector {
    pub values: [u8; Self::SECTOR_SIZE]
}

#[repr(C)]
#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct BootSector {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: [u8; 2],
    sectors_per_cluster: u8,
    reserved_sectors: [u8; 2],
    number_of_fats: u8,
    root_entry_count: [u8; 2],
    total_sectors16: [u8; 2],
    media: u8,
    fat_size: [u8; 2],
    sectors_per_track: [u8; 2],
    number_of_heads: [u8; 2],
    hidden_sectors: [u8; 4],
    total_sectors32: [u8; 4],
    // FAT32 specific fields
    sectors_per_fat: [u8; 4],
    ext_flags: [u8; 2],
    version: [u8; 2],
    root_cluster: [u8; 4],
    fs_info_sector: [u8; 2],
    boot_record_copy: [u8; 2],
    res0: [u8; 12],
    drive_number: u8,
    res1: u8,
    boot_signature: u8,
    volume_serial_number: [u8; 4],
    volume_label: [u8; 11],
    file_system_type: [u8; 8],
    res2: [u8; 420],
    signature_word: [u8; 2],
}

pub struct FAT32Config {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub number_of_fats: u8,
    pub root_entry_count: u16,
    pub total_sectors: u32,
    pub sectors_per_fat: u32,
    pub root_cluster: u32,
    pub fs_info_sector: u16
}

impl fmt::Display for FAT32Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n\
            \t-bytes per sector: {} \n\
            \t-sectors per cluster: {} \n\
            \t-reserved sectors: {}\n\
            \t-number of fats: {}\n\
            \t-root entry count: {}\n\
            \t-total sectors: {}\n\
            \t-sectors per fat: {}\n\
            \t-root cluster: {}\n\
            \t-fs info sector: {}",
            self.bytes_per_sector,
            self.sectors_per_cluster,
            self.reserved_sectors,
            self.number_of_fats,
            self.root_entry_count,
            self.total_sectors,
            self.sectors_per_fat,
            self.root_cluster,
            self.fs_info_sector
        )
    }
}

impl BootSector {
    pub fn try_from_sector(sector: &Sector) -> Result<Self, ()> {
        let candidate_boot_sector: BootSector = unsafe {
            // TODO: is it possible to avoid a copy here?
            *(sector as *const Sector as *const BootSector)
        };

        if candidate_boot_sector.signature_word[0] != 0x55 { return Err(()); }
        if candidate_boot_sector.signature_word[1] != 0xAA { return Err(()); }

        let bytes_per_sector = candidate_boot_sector.get_bytes_per_sector();

        if !(bytes_per_sector == 512
            || bytes_per_sector == 1024
            || bytes_per_sector == 2048
            || bytes_per_sector == 4096
        ) { return Err(()); }

        let sectors_per_cluster = candidate_boot_sector.get_sectors_per_cluster();

        // Bit hack to check if number is a power of 2
        if sectors_per_cluster & (sectors_per_cluster >> 1) != 0 {
            return Err(());
        }

        if candidate_boot_sector.get_reserved_sectors() == 0 {
            return Err(());
        }

        let number_of_fats = candidate_boot_sector.get_number_of_fats();

        if !(number_of_fats == 1 || number_of_fats == 2) {
            return Err(());
        }

        // For Fat 32 only
        if candidate_boot_sector.get_root_entry_count() != 0 {
            return Err(());
        }

        // For Fat 32 only
        if candidate_boot_sector.get_total_sectors16() != 0 { return Err(()); }

        // TODO: check media value

        // TODO: For Fat 32 only check fat_size

        // For Fat 32 only
        if candidate_boot_sector.get_total_sectors32() == 0 { return Err(()); }

        return Ok(candidate_boot_sector);
    }
    pub fn get_oem_name(&self) -> Result<&str, core::str::Utf8Error> {
        str::from_utf8(&self.oem_name)
    }

    pub fn get_bytes_per_sector(&self) -> u16 {
        u16::from_le_bytes(self.bytes_per_sector)
    }

    pub fn get_sectors_per_cluster(&self) -> u8 {
        self.sectors_per_cluster
    }

    pub fn get_reserved_sectors(&self) -> u16 {
        u16::from_le_bytes(self.reserved_sectors)
    }

    pub fn get_number_of_fats(&self) -> u8 {
        self.number_of_fats
    }

    pub fn get_root_entry_count(&self) -> u16 {
        u16::from_le_bytes(self.root_entry_count)
    }

    pub fn get_total_sectors16(&self) -> u16 {
        u16::from_le_bytes(self.total_sectors16)
    }

    // TODO: validate
    pub fn get_media(&self) -> u8 {
        self.media
    }

    pub fn fat_size(&self) -> u16 {
        u16::from_le_bytes(self.fat_size)
    }

    pub fn get_total_sectors32(&self) -> u32 {
        u32::from_le_bytes(self.total_sectors32)
    }

    pub fn get_sectors_per_fat(&self) -> u32 {
        u32::from_le_bytes(self.sectors_per_fat)
    }

    pub fn get_root_cluster_sector(&self) -> u32 {
        u32::from_le_bytes(self.root_cluster)
    }

    pub fn get_fs_info_cluster_sector(&self) -> u16 {
        u16::from_le_bytes(self.fs_info_sector)
    }

    pub fn get_filesystem_type(&self) -> Result<&str, core::str::Utf8Error> {
        str::from_utf8(&self.file_system_type)
    }

    pub fn as_config(&self) -> FAT32Config {
        FAT32Config {
            bytes_per_sector: self.get_bytes_per_sector(),
            sectors_per_cluster: self.get_sectors_per_cluster(),
            reserved_sectors: self.get_reserved_sectors(),
            number_of_fats: self.get_number_of_fats(),
            root_entry_count: self.get_root_entry_count(),
            total_sectors: self.get_total_sectors32(),
            sectors_per_fat: self.get_sectors_per_fat(),
            root_cluster: self.get_root_cluster_sector(),
            fs_info_sector: self.get_fs_info_cluster_sector()
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MBRSector {
    bootstrap_code_area: [u8; 446],
    pub partition_entries: [PartitionEntry; 4],
    boot_signature: [u8; 2]
}

impl MBRSector {
    pub fn try_from_sector(sector: Sector) -> Result<Self, ()> {
        let mbr_sector_candidate = unsafe {*(&sector as *const Sector as *mut MBRSector)};

        if mbr_sector_candidate.boot_signature[0] != 0x55 ||
            mbr_sector_candidate.boot_signature[1] != 0xAA {
                return Err(());
        } else {
            return Ok(mbr_sector_candidate);
        }
    }
}

#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct PartitionEntry {
    status: u8,
    chs_start_address: [u8; 3],
    partition_type: u8,
    chs_end_address: [u8; 3],
    first_sector_lba: u32,
    sectors_in_partition: u32
}

impl PartitionEntry {
    pub fn get_first_sector_lba(&self) -> u32 {
        self.first_sector_lba
    }

    pub fn get_sectors_in_partition(&self) -> u32 {
        self.sectors_in_partition
    }
}

impl Sector {
    pub const SECTOR_SIZE: usize = 512;

    pub fn load(number: u32, emmc: &mut EMMCController) -> Self {
        let mut sector = Self {
            values: [0; Self::SECTOR_SIZE]
        };

        emmc.read_blocks(number, &mut sector.values, 1);

        return sector;
    }

}

impl fmt::Display for Sector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let columns = 16;
        let rows = Self::SECTOR_SIZE / columns;

        for row in 0..rows {
            write!(f, "\n")?;
            for element in 0..columns {
                let idx = columns * row + element;
                write!(f, " {:02x} ", self.values[idx])?;
            }
            for element in 0..columns {
                let idx = columns * row + element;
                let c = self.values[idx] as char;

                if c.is_ascii_graphic() {
                    write!(f, "{}", c)?;
                } else {
                    write!(f, ".")?;
                }
            }
        }

        Ok(())
    }
}

#[repr(transparent)]
pub struct DirectorySector  {
    pub directory_entries: [DirectoryEntry; 16]
}

impl DirectorySector {
    pub unsafe fn from_sector(sector: Sector) -> Self {
        core::mem::transmute::<Sector, DirectorySector>(sector)
    }
} 

#[repr(packed)]
#[derive(Debug)]
pub struct DirectoryEntry {
    name: [u8; 11],
    attributes: DirectoryAttributes,
    res0: u8,
    creation_time_tenth: u8,
    creation_time: u16,
    creation_date: u16,
    last_access_date: u16,
    first_cluster_high_word: u16,
    last_write_time: u16,
    last_write_date: u16,
    first_cluster_low_word: u16,
    file_size: u32
}

bitfield! {
    DirectoryAttributes(u8) {
        read_only: 1-1,
        hidden: 2-2,
        system: 3-3,
        volume_id: 4-4,
        directory: 5-5,
        archive: 6-6
    }
}

impl DirectoryEntry {
    pub fn get_name(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(&self.name)
    }

    pub fn is_free(&self) -> bool {
        self.name[0] == 0xE5
            || self.name[0] == 0x0
    }
}

impl fmt::Display for DirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entry_type = match self.attributes.get_directory() {
            0 => "File",
            1 => "Directory",
            _ => panic!("")
        };
        write!(f, "{}: {}", entry_type, self.get_name().unwrap())?;

        if(self.attributes.get_read_only() != 0) {
            write!(f, " read only")?;
        }
        
        let size = self.file_size;

        write!(f, " {} sectors", size)?;

        Ok(())
    }
}

#[repr(u32)]
enum FAT32Entry {
    Free = 0x0,
    Allocated(u32),
    Defective,
    Reserved,
    EndOfFile
}

impl FAT32Entry {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => FAT32Entry::Free,
            x if (1..0xFFF_FFF7).contains(&x) => FAT32Entry::Allocated(x),
            0xFFF_FFF7 => FAT32Entry::Defective,
            0xFFF_FFF8..=0xFFF_FFFE => FAT32Entry::Reserved,
            0xFFF_FFFF => FAT32Entry::EndOfFile,
            _ => FAT32Entry::Reserved
        }
    }
}

#[repr(transparent)]
pub struct FATSector {
    fat_entries: [u32; 128]
}

impl FATSector {
    pub fn from_sector(sector: Sector) -> Self {
        unsafe {
            core::mem::transmute::<Sector, FATSector>(sector)
        }
    }
}