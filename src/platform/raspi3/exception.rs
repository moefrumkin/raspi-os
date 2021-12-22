use super::{mmio::MMIOController, gpio::{GPIOController, StatusLight, OutputLevel}, timer::Timer};

global_asm!(include_str!("exception.s"));

#[no_mangle]
pub extern "C" fn handle_exception(exception_source: usize, exception_type: usize, esr: usize, elr: usize, spsr: usize, far: usize, sp: usize) {
    let mmio = MMIOController::default();
    let gpio = GPIOController::new(&mmio);
    let timer = Timer::new(&mmio);
    let status_light = StatusLight::init(&gpio);

    const LONG_WAIT: u64 = 2500;
    const SHORT_WAIT: u64 = 1000;

    loop {
        for i in 0..exception_source + 5{
            status_light.set_blue(OutputLevel::High);
            timer.delay(SHORT_WAIT);
            status_light.set_blue(OutputLevel::Low);
            timer.delay(SHORT_WAIT);
        }

        timer.delay(LONG_WAIT);

        for i in 0..exception_type + 5 {
            status_light.set_red(OutputLevel::High);
            timer.delay(SHORT_WAIT);
            status_light.set_red(OutputLevel::Low);
            timer.delay(SHORT_WAIT);
        }

        timer.delay(LONG_WAIT);

        blink_out(elr, &timer, &status_light, SHORT_WAIT);

        timer.delay(LONG_WAIT);
    }
}

fn blink_out(n: usize, timer: &Timer, status_light: &StatusLight, wait: u64) {
    for i in 0..64 {
        if (n >> (64 - i)) & 1 == 1 {
            status_light.set_green(OutputLevel::High);
            timer.delay(wait);
            status_light.set_green(OutputLevel::Low);
        } else {
            status_light.set_blue(OutputLevel::High);
            timer.delay(wait);
            status_light.set_blue(OutputLevel::Low);
        }
        timer.delay(wait);
    }
}