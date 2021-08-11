use super::{
    gpio::{GPIOController, StatusLight, OutputLevel},
    timer::Timer,
    uart::UARTController,
    mmio::MMIOController,
    gpu
};
use crate::aarch64::{cpu, registers::SP};

global_asm!(include_str!("start.s"));

#[no_mangle]
pub fn main() {
    let mmio = MMIOController::default();
    let gpio = GPIOController::new(&mmio);
    let timer = Timer::new(&mmio);
    let uart = UARTController::init(&gpio, &mmio);
    let status_light = StatusLight::init(&gpio);

    blink_sequence(&status_light, &timer, 250);

    uart.writeln("UART Connection Initialized");

    uart.write_hex(&uart as *const UARTController as usize);

    uart.writeln("");

    unsafe {gpu::fn_init(&mmio, &uart);}

    loop {
        unsafe {
            gpu::draw_stuff();
        }
    }
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
