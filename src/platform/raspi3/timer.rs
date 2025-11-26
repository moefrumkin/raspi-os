//! Contains all necessary functions to interact with the system timer
use crate::bitfield;
use crate::volatile::Volatile;

const TIMER_BASE_OFFSET: usize = 0x3000;
const CLO_OFFSET: usize = 4;
const CHI_OFFSET: usize = 8;

#[repr(C)]
#[derive(Debug)]
pub struct TimerRegisters {
    control_status: Volatile<Status>,
    // TODO: can we treat this as a single u64?
    counter_low_bits: Volatile<u32>,
    counter_high_bits: Volatile<u32>,
    compare_values: [Volatile<u32>; 4],
}

impl TimerRegisters {
    fn get_count(&self) -> u64 {
        let low_bits = self.counter_low_bits.get() as u64;
        let high_bits = self.counter_high_bits.get() as u64;

        (high_bits << 32) | low_bits
    }

    /// Gets the system time in microseconds.
    /// Because the [mmio](super::mmio) module currently only supports 32 bit reads, this is done as two 32 bit reads which are concatenated.
    pub fn time(&self) -> u64 {
        self.get_count()
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

    pub fn set_timeout(&mut self, micros: u32) {
        self.compare_values[3].set(self.counter_low_bits.get() + micros)
    }

    pub fn set_kernel_timeout(&mut self, millis: u32) {
        self.compare_values[1].set(self.counter_low_bits.get() + millis);
    }

    pub fn clear_matches(&mut self) {
        self.control_status
            .set(Status::cleared().set_match3(1).set_match1(1));
    }
}

bitfield! {
    Status(u32) {
        match0: 0-0,
        match1: 1-1,
        match2: 2-2,
        match3: 3-3
    } with {
        pub fn cleared() -> Self {
            Self {
                value : 0
            }
        }
    }
}
