pub trait Timer {
    fn delay_micros(&self, micros: u64);
    fn delay_millis(&self, millis: u64);
    fn get_micros(&self) -> u64;
}