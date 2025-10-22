use super::gpio::{GPIOController, OutputLevel, StatusLight};
use core::arch::global_asm;

use crate::{
    aarch64::{
        cpu,
        registers::{ExceptionLinkRegister, ExceptionSyndromeRegister, FaultAddressRegister},
    },
    bitfield,
    platform::platform_devices::{get_platform, PLATFORM},
    println,
};

global_asm!(include_str!("exception.s"));

#[derive(Debug)]
#[repr(u64)]
pub enum ExceptionSource {
    CurrentELUserSP = 0,
    CurrentELCurrentSP = 1,
    LowerEL64 = 2,
    LowerEL32 = 3,
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum ExceptionType {
    Synchronous = 0,
    Interrupt = 1,
    FastInterrupt = 2,
    SystemError = 4,
}

#[no_mangle]
pub extern "C" fn handle_exception(
    exception_source: ExceptionSource,
    exception_type: ExceptionType,
    frame: &mut InterruptFrame,
) {
    if exception_type == ExceptionType::Interrupt {
        get_platform().handle_interrupt(frame);
    } else {
        println!(
            "Received Exception Type {:?} from {:?}",
            exception_type, exception_source
        );
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct InterruptFrame {
    pub regs: [u64; 32],
    pub elr: u64,
}

#[no_mangle]
pub extern "C" fn handle_synchronous_exception(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    frame: &mut InterruptFrame,
) {
    println!("Handling synchronous");

    let esr = ExceptionSyndromeRegister::read_to_buffer();
    let elr = ExceptionLinkRegister::read_to_buffer();
    let far = FaultAddressRegister::read_to_buffer();

    println!(
        "ESR: {:x}. ELR: {:x}. FAR: {:x}",
        esr.value(),
        elr.value(),
        far.value()
    );

    let exception_class = esr.get_exception_class();

    println!("Exception class: {:b}", exception_class);

    PLATFORM.update_frame(frame);

    if exception_class == 0b010101 {
        let syscall_number = esr.get_instruction_number();

        println!("arg1: {}", arg1);

        PLATFORM.handle_syscall(syscall_number, [arg1, arg2, arg3]);
    }
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
