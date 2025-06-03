use crate::platform::raspi3::mailbox::{MailboxController, Channel, MBOX_REQUEST};
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;

pub struct MessageBuilder {
    pub message: Vec<MessageWord>
}

#[derive(Copy, Clone)]
pub enum MessageWord {
    data(u32),
    tag(Instruction)
}

impl MessageWord {
    pub fn to_u32(self) -> u32 {
        match self {
            MessageWord::data(number) => number,
            MessageWord::tag(instruction) => instruction as u32
        }
    }
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            // First element is 0. Second element signifies this is a request We will fill it with the size later
            message: vec![MessageWord::data(0), MessageWord::data(MBOX_REQUEST)],
        }
    }

    pub fn push(mut self, word: MessageWord) -> Self {
        self.message.push(word);
        self
    }

    /*pub fn instruction(mut self, instruction: Instruction, length: u32) -> Self {
        self.message.push(instruction as u32);
        self.message.push(length);
        self.message.push(length);
        self
    }

    pub fn data(mut self, data: u32) -> Self {
        self.message.push(data);
        self
    }*/

    fn format(&mut self) {
        let size = 4 * self.message.len();
        self.message[0] = MessageWord::data(size as u32);
    }

    pub fn send(&mut self, mailbox: &mut MailboxController) -> MailboxBuffer {
        let buffer = self.to_buffer(); 

        buffer.send(mailbox);

        buffer
    }

    pub fn to_buffer(&mut self) -> MailboxBuffer {
       self.format();
       let ptr = vec![AlignedWord { word: 0 }; 4 * self.message.len()].into_boxed_slice().as_mut_ptr();
       let mut buffer = MailboxBuffer{
           buffer: ptr as *mut u32
       };

       for i in 0..self.message.len() {
           buffer.write(i as isize, self.message[i].to_u32());
       }

       buffer
    }
}

#[repr(C)]
#[repr(align(16))]
#[derive(Debug, Copy, Clone)]
pub struct AlignedWord {
    word: u32
}

pub struct MailboxBuffer {
    buffer: *mut u32
}

impl MailboxBuffer {
    pub fn with_capacity(capacity: usize) -> Self {
        // TODO: this is terrible
        let vec: Vec<AlignedWord> = vec![AlignedWord { word: 0}; capacity];
        let ptr = vec.into_boxed_slice().as_ptr() as usize;

        Self {
            buffer: unsafe { ptr as *mut u32 }
        }
    }

    pub fn send(&self, mailbox: &mut MailboxController) {
        let addr = self.buffer;

        mailbox.call(addr as u32, Channel::Prop);
    }

    pub fn write(&mut self, offset: isize, word: u32) {
        unsafe {
            core::ptr::write_volatile(self.buffer.offset(offset), word);
        }
    }

    pub fn read(&mut self, offset: isize) -> u32 {
        unsafe {
            core::ptr::read_volatile(self.buffer.offset(offset) as *const u32)
        }
    }

    pub fn start(&self) -> u32 {
        let addr = self.buffer;
        addr as u32
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
