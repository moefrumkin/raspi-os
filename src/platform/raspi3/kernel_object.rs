use core::fmt::Debug;

use crate::filesystem::fat32::FAT32DirectoryEntry;

pub type ObjectHandle = u64;

pub trait KernelObject: Debug {

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

}