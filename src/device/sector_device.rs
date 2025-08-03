#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Sector {
    pub values: [u8; Self::SECTOR_SIZE]
}

pub type SectorAddress = u32;

pub trait SectorDevice {
    fn read_sector(&mut self, address: SectorAddress) -> Sector;
}

impl Sector {
    pub const SECTOR_SIZE: usize = 512;
}

impl From<[u8; Self::SECTOR_SIZE]> for Sector {
    fn from(value: [u8; Self::SECTOR_SIZE]) -> Self {
        Self { values: value }
    }
}

