use crate::{device::sector_device::{Sector, SectorAddress, SectorDevice}, filesystem::master_boot_record};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MasterBootRecord {
    bootstrap_code_area: [u8; 446],
    pub partition_entries: [MastBootRecordPartitionEntry; 4],
    boot_signature: u16
}

#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct MastBootRecordPartitionEntry {
    status: u8,
    chs_start_address: [u8; 3],
    partition_type: u8,
    chs_end_address: [u8; 3],
    first_sector_lba: u32,
    sectors_in_partition: u32
}

impl MasterBootRecord {
    const BOOT_SIGNATURE: u16 = 0xAA55;

    pub fn scan_device_for_mbr(sector_device: &mut dyn SectorDevice, start: SectorAddress, end: SectorAddress) -> 
        Result<(SectorAddress, MasterBootRecord), ()> 
    {
        for address in start..end {
            let sector = sector_device.read_sector(address);

            if let Ok(master_boot_record) = MasterBootRecord::try_from(sector) {
                return Ok((address, master_boot_record));
            }
        }

        Err(())
    }

}

impl TryFrom<Sector> for MasterBootRecord {
    type Error = ();

    fn try_from(value: Sector) -> Result<Self, Self::Error> {
        let master_boot_record_candidate = unsafe {
            core::mem::transmute::<Sector, MasterBootRecord>(value)
        };    

        if(master_boot_record_candidate.boot_signature == Self::BOOT_SIGNATURE) {
            return Ok(master_boot_record_candidate)
        } else {
            return Err(());
        }
    }
}

impl MastBootRecordPartitionEntry {
    pub fn first_sector_address(&self) -> SectorAddress {
        self.first_sector_lba
    }

    pub fn sectors_in_partition(&self) -> SectorAddress {
        self.sectors_in_partition
    }

    pub fn last_sector_address(&self) -> SectorAddress {
        self.first_sector_address() + self.sectors_in_partition()
    }
}