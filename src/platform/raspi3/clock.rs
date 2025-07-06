use crate::bitfield;
use super::{
    mailbox::MailboxController,
    mailbox_property::{
        MessageBuilder,
        SimpleRequest,
        ToMailboxBuffer,
        FromMailboxBuffer,
        MailboxBufferSlice,
        MailboxInstruction
    }
};

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum Clock {
    EMMC = 0x1,
    UART = 0x2,
    ARM = 0x3,
    CORE = 0x4,
    V3D = 0x5,
    H264 = 0x6,
    ISP = 0x7,
    SDRAM = 0x8,
    PIXEL = 0x9,
    PWM = 0xa,
    HEVC = 0xb,
    EMMC2 = 0xc,
    M2MC = 0xd,
    PIXEL_BVB = 0xe
}

impl Clock {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }

    pub fn from_u32(val: u32) -> Clock {
        match val {
            0x1 => Clock::EMMC,
            0x2 => Clock::UART,
            0x3 => Clock::ARM,
            0x4 => Clock::CORE,
            0x5 => Clock::V3D,
            0x6 => Clock::H264,
            0x7 => Clock::ISP,
            0x8 => Clock::SDRAM,
            0x9 => Clock::PIXEL,
            0xa => Clock::PWM,
            0xb => Clock::HEVC,
            0xc => Clock::EMMC2,
            0xd => Clock::M2MC,
            0xe => Clock::PIXEL_BVB,
            _ => panic!("Invalid Clock id")
        }
    }
}

impl ToMailboxBuffer for Clock {
    fn write_to_mailbox_buffer(&self, buffer: &mut MailboxBufferSlice) {
        buffer[0].set(self.as_u32());
    }
}

#[derive(Copy, Clone)]
struct ClockRateResponse(Clock, u32);

impl FromMailboxBuffer for ClockRateResponse {
    fn read_from_mailbox_buffer(buffer: &MailboxBufferSlice) -> Self {
        Self(Clock::from_u32(buffer[0].get()), buffer[1].get())
    }
}

bitfield! {
    ClockState(u32) {
        on: 0-0,
        exists: 1-1 
    } with {
        pub fn is_on(&self) -> bool {
            self.get_on() == 1
        }

        pub fn exists(&self) -> bool {
            self.get_exists() == 1
        }
    }
}

pub fn get_clock_rate(mailbox: &mut MailboxController, clock: Clock) -> u32 {
    let mut request = SimpleRequest::<Clock, ClockRateResponse, 0x30002>::with_request(clock);

    let mut message = MessageBuilder::new().request(&mut request);

    message.send(mailbox);

    return request.get_response().1;
}
