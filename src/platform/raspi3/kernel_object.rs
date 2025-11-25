use core::fmt::Debug;

use crate::filesystem::fat32::FAT32DirectoryEntry;

pub type ObjectHandle = u64;

pub trait KernelObject: Debug {
    fn read(&self, _: &mut [u8]) -> usize {
        0
    }
}

#[derive(Debug)]
pub struct FileObject {
    fat_entry: FAT32DirectoryEntry
}

impl FileObject {
    pub fn from_entry(fat_entry: FAT32DirectoryEntry) -> Self {
        Self {
            fat_entry
        }
    }
}

impl KernelObject for FileObject {
    fn read(&self, buffer: &mut [u8]) -> usize {
        crate::println!("Reading: {:#p}", buffer);

        0
    }
}