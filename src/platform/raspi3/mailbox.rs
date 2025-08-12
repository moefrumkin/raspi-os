use core::{arch::asm, cell::RefCell};
use alloc::{vec, vec::Vec};
use crate::{bitfield, volatile::{AlignedBuffer, Volatile}};
use alloc::rc::Rc;

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

#[repr(C)]
pub struct MailboxRegisters {
    read: Volatile<u32>,
    res0: [u32; 5],
    status: Volatile<MailboxStatus>,
    res1: [u32; 1],
    write: Volatile<MailboxWriteData>
}

bitfield! {
    MailboxStatus(u32) {
        is_empty: 30-30,
        is_full: 31-31
    }
}

bitfield! {
    MailboxWriteData(u32) {
        channel: 0-3,
        data: 4-31
    }
}

pub struct MailboxController<'a> {
    registers: &'a mut MailboxRegisters
}

impl<'a> MailboxController<'a> {
    pub fn with_registers(registers: &'a mut MailboxRegisters) -> Self {
        return Self { registers };
    }

    /// Send the message to the channel and wait for the response.
    /// The lower 4 bits of the message must be 0, otherwise things won't be pretty
    pub fn call(&mut self, message: u32, channel: Channel) -> u32 {
        //wait there is space to write
        while self.registers.status.get().get_is_full() == 1 {
            unsafe {
                asm!("nop");
            }
        }

        self.registers.write.map_closure(&|write|
            write.set_channel(channel as u32).set_data(message)
        );

        //loop until the message has a response
        loop {
            //wait until the mailbox is not empty
            while self.registers.status.get().get_is_empty() == 1 {
                unsafe {
                    asm!("nop");
                }
            }

            let response = self.registers.read.get();

            // TODO get rid of magic number
            if response & 0b1111 == channel as u32 {
                return response;
            }
        }
    }

    pub fn property_message(&mut self, buffer: &MailboxBuffer) {
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
