use super::emmc::EMMCController;
use super::uart::CONSOLE;
use core::fmt;
use crate::{bitfield, print, println};
use alloc::vec::Vec;
use crate::utils::fat_name::fat_name_from_chars;

#[repr(transparent)]
#[derive(Copy, Clone)]
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

pub struct FAT32Filesystem<'a> {
    emmc_controller: &'a mut EMMCController<'a>,

    config: FAT32Config,

    boot_sector: u32,
    fat_start: u32,
    data_start: u32
}

impl<'a> FAT32Filesystem<'a> {
    pub fn new(emmc_controller: &'a mut EMMCController<'a>, partition_start: u32) -> Result<Self, &'a str> {
        let mut boot_sector = partition_start;
        let config;

        // Error if we go beyond end of filesystem
        loop {
            let sector = Sector::load(boot_sector, emmc_controller);

            if let Ok(sector) = BootSector::try_from_sector(&sector) {
                config = sector.as_config();
                break;
            }

            boot_sector += 1;
        }

        let fat_start = boot_sector + config.reserved_sectors as u32;
        let data_start = fat_start + config.number_of_fats as u32 * config.sectors_per_fat;

        Ok(Self {
            emmc_controller,
            boot_sector,
            config,

            fat_start,
            data_start
        })
    }

    fn cluster_number_to_sector_number(&self, cluster_number: u32) -> u32 {
        self.data_start + (cluster_number - 2) * self.config.sectors_per_cluster as u32
    }

    fn get_fat_entry(&mut self, cluster_number: u32) -> FAT32Entry {
        let fat_offset = cluster_number * 4; // FAT32 specific
        let fat_sector_number = self.fat_start + (fat_offset / self.config.bytes_per_sector as u32);
        let fat_sector_offset = cluster_number % self.config.bytes_per_sector as u32;

        let fat_sector = FATSector::from_sector(Sector::load(fat_sector_number, self.emmc_controller));

        fat_sector.get_entry(fat_sector_offset)
    }

    // True if continue, false if otherwise
    fn read_directory_cluster(&mut self, cluster: u32, entries: &mut Vec<DirectoryEntry>) -> bool {
        let first_sector = self.cluster_number_to_sector_number(cluster);

        for sector_number in first_sector..first_sector + self.config.sectors_per_cluster as u32{
            let sector = DirectorySector::from_sector(Sector::load(sector_number, self.emmc_controller));

            for entry_number in 0..16 {
                let entry = sector.directory_entries[entry_number];

                if entry.is_directory_end() {
                    return false;
                }

                if entry.is_directory_entry() {
                    entries.push(entry);
                }
            }
        }

        true
    }

    pub fn read_directory(&mut self, cluster_number: u32) -> Directory {
        let mut entries = Vec::new();

        let mut current_cluster = cluster_number;
        let mut keep_reading = true;

        while keep_reading {
            keep_reading = self.read_directory_cluster(current_cluster, &mut entries);

            if keep_reading {
                let fat_entry = self.get_fat_entry(current_cluster);

                match fat_entry {
                    FAT32Entry::Free | FAT32Entry::Defective | FAT32Entry::Reserved => panic!("Unexpected FAT entry"),
                    FAT32Entry::Allocated(next_cluster) => current_cluster = next_cluster,
                    FAT32Entry::EndOfFile => keep_reading = false
                }
            }
        }

        Directory {
            name: "",

            entries
        } 
    }

    pub fn get_root_directory(&mut self) -> Directory {
        self.read_directory(self.config.root_cluster)
    }
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
    pub fn from_sector(sector: Sector) -> Self {
        unsafe {
            core::mem::transmute::<Sector, DirectorySector>(sector)
        }
    }
} 

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
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
        read_only: 0-0,
        hidden: 1-1,
        system: 2-2,
        volume_id: 3-3,
        directory: 4-4,
        archive: 5-5
    }
}

impl DirectoryEntry {
    pub fn get_name(&self) -> Result<alloc::string::String, core::str::Utf8Error> {
        Ok(fat_name_from_chars(&self.name))
    }

    pub fn is_directory_entry(&self) -> bool {
        !self.is_free()
        && !self.is_directory_end()
        && !self.is_long_name()
        && !self.is_type_volume_id()
    }

    pub fn is_free(&self) -> bool {
        self.name[0] == 0xE5
            || self.name[0] == 0x0
    }

    pub fn is_directory_end(&self) -> bool {
        self.name[0] == 0x0
    }

    pub fn is_long_name(&self) -> bool {
        let attributes = self.attributes;

        return attributes.get_read_only() == 1
            || attributes.get_hidden() == 1
            || attributes.get_system() == 1
            || attributes.get_volume_id() == 1;
    }

    pub fn is_type_volume_id(&self) -> bool {
        self.attributes.get_volume_id() == 1
    }

    pub fn first_sector(&self) -> u32 {
        self.first_cluster_low_word as u32 | ((self.first_cluster_high_word as u32) << 16)
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

        write!(f, " starting at cluster {}", self.first_sector())?;

        Ok(())
    }
}

#[repr(u32)]
#[derive(Debug)]
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

    fn get_entry(&self, number: u32) -> FAT32Entry {
        FAT32Entry::from_u32(self.fat_entries[number as usize] &0xFFF_FFFF)
    }
}

pub struct Directory<'a> {
    name: &'a str,

    entries: Vec<DirectoryEntry>
}

impl<'a> fmt::Display for Directory<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Directory: {}", self.name)?;

        for entry in &self.entries {
            write!(f, "\n\t {}", entry)?;
        }

        Ok(())
    }
}