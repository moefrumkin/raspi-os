use core::arch::asm;
use super::mmio::MMIOController;
use alloc::{vec, vec::Vec};

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

const MBOX_REQUEST: u32 = 0x0;
#[allow(dead_code)]
const MBOX_RESPONSE: u32 = 0x80000000;
const MBOX_FULL: u32 = 0x80000000;
const MBOX_EMPTY: u32 = 0x40000000;

type Message<'a> = &'a [u32];

pub struct MessageBuilder {
    message: Vec<u32>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            message: vec![0, MBOX_REQUEST],
        }
    }

    pub fn instruction(mut self, instruction: Instruction, length: u32) -> Self {
        self.message.push(instruction as u32);
        self.message.push(length);
        self.message.push(length);
        self
    }

    pub fn data(mut self, data: u32) -> Self {
        self.message.push(data);
        self
    }

    pub fn send(mut self, mailbox: &mut MailboxController) -> Vec<u32> {
        self.message[0] = (4 * (self.message.len() - 1)) as u32;
        for (i, value) in self.message.iter().enumerate() {
            mailbox.buffer.data[i] = *value;
        }
        mailbox.call(mailbox.buffer as *const MessageBuffer as u32, Channel::Prop);
        mailbox.buffer.data.to_vec()
    }
}

pub struct MailboxController<'a> {
    mmio: &'a MMIOController,
    buffer: &'a mut MessageBuffer,
}

impl<'a> MailboxController<'a> {
    pub fn new(mmio: &'a MMIOController, buffer: &'a mut MessageBuffer) -> Self {
        return Self { mmio, buffer };
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
#[allow(dead_code)]
pub struct MessageBuffer {
    pub data: [u32; MessageBuffer::BUFFER_LENGTH],
}

impl MessageBuffer {
    const BUFFER_LENGTH: usize = 256;

    pub fn new() -> Self {
        Self {
            data: [0; Self::BUFFER_LENGTH],
        }
    }
}

pub enum Instruction {
    GetFirmwareRevision = 0x1,

    GetBoardModel = 0x10001,
    GetBoardRevision = 0x10002,
    GetBoardMAC = 0x10003,
    GetBoardSerial = 0x10004,
    GetARMMemory = 0x10005,
    GetVCMemory = 0x10006,
    GetClocks = 0x10007,

    GetCommandLine = 0x50001,

    GetDMAChannels = 0x60001,

    GetPowerState = 0x20001,
    GetTiming = 0x20002,
    SetPowerState = 0x28001,

    GetClockState = 0x30001,
    SetClockState = 0x38001,
    GetClockRate = 0x30002,
    GetLEDStatus = 0x30041,
    TestLEDStatus = 0x34041,
    SetLEDStatus = 0x38041,
    GetMeasuredClock = 0x30047,
    SetClockRate = 0x38002,
    GetMaxClockRate = 0x30004,
    GetMinClockRate = 0x30007,
    GetTurbo = 0x30009,
    SetTurbo = 0x38009,

    GetVoltage = 0x30003,
    SetVoltage = 0x38003,
    GetMaxVoltage = 0x30005,
    GetMinVoltage = 0x30008,

    GetTemperature = 0x30006,
    GetMaxTemperature = 0x3000a,

    AllocateMemory = 0x3000c,
    LockMemory = 0x3000d,
    UnlockMemory = 0x3000e,
    ReleaseMemory = 0x3000f,

    ExecuteCode = 0x30010,

    GetDispmanxResourceHandle = 0x30014,

    GetEDIDBlock = 0x30020,

    AllocateBuffer = 0x40001,
    ReleaseBuffer = 0x48001,

    BlankScreen = 0x40002,

    GetPhysicalDimensions = 0x40003,
    TestPhysicalDimensions = 0x44003,
    SetPhysicalDimensions = 0x48003,

    GetVirtualDimensions = 0x40004,
    TestVirtualDimensions = 0x44004,
    SetVirtualDimensions = 0x48004,

    GetDepth = 0x40005,
    TestDepth = 0x44005,
    SetDepth = 0x48005,

    GetPixelOrder = 0x40006,
    TestPixelOrder = 0x44006,
    SetPixelOrder = 0x48006,

    GetAlphaMode = 0x40007,
    TestAlphaMode = 0x44007,
    SetAlphaMode = 0x48007,

    GetPitch = 0x40008,

    GetVirtualOffset = 0x40009,
    TestVirtualOffset = 0x44009,
    SetVirtualOffset = 0x48009,

    GetOverscan = 0x4000a,
    TestOverscan = 0x4400a,
    SetOverScan = 0x4800a,

    GetPalette = 0x4000b,
    TestPalette = 0x4400b,
    SetPalette = 0x4800b,

    SetCursorInfo = 0x8010,
    SetCursorState = 0x8011,
}
