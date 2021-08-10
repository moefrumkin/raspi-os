//! The panic handler that is used in the case of a runtime exception
//!
//! The standard library has the default options of stack unwinding or aborting, however neither of those can be used, as the full standard library is not included

use core::panic::PanicInfo;
#[cfg(feature = "raspi3")]
use super::platform::{
    gpio::{StatusLight, GPIOController, OutputLevel},
    mmio::MMIOController,
    uart::UARTController,
    timer::Timer
};

///The global panic handler
#[cfg(feature = "raspi3")]
#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! {
    let mmio = MMIOController::default();
    let gpio = GPIOController::new(&mmio);
    let timer = Timer::new(&mmio);
    let uart = UARTController::init(&gpio, &mmio);
    let status_light = StatusLight::init(&gpio);


    status_light.set_green(OutputLevel::Low);
    status_light.set_blue(OutputLevel::Low);

    status_light.set_red(OutputLevel::High);
    
    loop {}
}

#[cfg(not(feature = "raspi3"))]
#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! {
    loop {}
}