use core::arch::global_asm;
use crate::aarch64::{cpu, mmu};
use crate::canvas::{canvas2d::Canvas2D, line::Line, matrix::Matrix, vector::Vector};
use crate::ALLOCATOR;
use crate::{print, println, read, write};
use alloc::vec::Vec;
use alloc::slice;


use super::{
    gpio::{GPIOController, OutputLevel, Pin, StatusLight},
    lcd::LCDController,
    mailbox::{Channel, MailboxController},
    mmio::MMIOController,
    timer::Timer,
    uart::{LogLevel, UARTController, CONSOLE},
    framebuffer::{
        FrameBuffer, PixelOrder, Overscan, FrameBufferConfig,
        Offset,
        Dimensions,
        FrameBufferConfigBuilder
    },
    hardware_config::HardwareConfig,
    clock::{
        self,
        Clock,
        ClockState,
        CLOCKS
    },
    power::{
        Device,
        PowerState,
        DEVICES
    },
    emmc::{
        EMMCRegisters
    }
};

static MMIO: MMIOController = MMIOController::new();
static GPIO: GPIOController = GPIOController::new(&MMIO);

global_asm!(include_str!("start.s"));

#[no_mangle]
pub extern "C" fn main(heap_start: usize, heap_size: usize, table_start: usize) {
    let mmio = MMIOController::default();
    let gpio = GPIOController::new(&mmio);
    let timer = Timer::new(&mmio);
    
    let status_light = StatusLight::init(&gpio);

    blink_sequence(&status_light, &timer, 100);

    let mut console = UARTController::init(&GPIO, &MMIO);
    console.set_log_level(LogLevel::Debug);

    unsafe {
        // TODO: refactor
        *CONSOLE.lock() = Some(console);
    }

    println!("Entering Boot Sequence (with new build system?)");
    println!("Initializing Memory Virtualization");

    unsafe { 
        mmu::init(table_start as *mut usize);
    };

    println!("Memory Virtualization Initialized");

    println!("Initializing Heap Allocator");
    ALLOCATOR.lock().init(heap_start, heap_size);
    println!("Heap Allocator initialized at {:#x} with size {}", heap_start, heap_size);

    let mut mailbox = MailboxController::new(&mmio);

    let hardware_config = HardwareConfig::from_mailbox(&mut mailbox);

    println!("Hardware Configuration Detected: {}\n", hardware_config);

    println!("Devices:");

    for device in &DEVICES {
        println!("\t-{}: Powered: {}, Timing: {}",
            device,
            device.get_power_state(&mut mailbox).is_on(),
            device.get_timing(&mut mailbox));
    }

    println!("Clocks:");

    for clock in &CLOCKS {
        println!("\t-{}: On: {}, Set Rate: {}, Min Rate: {}, Max Rate: {}",
            clock,
            clock.get_clock_state(&mut mailbox).is_on(),
            clock.get_clock_rate(&mut mailbox),
            clock.get_min_clock_rate(&mut mailbox),
            clock.get_max_clock_rate(&mut mailbox)
        );
    }

    println!("Trying to initialize the sd card");

    let emmc_regs = EMMCRegisters::get();

    emmc_regs.sd_init(&timer, &gpio);

    let mut buffer: [u8; 512] = [0; 512];

    println!("Reading block");
    emmc_regs.sd_readblock(0, &mut buffer, 1, &timer);

    for i in 0..128 {
        if i % 64 == 0 {
            println!("");
        }

        if(buffer[i] != 0) {
            print!("{}", buffer[i] as char);
        }
    }

    println!("");

    let resolution = Dimensions::new(1920, 1080);

    let fb_config = FrameBufferConfigBuilder::new()
        .depth(32)
        .physical_dimensions(resolution)
        .virtual_dimensions(resolution)
        .pixel_order(PixelOrder::RGB)
        .virtual_offset(Offset::none())
        .overscan(Overscan::none())
        .build();

    println!("Initializing Frame Buffer with config {}", fb_config);

    let mut fb = FrameBuffer::from_config(fb_config, &mut mailbox);

    println!("Actual config is {}", fb.get_config());

    for i in 0..(1920 * 1080) {
        fb.write_idx(i, 0xff00ffff);
    }

    for j in 0..1920 {
        for i in 0..1080 {
            fb.write_pixel(j, i, 0xff000000 + ((255 * i / 1080) << 16) + ((255 * j / 1920) << 8) + 0xff);
        }
    }

    println!("Done!");
   
    status_light.set_green(OutputLevel::High);

    loop{}
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

pub fn test_allocator(limit: usize){
    let mut vec_vec: Vec<Vec<usize>> = alloc::vec!();

    for n in 0..limit {
        let num_vec: Vec<usize> = alloc::vec!();
        vec_vec.push(num_vec);
        for m in 0..n {
           vec_vec[n].push(m * n);
        }
    }

    for n in 1 .. limit {
        for m in 1..n {
            if vec_vec[n][m] != m * n  {
                panic!("Expected {:?}, received {:?}", m * n, vec_vec[n][m]);
            }
        }
    }
}
