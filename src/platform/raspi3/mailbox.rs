use crate::{
    aarch64::cpu,
    bitfield,
    volatile::{AlignedBuffer, Volatile},
};

pub trait MailboxController {
    // TODO: Should buffer be mutable since it is modified by the call?
    fn send_message_on_channel(&self, buffer: &MailboxBuffer, channel: Channel) -> u32;

    fn send_property_message(&self, buffer: &MailboxBuffer) {
        self.send_message_on_channel(buffer, Channel::Prop);
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct MailboxRegisters {
    read: Volatile<u32>,
    res0: [u32; 5],
    status: Volatile<MailboxStatus>,
    res1: [u32; 1],
    write: Volatile<MailboxWriteData>,
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

impl MailboxRegisters {
    pub fn send_message(&mut self, message: u32, channel: Channel) -> u32 {
        self.wait_until_not_full();

        self.write.map_closure(&|write|
            // TODO: find better way instead of bit shifting data
            write.set_channel(channel as u32).set_data(message >> 4));

        loop {
            self.wait_until_not_empty();

            let response = self.read.get();

            if response & 0b1111 == channel as u32 {
                return response;
            }
        }
    }

    fn wait_until_not_empty(&self) {
        while self.status.get().get_is_empty() == 1 {
            //Why do we need a no op?
            cpu::nop();
        }
    }

    fn wait_until_not_full(&self) {
        while self.status.get().get_is_full() == 1 {
            cpu::nop();
        }
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
    pub word: u32,
}

pub type MailboxBuffer = AlignedBuffer<Volatile<u32>>;
