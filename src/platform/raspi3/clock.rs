use crate::bitfield;
use crate::platform::raspi3::mailbox_property::{
    MessageBuilder,
    SimpleRequest,
    ToMailboxBuffer,
    FromMailboxBuffer,
};


#[repr(u32)]
pub enum ClockId {
    EMMC = 0x1,
    UART = 0x2,
    ARM = 0x3,
    CORE = 0x4,
    V3D = 0x5,
    H264 = 0x6,
    ISP = 0x8,
    PIXEL = 0x9,
    PWM = 0xa,
    HEVC = 0xb,
    M2MC = 0xd,
    PIXEL_BVB = 0xe
}

bitfield! {
    ClockState(u32) {
        on: 0-0,
        exists: 1-1 
    }
}

