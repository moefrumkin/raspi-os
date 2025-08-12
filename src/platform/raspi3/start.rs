use core::arch::global_asm;
use crate::aarch64::{cpu, mmu, interrupt};
use crate::canvas::{canvas2d::Canvas2D, line::Line, matrix::Matrix, vector::Vector};
use crate::ALLOCATOR;
use crate::{print, println, read, write};
use alloc::vec::Vec;
use alloc::slice;

use crate::{
    device::sector_device::SectorDevice,
    filesystem::{
        master_boot_record::{MasterBootRecord},
        fat32::{FAT32Filesystem}
    }
};

use super::{
    gpio::{GPIOController, OutputLevel, Pin, StatusLight},
    mailbox::{Channel, MailboxController},
    timer::Timer,
    mini_uart::{LogLevel, MiniUARTController},
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
        EMMCRegisters,
        EMMCController
    },
    interrupt::{
        InterruptController
    },
    hardware_devices::{
        PLATFORM
    }
};

global_asm!(include_str!("start.s"));

#[no_mangle]
pub extern "C" fn main(heap_start: usize, heap_size: usize, table_start: usize) {
    ALLOCATOR.lock().init(heap_start, heap_size);
    PLATFORM.init();

    //let status_light = PLATFORM.get_status_light().unwrap();
    let timer = PLATFORM.get_timer().unwrap();
    let console = PLATFORM.get_console().unwrap();

    //blink_sequence(&status_light.borrow(), &timer.borrow(), 100);

    println!("Starting");

    console.borrow_mut().set_log_level(LogLevel::Debug);

    println!("Entering Boot Sequence (with new build system?)");
    println!("Initializing Memory Virtualization");

    unsafe { 
        mmu::init(table_start as *mut usize);
    };

    println!("Memory Virtualization Initialized");

    println!("Initializing Heap Allocator");
    println!("Heap Allocator initialized at {:#x} with size {}", heap_start, heap_size);

    let mut mailbox = PLATFORM.get_mailbox_controller().unwrap();

    let hardware_config = HardwareConfig::from_mailbox(mailbox.clone());

    println!("Hardware Configuration Detected: {}\n", hardware_config);

    println!("Devices:");

    for device in &DEVICES {
        println!("\t-{}: Powered: {}, Timing: {}",
            device,
            device.get_power_state(mailbox.clone()).is_on(),
            device.get_timing(mailbox.clone()));
    }

    println!("Clocks:");

    for clock in &CLOCKS {
        println!("\t-{}: On: {}, Set Rate: {}, Min Rate: {}, Max Rate: {}",
            clock,
            clock.get_clock_state(mailbox.clone()).is_on(),
            clock.get_clock_rate(mailbox.clone()),
            clock.get_min_clock_rate(mailbox.clone()),
            clock.get_max_clock_rate(mailbox.clone())
        );
    }

    let mut emmc_controller = PLATFORM.get_emmc_controller().unwrap();

    let (mbr_sector_number, master_boot_record) = MasterBootRecord::scan_device_for_mbr(
        emmc_controller.clone(),
        0,
        20)
        .expect("Unable to read Master Boot Record");

    let partition = master_boot_record.partition_entries[0];
    
    let mut filesystem = FAT32Filesystem::load_in_partition(
        emmc_controller.clone(),
        mbr_sector_number + partition.first_sector_address(),
        mbr_sector_number + partition.last_sector_address())
        .expect("Unable to initialize a FAT32 filesystem in partition");

    let root_dir = filesystem.get_root_directory();

    println!("Root directory: {}", root_dir);

    println!("Enabling IRQs");

    interrupt::enable_irq();

    let mut interrupt_controller = InterruptController::new();

    interrupt_controller.enable_timer_interrupt_3();
    interrupt_controller.enable_mini_uart_interrupt();

    println!("Timer interrupt enabled!");

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

    let mut fb = FrameBuffer::from_config(fb_config, mailbox);

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
   
    //status_light.borrow_mut().set_green(OutputLevel::High);

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
