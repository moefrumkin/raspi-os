#[cfg(not(test))]
global_asm!(include_str!("start.s"));

use super::gpu;
use super::gpio;
use super::timer;

#[no_mangle]
pub extern "C" fn start() {
    let pin = gpio::Pin::new(23).unwrap();
    pin.set_mode(gpio::Mode::OUT);

    for i in 0..=10 {
        pin.set_out(gpio::OutputLevel::High);
        timer::delay(5000);
        pin.set_out(gpio::OutputLevel::Low);
        timer::delay(5000);
    }

    /*let fb = gpu::init();
    
    if let Ok(fbi) = fb {
        fbi.draw();
    }*/
}
