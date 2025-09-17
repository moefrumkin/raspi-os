use core::arch::global_asm;
use super::{gpio::{GPIOController, StatusLight, OutputLevel}
};

use crate::{aarch64::cpu, platform::platform_devices::get_platform, println};

global_asm!(include_str!("exception.s"));

#[derive(Debug)]
#[repr(u64)]
pub enum ExceptionSource {
    CurrentELUserSP = 0,
    CurrentELCurrentSP = 1,
    LowerEL64 = 2,
    LowerEL32 = 3
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum ExceptionType {
    Synchronous = 0,
    Interrupt = 1,
    FastInterrupt = 2,
    SystemError = 4
}

#[no_mangle]
pub extern "C" fn handle_exception(
        exception_source: ExceptionSource,
        exception_type: ExceptionType
    ) {
    println!(
        "Exception of type {:?} received with source {:?}",
        exception_type,
        exception_source
    );

    if exception_type == ExceptionType::Interrupt {
        get_platform().handle_interrupt();
        return;
    }

    loop {} 
}

/*fn blink_out(n: usize, timer: &Timer, status_light: &StatusLight, wait: u64) {
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
}*/