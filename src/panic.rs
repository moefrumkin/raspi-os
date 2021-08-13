//! The panic handler that is used in the case of a runtime exception
//!
//! The standard library has the default options of stack unwinding or aborting, however neither of those can be used, as the full standard library is not included

#[cfg(feature = "raspi3")]
use super::platform::{
    gpio::{GPIOController, OutputLevel, StatusLight},
    mmio::MMIOController,
    uart::UARTController,
};
use core::{panic::PanicInfo, alloc::Layout};

///The global panic handler
#[cfg(feature = "raspi3")]
#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    let mmio = MMIOController::default();
    let gpio = GPIOController::new(&mmio);
    let uart = UARTController::init(&gpio, &mmio);
    let status_light = StatusLight::init(&gpio);

    status_light.set_green(OutputLevel::Low);
    status_light.set_blue(OutputLevel::Low);

    status_light.set_red(OutputLevel::High);

    uart.writeln("");
    uart.writeln("A Fatal Kernel Panic Occured");
    if let Some(args) = info.message() {
        if let Some(location) = args.as_str() {
            uart.writeln(location);
        } else {
            uart.writeln("No message supplied");
        }
    }

    if let Some(location) = info.location() {
        uart.write("@ ");
        uart.write(location.file());
        uart.write(": ");
        uart.write_hex(location.line() as usize);
        uart.writeln("");
    } else {
        uart.writeln("No location found");
    }

    loop {}
}

#[cfg(feature = "raspi3")]
#[alloc_error_handler]
fn on_alloc_error(layout: Layout) -> ! {
    panic!("Unable to allocate: {:?}", layout);
}

#[cfg(not(feature = "raspi3"))]
#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[cfg(not(feature = "raspi3"))]
#[alloc_error_handler]
fn on_alloc_error(_layout: Layout) -> ! {
    panic!();
}