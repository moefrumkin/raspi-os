use super::{
    gpio::{Mode, OutputLevel, Pin},
    timer,
};
use crate::aarch64::{cpu, registers::SP};

extern "C" {
    static STACK_PTR: usize;

    static bss_start: *mut usize;
    static bss_end: *mut usize;
}

#[no_mangle]
#[naked]
pub fn _start() {
    if cpu::core_id() == 0 {
        //Zero BSS
        unsafe {
            cpu::init_region(bss_start, bss_end, 0);
        }

        //Set stack pointer
        unsafe {
            SP.write(STACK_PTR);
        }

        blink_sequence(500);
    }

    loop {}
}

pub fn blink_sequence(interval: u64) {
    let red_pin = Pin::new(17).unwrap();
    let blue_pin = Pin::new(22).unwrap();
    let green_pin = Pin::new(27).unwrap();

    red_pin.set_mode(Mode::OUT);
    blue_pin.set_mode(Mode::OUT);
    green_pin.set_mode(Mode::OUT);

    green_pin.set_out(OutputLevel::High);

    timer::delay(interval);

    green_pin.set_out(OutputLevel::Low);
    blue_pin.set_out(OutputLevel::High);

    timer::delay(interval);

    blue_pin.set_out(OutputLevel::Low);
    red_pin.set_out(OutputLevel::High);

    timer::delay(interval);

    red_pin.set_out(OutputLevel::Low);
}
