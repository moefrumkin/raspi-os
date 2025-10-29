use super::kernel::Kernel;
use super::kernel::TICK;
use crate::aarch64::{cpu, interrupt, mmu, syscall::Syscall};
use crate::allocator::page_allocator::PageAllocator;
use crate::canvas::{canvas2d::Canvas2D, line::Line, matrix::Matrix, vector::Vector};
use crate::ALLOCATOR;
use crate::{print, println, read, write};
use alloc::boxed::Box;
use alloc::slice;
use alloc::string::String;
use alloc::vec::Vec;
use core::arch::global_asm;
use core::cell::RefCell;
use core::time::Duration;

use crate::device::timer::Timer;

use crate::{
    device::sector_device::SectorDevice,
    filesystem::{fat32::FAT32Filesystem, master_boot_record::MasterBootRecord},
};

use super::{
    clock::{self, Clock, ClockState, CLOCKS},
    emmc::{EMMCController, EMMCRegisters},
    framebuffer::{
        Dimensions, FrameBuffer, FrameBufferConfig, FrameBufferConfigBuilder, Offset, Overscan,
        PixelOrder,
    },
    gpio::{GPIOController, OutputLevel, Pin, StatusLight},
    hardware_config::HardwareConfig,
    interrupt::InterruptController,
    mailbox::{Channel, MailboxController},
    platform_devices::{get_platform, PLATFORM},
    power::{Device, PowerState, DEVICES},
};

unsafe extern "C" {
    unsafe static PAGE_SECTION_START: usize;
    unsafe static PAGE_SECTION_SIZE: usize;
}

global_asm!(include_str!("start.s"));

#[no_mangle]
pub extern "C" fn main(heap_start: usize, heap_size: usize, table_start: usize) {
    ALLOCATOR.lock().init(heap_start, heap_size);
    let platform = get_platform();

    platform.init();
    // let status_light = PLATFORM.get_status_light().unwrap();

    // blink_sequence(&status_light.borrow(), timer, 100);

    println!("Starting");

    println!("Entering Boot Sequence (with new build system?)");
    println!("Initializing Memory Virtualization");

    unsafe {
        mmu::init(table_start as *mut usize);
    };

    println!("Memory Virtualization Initialized");

    println!(
        "Heap Allocator initialized at {:#x} with size {}",
        heap_start, heap_size
    );

    let mailbox = platform.get_mailbox_controller();

    let hardware_config = platform.get_hardware_config();

    println!("Hardware Configuration Detected: {}\n", hardware_config);

    println!("Devices:");

    for device in &DEVICES {
        println!(
            "\t-{}: Powered: {}, Timing: {}",
            device,
            device.get_power_state(mailbox).is_on(),
            device.get_timing(mailbox)
        );
    }

    println!("Clocks:");

    for clock in &CLOCKS {
        println!(
            "\t-{}: On: {}, Set Rate: {}, Min Rate: {}, Max Rate: {}",
            clock,
            clock.get_clock_state(mailbox).is_on(),
            clock.get_clock_rate(mailbox),
            clock.get_min_clock_rate(mailbox),
            clock.get_max_clock_rate(mailbox)
        );
    }

    let emmc_controller = PLATFORM.get_emmc_controller();

    let (mbr_sector_number, master_boot_record) =
        MasterBootRecord::scan_device_for_mbr(emmc_controller, 0, 20)
            .expect("Unable to read Master Boot Record");

    let partition = master_boot_record.partition_entries[0];

    let mut filesystem = FAT32Filesystem::load_in_partition(
        emmc_controller,
        mbr_sector_number + partition.first_sector_address(),
        mbr_sector_number + partition.last_sector_address(),
    )
    .expect("Unable to initialize a FAT32 filesystem in partition");

    let root_dir = filesystem.get_root_directory();

    println!("Root directory: {}", root_dir);

    let page_allocator: RefCell<PageAllocator>;

    unsafe {
        let page_start: usize = &PAGE_SECTION_START as *const usize as usize;
        let page_size: usize = &PAGE_SECTION_SIZE as *const usize as usize;
        println!(
            "Initializing Page allocator at {:#x} with size {}",
            page_start, page_size
        );

        page_allocator = RefCell::new(PageAllocator::with_start_and_length(page_start, page_size));
    }

    let kernel = Kernel::with_page_allocator(page_allocator);

    PLATFORM.register_kernel(kernel);

    println!("Enabling IRQs");

    interrupt::enable_irq();

    let mut interrupt_controller = InterruptController::new();

    //interrupt_controller.enable_timer_interrupt_3();
    interrupt_controller.enable_timer_interrupt_1();
    interrupt_controller.enable_auxiliary_device_interrupts();

    println!("Timer interrupt enabled!");

    cpu::create_thread(graphics_thread, String::from("Graphics"), 0);
    for i in 0..20 {
        cpu::create_thread(
            counter_thread,
            String::from(alloc::format!("Counter {}", i)),
            i,
        );
    }

    cpu::create_thread(long_count, String::from("Long Count"), 0);

    PLATFORM.set_kernel_timeout(TICK);

    //status_light.borrow_mut().set_green(OutputLevel::High);

    loop {}
}

pub extern "C" fn long_count(_: usize) {
    let timer = get_platform().get_timer();
    println!("Starting long count");
    loop {
        cpu::sleep(1_000_000);
        println!(
            "Long Count Timer: {:?}",
            Duration::from_micros(timer.get_micros())
        );
    }
}

pub extern "C" fn counter_thread(number: usize) {
    let mut count = 1;
    let mut oops = alloc::vec![];
    interrupt::disable_irq();
    println!("Starting thread: {}", number);
    for i in 0..10 {
        println!("Hello, World! from thread {}. Iteration: {}", number, count);
        count += 1;
        oops.push(i);

        cpu::sleep(200_000);
    }

    println!("Goodbye!");

    cpu::exit_thread();
}

pub extern "C" fn graphics_thread(_arg: usize) {
    let platform = get_platform();
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

    let mut fb = FrameBuffer::from_config(fb_config, platform.get_mailbox_controller());

    println!("Actual config is {}", fb.get_config());

    loop {
        for i in 0..(1920 * 1080) {
            fb.write_idx(i, 0xff00ffff);
        }

        for j in 0..1920 {
            for i in 0..1080 {
                fb.write_pixel(
                    j,
                    i,
                    0xff000000 + ((255 * i / 1080) << 16) + ((255 * j / 1920) << 8) + 0xff,
                );
            }
        }
    }

    println!("Done!");
}

pub fn blink_sequence(status_light: &mut StatusLight, timer: &dyn Timer, interval: u64) {
    status_light.set_green(OutputLevel::High);

    timer.delay_micros(interval);

    status_light.set_green(OutputLevel::Low);
    status_light.set_blue(OutputLevel::High);

    timer.delay_micros(interval);

    status_light.set_blue(OutputLevel::Low);
    status_light.set_red(OutputLevel::High);

    timer.delay_micros(interval);

    status_light.set_red(OutputLevel::Low);
}

pub fn test_allocator(limit: usize) {
    let mut vec_vec: Vec<Vec<usize>> = alloc::vec!();

    for n in 0..limit {
        let num_vec: Vec<usize> = alloc::vec!();
        vec_vec.push(num_vec);
        for m in 0..n {
            vec_vec[n].push(m * n);
        }
    }

    for n in 1..limit {
        for m in 1..n {
            if vec_vec[n][m] != m * n {
                panic!("Expected {:?}, received {:?}", m * n, vec_vec[n][m]);
            }
        }
    }
}
