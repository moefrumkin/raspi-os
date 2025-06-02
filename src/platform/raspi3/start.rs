use core::arch::global_asm;
use crate::aarch64::{cpu, mmu};
use crate::canvas::{canvas2d::Canvas2D, line::Line, matrix::Matrix, vector::Vector};
use crate::ALLOCATOR;
use crate::{print, println, read, write};
use alloc::vec::Vec;
use alloc::slice;

use super::{
    gpio::{GPIOController, OutputLevel, Pin, StatusLight},
    gpu::{FBConfig, GPUController},
    lcd::LCDController,
    mailbox::{Channel, MailboxController},
    mmio::MMIOController,
    timer::Timer,
    uart::{LogLevel, UARTController, CONSOLE},
    mailbox_property::{MessageBuilder, MessageWord, Instruction, AlignedWord}
};

static MMIO: MMIOController = MMIOController::new();
static GPIO: GPIOController = GPIOController::new(&MMIO);

global_asm!(include_str!("start.s"));

#[no_mangle]
pub extern "C" fn main(heap_start: usize, heap_size: usize, mailbox_start: usize, table_start: usize) {
    let mmio = MMIOController::default();
    let gpio = GPIOController::new(&mmio);
    let timer = Timer::new(&mmio);
    
    let status_light = StatusLight::init(&gpio);

    blink_sequence(&status_light, &timer, 100);

    let mut console = UARTController::init(&GPIO, &MMIO);
    console.set_log_level(LogLevel::Debug);

    unsafe {
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

    let mailbox_start = 0x95000;
    let buffer = unsafe {
        slice::from_raw_parts_mut(mailbox_start as *mut u32, 8)
    };

    println!("Initializing mailbox buffer at {:#x}", mailbox_start);

    buffer[0] = 32;
    buffer[1] = 0;

    buffer[2] = 0x10002;
    buffer[3] = 4;
    buffer[4] = 0;
    buffer[5] = 0;

    buffer[6] = 0;
    buffer[7] = 0;
    
    let mut mailbox = MailboxController::new(&mmio);

    let mut message = MessageBuilder::new()
        .push(MessageWord::data(0x10002))
        .push(MessageWord::data(4))
        .push(MessageWord::data(0))
        .push(MessageWord::data(0))
        .push(MessageWord::data(0))
        .push(MessageWord::data(0));

    println!("Sending mailbox message");

    println!("Size of Aligned is {}", core::mem::size_of::<AlignedWord>());

    let mut mbuf = message.to_buffer();

    let mbuf_start = mbuf.start();

    let mbuf_raw = unsafe {
        slice::from_raw_parts_mut(mbuf_start as *mut u32, 8)
    };

    println!("mbox: {:#x}, mbuf: {:#x}", mailbox_start, mbuf_start);

    for i in 0..8 {
        println!("buffer[{}] = {:#x}, mbuf[{}] = {:#x}", i, buffer[i], i, mbuf_raw[i]);
    }

    mbuf.send(&mut mailbox);
    mailbox.call(mailbox_start as u32, Channel::Prop);

    println!("Message sent!");

    for i in 0..8 {
        println!("buffer[{}] = {:#x}, mbuf[{}] = {:#x}", i, buffer[i], i, mbuf_raw[i]);
    }


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
        let mut num_vec: Vec<usize> = alloc::vec!();
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
