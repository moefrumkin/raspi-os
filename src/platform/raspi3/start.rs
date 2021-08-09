use super::{
    gpio::{StatusLight, OutputLevel},
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
    let status_light = StatusLight::init();

    status_light.set_green(OutputLevel::High);

    timer::delay(interval);

    status_light.set_green(OutputLevel::Low);
    status_light.set_blue(OutputLevel::High);

    timer::delay(interval);

    status_light.set_blue(OutputLevel::Low);
    status_light.set_red(OutputLevel::High);

    timer::delay(interval);

    status_light.set_red(OutputLevel::Low);
}
