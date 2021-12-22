use crate::aarch64::cpu;
use crate::canvas::{canvas2d::Canvas2D, line::Line, matrix::Matrix, vector::Vector};
use crate::ALLOCATOR;
use crate::{print, println, read, write};

use super::{
    gpio::{GPIOController, OutputLevel, Pin, StatusLight},
    gpu::{FBConfig, GPUController},
    lcd::LCDController,
    mailbox::{Channel, Instruction, MailboxController, MessageBuffer, MessageBuilder},
    mmio::MMIOController,
    timer::Timer,
    uart::{LogLevel, UARTController, CONSOLE},
};

static MMIO: MMIOController = MMIOController::new();
static GPIO: GPIOController = GPIOController::new(&MMIO);

global_asm!(include_str!("start.s"));

#[no_mangle]
pub extern "C" fn main(heap_start: usize, heap_size: usize, mailbox_start: usize) {
    let mmio = MMIOController::default();
    let gpio = GPIOController::new(&mmio);
    let timer = Timer::new(&mmio);
    let mut message_buffer = MessageBuffer::new();
    let mut mailbox = MailboxController::new(&mmio, &mut message_buffer);

    let mut console = UARTController::init(&GPIO, &MMIO);
    console.set_log_level(LogLevel::Debug);
    unsafe {
        *CONSOLE.lock() = Some(console);
    }

    println!();
    println!();
    println!("UART Connection Initialized");
    println!();

    println!("Initializing Heap Allocator");

    ALLOCATOR.lock().init(heap_start, heap_size);
    println!(
        "Heap Allocator initialized at {:#x} with size {}",
        heap_start, heap_size
    );
    println!();

    println!("Initializing Status Light");

    let status_light = StatusLight::init(&gpio);

    println!("Status Light Initialized");
    println!();

    blink_sequence(&status_light, &timer, 100);

    println!("Testing Message Buffer");
    let mut mb = MessageBuffer::new();
    println!(
        "Message Buffer Acquired At: {:#x}",
        &mb as *const MessageBuffer as usize
    );
    mb.data[0] = 44;
    mb.data[1] = 0; //Req
    mb.data[2] = 0x30002;
    mb.data[3] = 8;
    mb.data[4] = 8;
    mb.data[5] = 0x3;
    mb.data[6] = 0;

    let val = mailbox.call(&mb as *const MessageBuffer as u32, Channel::Prop) & !0b1111;
    println!("Message Received at {:#x}: {:?}", val, unsafe {
        core::slice::from_raw_parts(val as *const u32, 64)
    });

    println!("Initializing GPU");

    let mut gpu = GPUController::init(&mmio, &mailbox, FBConfig::default());

    println!("GPU Initialized with Config:");
    println!("{:?}", gpu.config());
    println!();

    println!("Initializing Canvas");

    let mut canvas = Canvas2D::new(&mut gpu, 1920, 1080);

    println!("Canvas Initialized");

    canvas.add_line(Line(Vector(0.0, 0.0), Vector(500.0, 250.0)), 0xaa00ff);
    canvas.add_line(Line(Vector(500.0, 250.0), Vector(700.0, 270.0)), 0x00aaff);

    canvas.add_point(Vector(1.0, 0.0), 0x00ff00);

    let rot = Matrix(
        Vector(0.99984769515, -0.01745240643),
        Vector(0.01745240643, 0.99984769515),
    );

    println!("Drawing Canvas");
    canvas.draw(Vector(-960.0, -540.0), 1920.0, 1080.0);
    loop {}
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
