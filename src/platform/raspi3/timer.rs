//! Contains all necessary functions to interact with the system timer

use super::mmio;

const TIMER_BASE_OFFSET: usize = 0x300;
const CLO_OFFSET: usize = 4;
const CHI_OFFSET: usize = 8;

/// Gets the system time in microseconds.
/// Because the [mmio](super::mmio) module currently only supports 32 bit reads, this is done as two 32 bit reads which are concatenated.
pub fn time() -> u64 {
    let lo =  mmio::read_at_offset(TIMER_BASE_OFFSET + CLO_OFFSET) as u64;
    let hi = mmio::read_at_offset(TIMER_BASE_OFFSET + CHI_OFFSET) as u64;
    (hi << 32) + lo
}

/// Pauses execution of the thread for the amount of time specified in milliseconds
pub fn delay(millis: u64) {
    delay_microseconds(1000 * millis);
}

/// Pauses exection of the thread for the amount of time specified in microseconds
pub fn delay_microseconds(micros: u64) {
    let target = time() + micros;
    while time() < target {}
}
