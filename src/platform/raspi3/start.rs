use crate::ALLOCATOR;
use super::{
    gpio::{GPIOController, OutputLevel, StatusLight},
    gpu::{FBConfig, GPUController},
    mailbox::MailboxController,
    mmio::MMIOController,
    timer::Timer,
    uart::UARTController,
};

global_asm!(include_str!("start.s"));

#[no_mangle]
pub fn main(heap_start: usize) {
    let mmio = MMIOController::default();
    let gpio = GPIOController::new(&mmio);
    let timer = Timer::new(&mmio);
    let mailbox = MailboxController::new(&mmio);

    let mut uart = UARTController::init(&gpio, &mmio);

    uart.newline();
    uart.newline();
    uart.writeln("UART Connection Initialized");
    uart.newline();

    let heap_size = 65536;

    uart.writeln("Initializing Heap Allocator");

    ALLOCATOR.lock().init(heap_start, heap_size);
    uart.writefln(format_args!("Heap Allocator initialized at {:#x} with size {}", heap_start, heap_size));
    uart.newline();

    uart.writeln("Initializing Status Light");

    let status_light = StatusLight::init(&gpio);

    uart.writeln("Status Light Initialized");
    uart.newline();

    blink_sequence(&status_light, &timer, 100);

    uart.writeln("Initializing GPU");

    let mut gpu = GPUController::init(&mmio, &mailbox, FBConfig::default());

    uart.writeln("GPU Initialized");
    uart.newline();

    loop {
        for offset in 0..64 {
            for y in 0..1080 {
                for x in 0..1920 {
                    let red = (x + 4 * offset) & 0xff;
                    let blue = (y + 4 * offset) & 0xff;
                    let green = 4 * offset;
                    let color = (red << 16) + (blue << 8) + green;
                    gpu.set_pxl(x, y, color);
                }
            }
        }
    }
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
