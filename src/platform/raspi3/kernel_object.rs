use core::fmt::Debug;
use alloc::sync::Arc;

use crate::{aarch64::interrupt::IRQLock, filesystem::fat32::{FAT32DirectoryEntry, FAT32Filesystem}, platform::platform_devices::{PLATFORM, get_platform}};

pub type ObjectHandle = u64;

pub trait KernelObject: Debug {
    fn read(&self, _: &mut [u8]) -> usize {
        0
    }
}

#[derive(Debug)]
pub struct FileObject {
    fat_entry: FAT32DirectoryEntry,
}

impl FileObject {
    pub fn from_entry(fat_entry: FAT32DirectoryEntry,
    ) -> Self {
        Self {
            fat_entry,
        }
    }
}

impl KernelObject for FileObject {
    fn read(&self, buffer: &mut [u8]) -> usize {
        get_platform().read(self.fat_entry, buffer)
    }
}