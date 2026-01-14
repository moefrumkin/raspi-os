use super::{
    mailbox::MailboxController,
    mailbox_property::{
        FromMailboxBuffer, MailboxBufferSlice, MessageBuilder, SimpleRequest, ToMailboxBuffer,
    },
};
use crate::bitfield;
use core::fmt;

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
    PixelBVB = 0xe,
}

pub const CLOCKS: [Clock; 14] = [
    Clock::EMMC,
    Clock::UART,
    Clock::ARM,
    Clock::CORE,
    Clock::V3D,
    Clock::H264,
    Clock::ISP,
    Clock::SDRAM,
    Clock::PIXEL,
    Clock::PWM,
    Clock::HEVC,
    Clock::EMMC2,
    Clock::M2MC,
    Clock::PixelBVB,
];

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Clock::EMMC => "EMMC",
                Clock::UART => "UART",
                Clock::ARM => "ARM",
                Clock::CORE => "CORE",
                Clock::V3D => "V3D",
                Clock::H264 => "H264",
                Clock::ISP => "ISP",
                Clock::SDRAM => "SDRAM",
                Clock::PIXEL => "PIXEL",
                Clock::PWM => "PWM",
                Clock::HEVC => "HEVC",
                Clock::EMMC2 => "EMMC2",
                Clock::M2MC => "M2MC",
                Clock::PixelBVB => "PIXEL BVB",
            }
        )
    }
}

impl Clock {
    const GET_CLOCK_STATE: u32 = 0x30001;
    const GET_CLOCK_RATE: u32 = 0x30002;
    const GET_CLOCK_RATE_MEASURED: u32 = 0x30047;
    const GET_MAX_CLOCK_RATE: u32 = 0x30004;
    const GET_MIN_CLOCK_RATE: u32 = 0x30007;

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
            0xe => Clock::PixelBVB,
            _ => panic!("Invalid Clock id"),
        }
    }

    // TODO: find some abstraction for these functions
    pub fn get_clock_rate(self, mailbox: &dyn MailboxController) -> u32 {
        let mut request =
            SimpleRequest::<Clock, ClockRateResponse, { Self::GET_CLOCK_RATE }>::with_request(self);

        let mut message = MessageBuilder::new().request(&mut request);

        message.send(mailbox);

        return request.get_response().1;
    }

    pub fn get_clock_rate_measured(self, mailbox: &dyn MailboxController) -> u32 {
        let mut request = SimpleRequest::<
            Clock,
            ClockRateResponse,
            { Self::GET_CLOCK_RATE_MEASURED },
        >::with_request(self);

        let mut message = MessageBuilder::new().request(&mut request);

        message.send(mailbox);

        return request.get_response().1;
    }

    pub fn get_max_clock_rate(self, mailbox: &dyn MailboxController) -> u32 {
        let mut request =
            SimpleRequest::<Clock, ClockRateResponse, { Self::GET_MAX_CLOCK_RATE }>::with_request(
                self,
            );

        let mut message = MessageBuilder::new().request(&mut request);

        message.send(mailbox);

        return request.get_response().1;
    }

    pub fn get_min_clock_rate(self, mailbox: &dyn MailboxController) -> u32 {
        let mut request =
            SimpleRequest::<Clock, ClockRateResponse, { Self::GET_MIN_CLOCK_RATE }>::with_request(
                self,
            );

        let mut message = MessageBuilder::new().request(&mut request);

        message.send(mailbox);

        return request.get_response().1;
    }

    pub fn get_clock_state(self, mailbox: &dyn MailboxController) -> ClockState {
        let mut request =
            SimpleRequest::<Clock, ClockStateResponse, { Self::GET_CLOCK_STATE }>::with_request(
                self,
            );

        let mut message = MessageBuilder::new().request(&mut request);

        message.send(mailbox);

        return request.get_response().1;
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
        pub fn from_u32(value: u32) -> Self {
            Self {
                value
            }
        }

        pub fn is_on(&self) -> bool {
            self.get_on() == 1
        }

        pub fn exists(&self) -> bool {
            self.get_exists() == 1
        }
    }
}

#[derive(Copy, Clone)]
struct ClockStateResponse(Clock, ClockState);

impl FromMailboxBuffer for ClockStateResponse {
    fn read_from_mailbox_buffer(buffer: &MailboxBufferSlice) -> Self {
        Self(
            Clock::from_u32(buffer[0].get()),
            ClockState::from_u32(buffer[1].get()),
        )
    }
}
