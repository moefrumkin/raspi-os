//! The panic handler that is used in the case of a runtime exception
//!
//! The standard library has the default options of stack unwinding or aborting, however neither of those can be used, as the full standard library is not included

use core::panic::PanicInfo;
use super::platform::gpio::{Pin, Mode, OutputLevel};

///The global panic handler
#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! {
    let red_pin = Pin::new(17).unwrap();
    let blue_pin = Pin::new(22).unwrap();
    let green_pin = Pin::new(27).unwrap();

    red_pin.set_mode(Mode::OUT);
    blue_pin.set_mode(Mode::OUT);
    green_pin.set_mode(Mode::OUT);

    red_pin.set_out(OutputLevel::High);
    blue_pin.set_out(OutputLevel::Low);
    green_pin.set_out(OutputLevel::Low);
    
    loop {}
}
