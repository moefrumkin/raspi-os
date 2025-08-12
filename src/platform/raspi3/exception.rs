use core::arch::global_asm;
use super::{gpio::{GPIOController, StatusLight, OutputLevel}, timer::Timer,
    mini_uart::MiniUARTController
};

use crate::println;

global_asm!(include_str!("exception.s"));

#[no_mangle]
pub extern "C" fn handle_exception(exception_source: usize, exception_type: usize, esr: usize, elr: usize, _spsr: usize, _far: usize, _sp: usize) {
    println!(
        "Exception of type {} received with source {}",
        exception_type,
        exception_source
    );

    loop {} 
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
