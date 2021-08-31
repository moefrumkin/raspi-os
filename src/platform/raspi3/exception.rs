use super::{mmio::MMIOController, gpio::{GPIOController, StatusLight, OutputLevel}, timer::Timer};

global_asm!(include_str!("exception.s"));

#[no_mangle]
pub extern "C" fn handle_exception(exception_source: usize, exception_type: usize, esr: usize, elr: usize, spsr: usize, far: usize, sp: usize) {
    let mmio = MMIOController::default();
    let gpio = GPIOController::new(&mmio);
    let timer = Timer::new(&mmio);
    let status_light = StatusLight::init(&gpio);

    const LONG_WAIT: u64 = 500;
    const SHORT_WAIT: u64 = 250;

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

        for i in 0..32 {
            if (esr >> i) & 1 == 1 {
                status_light.set_green(OutputLevel::High);
                timer.delay(SHORT_WAIT);
                status_light.set_green(OutputLevel::Low);
            } else {
                status_light.set_blue(OutputLevel::High);
                timer.delay(SHORT_WAIT);
                status_light.set_blue(OutputLevel::Low);
            }
            timer.delay(SHORT_WAIT);
        }

        timer.delay(LONG_WAIT);

        for i in 0..32 {
            if (esr >> i) & 1 == 1 {
                status_light.set_green(OutputLevel::High);
                timer.delay(SHORT_WAIT);
                status_light.set_green(OutputLevel::Low);
            } else {
                status_light.set_blue(OutputLevel::High);
                timer.delay(SHORT_WAIT);
                status_light.set_blue(OutputLevel::Low);
            }
            timer.delay(SHORT_WAIT);
        }

        timer.delay(LONG_WAIT);
    }
}