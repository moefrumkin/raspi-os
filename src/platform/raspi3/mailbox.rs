use core::arch::asm;
use super::mmio::MMIOController;
use alloc::{vec, vec::Vec};
use crate::volatile::{AlignedBuffer, Volatile};

const MBOX_BASE_OFFSET: usize = 0xB880;
const MBOX_READ: usize = MBOX_BASE_OFFSET + 0x0;
#[allow(dead_code)]
const MBOX_POLL: usize = MBOX_BASE_OFFSET + 0x10;
#[allow(dead_code)]
const MBOX_SENDER: usize = MBOX_BASE_OFFSET + 0x14;
const MBOX_STATUS: usize = MBOX_BASE_OFFSET + 0x18;
#[allow(dead_code)]
const MBOX_CONFIG: usize = MBOX_BASE_OFFSET + 0x1c;
const MBOX_WRITE: usize = MBOX_BASE_OFFSET + 0x20;

pub const MBOX_REQUEST: u32 = 0x0;
#[allow(dead_code)]
pub const MBOX_RESPONSE: u32 = 0x80000000;
const MBOX_FULL: u32 = 0x80000000;
const MBOX_EMPTY: u32 = 0x40000000;

pub struct MailboxController<'a> {
    mmio: &'a MMIOController,
}

impl<'a> MailboxController<'a> {
    pub fn new(mmio: &'a MMIOController) -> Self {
        return Self { mmio };
    }

    /// Send the message to the channel and wait for the response.
    /// The lower 4 bits of the message must be 0, otherwise things won't be pretty
    pub fn call(&self, message: u32, channel: Channel) -> u32 {
        //wait there is space to write
        while self.mmio.read_at_offset(MBOX_STATUS) & MBOX_FULL != 0 {
            unsafe {
                asm!("nop");
            }
        }

        self.mmio
            .write_at_offset(message | channel as u32, MBOX_WRITE);

        //loop until the message has a response
        loop {
            //wait until the mailbox is not empty
            while self.mmio.read_at_offset(MBOX_STATUS) & MBOX_EMPTY != 0 {
                unsafe {
                    asm!("nop");
                }
            }

            let response = self.mmio.read_at_offset(MBOX_READ as usize);

            if response & 0b1111 == channel as u32 {
                return response;
            }
        }
    }

    pub fn property_message(&self, buffer: &MailboxBuffer) {
        let addr = buffer.as_ptr();

        self.call(addr as u32, Channel::Prop);
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Channel {
    Power = 0,
    FrameBuffer = 1,
    VUART = 2,
    VCHIQ = 3,
    LEDS = 4,
    BTNS = 5,
    Touch = 6,
    Count = 7,
    Prop = 8,
}


#[repr(C)]
#[repr(align(16))]
#[derive(Debug, Copy, Clone)]
pub struct AlignedWord {
    pub word: u32
}

pub type MailboxBuffer = AlignedBuffer<Volatile<u32>>;
