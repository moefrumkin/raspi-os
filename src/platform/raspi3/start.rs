use crate::ALLOCATOR;
use crate::canvas::{canvas2d::Canvas2D, vector::Vector};
use crate::aarch64::cpu;
use crate::{write, read};

use super::{
    gpio::{GPIOController, OutputLevel, StatusLight},
    gpu::{FBConfig, GPUController},
    mailbox::MailboxController,
    mmio::MMIOController,
    timer::Timer,
    uart::{UARTController, LogLevel},
};

global_asm!(include_str!("start.s"));

#[no_mangle]
pub extern "C" fn main(heap_start: usize) {
    if cpu::el() == 2 {
        // Counter and Timer Hyp Control
        // allow el 1 and 0 access to the timer and counter reigsters
        write!("CNTHCTL_EL2", read!("CNTHCTL_EL2") | 0b11);

        // set offset to 0
        write!("CNTVOFF_EL2", 0);

        // allow el1 and 0 to use the fancy SIMD and FP registers (I paid for them, I'm damned well going to use them)
        write!("CPTR_EL2", read!("CPTR_EL2") | (0b11 << 20));
        write!("CPACR_EL1", read!("CPACR_EL1") | (0b11 << 20));

        // set el1 to 64 bit execution
        // 31: 64 bit execution, 1: Set/Way Invalidation Override
        write!("HCR_EL2", (1 << 31) | (1 << 1));

        // Saved program status register
        // fake an exception to enter EL1
        // 9-6: DAIF
        // 5: Res0
        // 4: 0b0: AArch64 execution state
        // 3-0: SP: 0b0100 = EL1h = sp_el0
        write!("SPSR_EL2", 0b1111000100);

        write!("ELR_el2", init_el1 as *const () as usize);

        cpu::eret();
    }
}

pub fn spin() {
    loop {}
}

#[no_mangle]
pub fn init_el1() {
    let mmio = MMIOController::default();
    let gpio = GPIOController::new(&mmio);
    let status_light = StatusLight::init(&gpio);
    let timer = Timer::new(&mmio);
    let mut uart = UARTController::init(&gpio, &mmio);
    
    blink_sequence(&status_light, &timer, 50);

    
}

#[inline(never)]
#[no_mangle]
pub fn fun() {
    let mmio = MMIOController::default();
    let gpio = GPIOController::new(&mmio);
    let timer = Timer::new(&mmio);
    
    let mut uart = UARTController::init(&gpio, &mmio);

    uart.putc('f');
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
