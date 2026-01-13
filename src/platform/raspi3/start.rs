use super::kernel::Kernel;
use super::kernel::TICK;
use super::programs::ls;
use super::programs::{counter, readelf, write};
use crate::aarch64::interrupt::IRQLock;
use crate::aarch64::{cpu, interrupt, mmu, syscall, syscall::Syscall};
use crate::allocator::page_allocator::PageAllocator;
use crate::canvas::{canvas2d::Canvas2D, line::Line, matrix::Matrix, vector::Vector};
use crate::ALLOCATOR;
use crate::{print, println, read, write};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::slice;
use alloc::string::String;
use alloc::sync::Arc;
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
    unsafe static PAGE_SECTION_SIZE: &'static usize;
}

global_asm!(include_str!("start.s"));

#[no_mangle]
pub extern "C" fn main(heap_start: usize, heap_size: usize) {
    ALLOCATOR.lock().init(heap_start, heap_size);
    let platform = get_platform();

    platform.init();

    println!("Booting");

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

    let filesystem = FAT32Filesystem::load_in_partition(
        emmc_controller,
        mbr_sector_number + partition.first_sector_address(),
        mbr_sector_number + partition.last_sector_address(),
    )
    .expect("Unable to initialize a FAT32 filesystem in partition");

    let page_allocator: IRQLock<PageAllocator>;

    unsafe {
        let page_start: usize = &PAGE_SECTION_START as *const usize as usize;
        let page_size: usize = 6553600;
        println!(
            "Initializing Page allocator at {:#x} with size {}",
            page_start, page_size
        );

        page_allocator = IRQLock::new(PageAllocator::with_start_and_length(page_start, page_size));
    }

    let kernel =
        Kernel::with_page_allocator_and_filesystem(page_allocator, IRQLock::new(filesystem));

    PLATFORM.register_kernel(kernel);

    println!("Enabling IRQs");

    interrupt::enable_irq();

    let mut interrupt_controller = InterruptController::new();

    interrupt_controller.enable_timer_interrupt_1();
    interrupt_controller.enable_auxiliary_device_interrupts();

    println!("Timer interrupt enabled!");

    syscall::create_thread(write::write, String::from("write"), 0);

    PLATFORM.set_kernel_timeout(TICK);

    loop {
        syscall::yield_thread();
    }
}
