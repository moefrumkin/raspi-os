//! Contains all necessary functions to interact with the system timer

use super::mmio::MMIOController;

const TIMER_BASE_OFFSET: usize = 0x3000;
const CLO_OFFSET: usize = 4;
const CHI_OFFSET: usize = 8;

pub struct Timer<'a> {
    mmio: &'a MMIOController,
}

impl<'a> Timer<'a> {
    pub fn new(mmio: &'a MMIOController) -> Self {
        Timer { mmio }
    }

    /// Gets the system time in microseconds.
    /// Because the [mmio](super::mmio) module currently only supports 32 bit reads, this is done as two 32 bit reads which are concatenated.
    pub fn time(&self) -> u64 {
        let lo = self.mmio.read_at_offset(TIMER_BASE_OFFSET + CLO_OFFSET) as u64;
        let hi = self.mmio.read_at_offset(TIMER_BASE_OFFSET + CHI_OFFSET) as u64;
        (hi << 32) + lo
    }

    /// Pauses execution of the thread for the amount of time specified in milliseconds
    pub fn delay(&self, millis: u64) {
        self.delay_microseconds(1000 * millis);
    }

    /// Pauses exection of the thread for the amount of time specified in microseconds
    pub fn delay_microseconds(&self, micros: u64) {
        let target = self.time() + micros;
        while self.time() < target {}
    }
}
