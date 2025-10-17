pub trait Timer {
    fn delay_micros(&self, micros: u64);
    fn delay_millis(&self, millis: u64) {
        self.delay_micros(1000 * millis);
    }

    fn get_micros(&self) -> u64;

    fn set_timeout(&self, micros: u32);

    fn clear_matches(&self);
}
