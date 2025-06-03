use crate::platform::raspi3::mailbox::{MailboxController, Channel, MBOX_REQUEST, MailboxBuffer, AlignedWord};
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;

pub struct MessageBuilder<'a> {
    pub instructions: Vec<(&'a mut dyn MailboxInstruction, u32)>,
    pub word_length: u32
}

#[derive(Copy, Clone)]
pub enum MessageWord {
    Value(u32),
    Tag(u32),
    Padding,
}

impl MessageWord {
    pub fn to_u32(self) -> u32 {
        match self {
            MessageWord::Value(number) => number,
            MessageWord::Tag(encoding) => encoding,
            MessageWord::Padding => 0
        }
    }
}

impl<'a> MessageBuilder<'a> {
    pub fn new() -> Self {
        Self {
            // First element is 0. Second element signifies this is a request We will fill it with the size later
            instructions: Vec::new(),
            word_length: 2 // First element is length, second signifies request
        }
    }

    const TAG_METADATA_WORDS: u32 = 3;

    pub fn request(mut self, request: &'a mut dyn MailboxInstruction) -> Self {
        let offset = self.word_length;
        self.word_length += Self::TAG_METADATA_WORDS; // Tag, Size, Req Code
        self.word_length += request.get_buffer_words();
        self.instructions.push((request, offset));
       self
    }

    pub fn send(&mut self, mailbox: &mut MailboxController) {
        let buffer = self.to_buffer(); 

        buffer.send(mailbox);

        for i in 0..self.instructions.len() {
            let (req, offset) = &mut self.instructions[i];

            req.read_data_at_offset(&buffer, *offset + 3);
        }
    }

    pub fn to_buffer(&mut self) -> MailboxBuffer {
       let mut buffer = MailboxBuffer::with_capacity(self.word_length as usize);

       // TODO: add padding at end
       buffer.write(0, 4 * self.word_length);
       buffer.write(1, MBOX_REQUEST);

       for i in 0..self.instructions.len() {
           let (req, offset) = &self.instructions[i];
           buffer.write(*offset as isize, req.get_encoding());
           buffer.write((offset + 1) as isize, req.get_buffer_bytes());
           buffer.write((offset + 2) as isize, 0);
           req.write_data_at_offset(&mut buffer, offset + 3);
       }

       buffer
    }
}

pub trait MailboxInstruction {
    fn get_encoding(&self) -> u32;

    fn get_buffer_bytes(&self) -> u32;

    fn get_buffer_words(&self) -> u32;

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32);

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32);
}


pub struct GetBoardRevision {
    pub revision: u32
}

impl GetBoardRevision {
    pub fn new() -> Self {
        Self {
            revision: 0
        }
    }

    pub fn get_response(&self) -> u32 {
        self.revision
    }
}

impl MailboxInstruction for GetBoardRevision {
    fn get_encoding(&self) -> u32 {
        0x10002
    }

    fn get_buffer_bytes(&self) -> u32 {
        4
    }

    fn get_buffer_words(&self) -> u32 {
        1
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
        buffer.write(offset as isize, 0);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        self.revision = buffer.read(offset as isize);
    }
}

#[derive(Copy, Clone)]
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
