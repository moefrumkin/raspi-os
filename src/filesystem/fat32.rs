use crate::{bitfield, device::sector_device::{Sector, SectorAddress, SectorDevice}, utils::fat_name::fat_name_from_chars};
use core::{cell::RefCell, fmt::{self, Display, Formatter}};
use alloc::vec::Vec;
use alloc::rc::Rc;

pub struct FAT32Filesystem {
    sector_device: Rc<RefCell<dyn SectorDevice>>,

    config: FAT32Config,

    boot_sector: SectorAddress,
    fat_start: SectorAddress,
    data_start: SectorAddress,
    number_of_sectors: SectorAddress
}

#[repr(C)]
#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct FAT32BootSector {
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

#[repr(transparent)]
pub struct FAT32FATSector {
    fat_entries: [u32; 128]
}

#[derive(Debug)]
enum FAT32Entry {
    Free,
    Allocated(u32),
    Defective,
    Reserved,
    EndOfFile
}

pub struct FAT32Directory<'a> {
    name: &'a str,

    entries: Vec<FAT32DirectoryEntry>
}

#[repr(transparent)]
pub struct FAT32DirectorySector  {
    pub directory_entries: [FAT32DirectoryEntry; 16]
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct FAT32DirectoryEntry {
    name: [u8; 11],
    attributes: FAT32DirectoryAttributes,
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
    FAT32DirectoryAttributes(u8) {
        read_only: 0-0,
        hidden: 1-1,
        system: 2-2,
        volume_id: 3-3,
        directory: 4-4,
        archive: 5-5
    }
}

impl FAT32Filesystem {
    pub fn load_in_partition(sector_device: Rc<RefCell<dyn SectorDevice>>, start: SectorAddress, end: SectorAddress) ->
        Result<Self, ()>
    {
        if let Ok((boot_sector_number, boot_sector)) = 
            FAT32BootSector::scan_for_boot_sector(sector_device.clone(), start, end) {
            let config = FAT32Config::from(boot_sector);
            let fat_start = boot_sector_number + config.reserved_sectors as u32;
            let data_start = fat_start + config.number_of_fats as u32 * config.sectors_per_fat;
            let number_of_sectors = end - start;

            return Ok(Self {
                sector_device,
                boot_sector: boot_sector_number,
                config,

                fat_start,
                data_start,
                number_of_sectors
            });
        } else {
            return Err(());
        }
    }
    
    pub fn read_directory(&mut self, cluster_number: u32) -> FAT32Directory<'_> {
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

        FAT32Directory {
            name: "",

            entries
        } 
    }

    pub fn get_root_directory(&mut self) -> FAT32Directory<'_> {
        self.read_directory(self.config.root_cluster)
    }
    
    fn read_directory_cluster(&mut self, cluster: u32, entries: &mut Vec<FAT32DirectoryEntry>) -> bool {
        let first_sector = self.cluster_number_to_sector_number(cluster);

        for sector_number in first_sector..first_sector + self.config.sectors_per_cluster as u32{
            let sector = FAT32DirectorySector::from(self.sector_device.borrow_mut().read_sector(sector_number));

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
    
    fn cluster_number_to_sector_number(&self, cluster_number: u32) -> u32 {
        self.data_start + (cluster_number - 2) * self.config.sectors_per_cluster as u32
    }

    fn get_fat_entry(&mut self, cluster_number: u32) -> FAT32Entry {
        let fat_offset = cluster_number * 4; // FAT32 specific
        let fat_sector_number = self.fat_start + (fat_offset / self.config.bytes_per_sector as u32);
        let fat_sector_offset = cluster_number % self.config.bytes_per_sector as u32;

        let fat_sector = FAT32FATSector::from(self.sector_device.borrow_mut().read_sector(fat_sector_number));

        fat_sector.get_fat32_entry(fat_sector_offset)
    }
}

impl Display for FAT32Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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


impl From<FAT32BootSector> for FAT32Config {
    fn from(value: FAT32BootSector) -> Self {
        Self {
            bytes_per_sector: value.get_bytes_per_sector(),
            sectors_per_cluster: value.get_sectors_per_cluster(),
            reserved_sectors: value.get_reserved_sectors(),
            number_of_fats: value.get_number_of_fats(),
            root_entry_count: value.get_root_entry_count(),
            total_sectors: value.get_total_sectors32(),
            sectors_per_fat: value.get_sectors_per_fat(),
            root_cluster: value.get_root_cluster_sector(),
            fs_info_sector: value.get_fs_info_cluster_sector()
        }
    }
}

impl TryFrom<Sector> for FAT32BootSector {
    type Error = ();

    fn try_from(value: Sector) -> Result<Self, Self::Error> {
        let candidate_boot_sector: Self = unsafe {
            core::mem::transmute(value)
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
}

impl FAT32BootSector {
    pub fn scan_for_boot_sector(
        sector_device: Rc<RefCell<dyn SectorDevice>>,
        start: SectorAddress,
        end: SectorAddress
    ) -> Result<(SectorAddress, Self), ()> 
    {
        for address in start..end {
            let sector = sector_device.borrow_mut().read_sector(address);

            if let Ok(boot_sector) = Self::try_from(sector) {
                return Ok((address, boot_sector));
            }
        }

        Err(())
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
}

impl From<Sector> for FAT32FATSector {
    fn from(value: Sector) -> Self {
        unsafe {
            core::mem::transmute(value)
        }
    }
}

impl FAT32FATSector {
    const FAT_ENTRY_MASK: u32 = 0x0FFF_FFFF;

    fn get_fat32_entry(&self, number: SectorAddress) -> FAT32Entry {
        FAT32Entry::from(self.fat_entries[number as usize] & Self::FAT_ENTRY_MASK)
    }
}

impl From<u32> for FAT32Entry {
    fn from(value: u32) -> Self {
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

impl Display for FAT32Directory<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Directory: {}", self.name)?;

        for entry in &self.entries {
            write!(f, "\n\t {}", entry)?;
        }

        Ok(())
    }
}

impl From<Sector> for FAT32DirectorySector {
    fn from(value: Sector) -> Self {
        unsafe {
            core::mem::transmute(value)
        }
    }
}

impl FAT32DirectoryEntry {
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

impl Display for FAT32DirectoryEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let entry_type = match self.attributes.get_directory() {
            0 => "File",
            1 => "Directory",
            _ => panic!("")
        };
        write!(f, "{}: {}", entry_type, self.get_name().unwrap())?;

        if self.attributes.get_read_only() != 0 {
            write!(f, " read only")?;
        }
        
        let size = self.file_size;

        write!(f, " {} sectors", size)?;

        write!(f, " starting at cluster {}", self.first_sector())?;

        Ok(())
    }
}