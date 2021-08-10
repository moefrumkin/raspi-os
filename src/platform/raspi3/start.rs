use super::{
    gpio::{GPIOController, StatusLight, OutputLevel},
    timer::Timer,
    uart::UARTController,
    mmio::MMIOController
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

        let mmio = MMIOController::default();
        let gpio = GPIOController::new(&mmio);
        let timer = Timer::new(&mmio);
        let uart = UARTController::init(&gpio, &mmio);
        let status_light = StatusLight::init(&gpio);

        blink_sequence(&status_light, &timer, 250);

        uart.writeln("UART Connection Initialized");

    }

    loop {}
}

pub fn blink_sequence(status_light: &StatusLight, timer: &Timer, interval: u64) {

    status_light.set_green(OutputLevel::High);

    timer.delay(interval);

    status_light.set_green(OutputLevel::Low);
    status_light.set_blue(OutputLevel::High);

    timer.delay(interval);

    status_light.set_blue(OutputLevel::Low);
    status_light.set_red(OutputLevel::High);

    timer.delay(interval);

    status_light.set_red(OutputLevel::Low);
}
