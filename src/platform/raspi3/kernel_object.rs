use core::fmt::Debug;

use crate::{filesystem::fat32::FAT32DirectoryEntry, platform::platform_devices::get_platform};

pub type ObjectHandle = u64;

pub trait KernelObject: Debug {
    fn read(&self, _: &mut [u8]) -> usize {
        0
    }

    fn write(&self, _: &mut [u8]) -> usize {
        0
    }
}

#[derive(Debug)]
pub struct FileObject {
    fat_entry: FAT32DirectoryEntry,
}

impl FileObject {
    pub fn from_entry(fat_entry: FAT32DirectoryEntry) -> Self {
        Self { fat_entry }
    }
}

impl KernelObject for FileObject {
    fn read(&self, buffer: &mut [u8]) -> usize {
        get_platform().read(self.fat_entry, buffer)
    }
}

#[derive(Debug)]
pub struct Stdio {}

impl Stdio {
    pub fn new() -> Self {
        Self {}
    }
}

impl KernelObject for Stdio {
    fn write(&self, buffer: &mut [u8]) -> usize {
        let msg = core::str::from_utf8(buffer).expect("Error converting strings");

        crate::print!("{}", msg);

        msg.len()
    }
}
