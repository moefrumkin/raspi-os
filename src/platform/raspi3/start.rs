use crate::ALLOCATOR;
use crate::canvas::{canvas2d::Canvas2D, vector::Vector, matrix::Matrix, line::Line};
use crate::aarch64::cpu;
use crate::{write, read};

use super::{
    gpio::{GPIOController, OutputLevel, StatusLight, Pin},
    gpu::{FBConfig, GPUController},
    mailbox::MailboxController,
    mmio::MMIOController,
    timer::Timer,
    uart::{UARTController, LogLevel},
    lcd::LCDController
};

global_asm!(include_str!("start.s"));

#[no_mangle]
pub extern "C" fn main(heap_start: usize) {
    let mmio = MMIOController::default();
    let gpio = GPIOController::new(&mmio);
    let timer = Timer::new(&mmio);
    let mailbox = MailboxController::new(&mmio);

    let mut uart = UARTController::init(&gpio, &mmio);
    uart.set_log_level(LogLevel::Debug);

    uart.newline();
    uart.newline();
    uart.writeln("UART Connection Initialized");
    uart.newline();

    let heap_size = 1048576;

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

    uart.writeln("GPU Initialized with Config:");
    uart.writefln(format_args!("{:?}", gpu.config()));
    uart.newline();

    /*for y in 0..1080 {
        for x in 0..1920 {
            let red = x & 0xff;
            let blue = y & 0xff;
            let green = 0;
            let color = (red << 16) + (green << 8) + blue;
            gpu.set_pxl(x, y, 0xffffff as u32);
        }
    }*/

    uart.writeln("Initializing Canvas");

    let mut canvas = Canvas2D::new(&mut gpu, 1920, 1080);

    uart.writeln("Canvas Initialized");

    canvas.add_line(Line (Vector (0.0, 0.0), Vector (500.0, 250.0)), 0xaa00ff);
    canvas.add_line(Line (Vector (500.0, 250.0), Vector (700.0, 270.0)), 0x00aaff);
    
    canvas.add_point(Vector (1.0, 0.0), 0x00ff00);
    /*for x in 0..250 {
        let red = (8 * x) & 0xff;
        let blue = !((8 * x) & 0xff);
        let green = 0;
        let color = (red << 16) + (green << 8) + blue;
        canvas.add_point(Vector (x as f64, x as f64), 0);
    }*/

    let rot = Matrix ( Vector (0.99984769515, -0.01745240643), Vector (0.01745240643, 0.99984769515) );

    uart.writeln("Drawing Canvas");
    canvas.draw(Vector(-960.0, -540.0), 1920.0, 1080.0);
    /*if cpu::el() == 2 {
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
    }*/
}

#[no_mangle]
pub fn init_el1() {
    let mmio = MMIOController::default();
    let gpio = GPIOController::new(&mmio);
    let status_light = StatusLight::init(&gpio);
    let timer = Timer::new(&mmio);

    blink_sequence(&status_light, &timer, 50);

    fun();
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
