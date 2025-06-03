use crate::platform::raspi3::mailbox::{MailboxController, Channel, MBOX_REQUEST, MailboxBuffer, AlignedWord};
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

    pub fn instruction(mut self, instruction: Instruction) -> Self {
        let details = instruction.get_details();
        self.message.push(MessageWord::data(details.encoding));
        self.message.push(MessageWord::data(details.response_len));
        self.message.push(MessageWord::data(0)); // Request code
        for i in 0 .. details.response_len/4 {
            self.message.push(MessageWord::data(0));
        }
        self
    }

    pub fn data(mut self, data: u32) -> Self {
        self.message.push(MessageWord::data(data));
        self
    }

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

pub struct InstructionDetails {
    pub encoding: u32,
    // Lengths of the request and response data in bytes
    pub request_len: u32,
    pub response_len: u32
}

impl InstructionDetails {
    pub fn new(encoding: u32, request_len: u32, response_len: u32) -> Self {
        Self {
            encoding,
            request_len,
            response_len
        }
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

impl Instruction {
    pub fn get_details(self) -> InstructionDetails {
        match self {
            Instruction::GetFirmwareRevision => InstructionDetails::new(0x1, 0, 4),

            Instruction::GetBoardModel => InstructionDetails::new(0x10001, 0, 4),
            Instruction::GetBoardRevision => InstructionDetails::new(0x10002, 0, 4),
            Instruction::GetBoardMAC => InstructionDetails::new(0x10003, 0, 6),
            Instruction::GetBoardSerial => InstructionDetails::new(0x10004, 0, 8),
            Instruction::GetARMMemory => InstructionDetails::new(0x10005, 0, 8),
            Instruction::GetVCMemory => InstructionDetails::new(0x10006, 0, 8),
            Instruction::GetClocks => InstructionDetails::new(0x10007, 0, 8), // This response
                                                                              // length is
                                                                              // variable. What to
                                                                             // do?
            _ => unimplemented!()

            /*GetCommandLine = 0x50001,

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
            SetCursorState = 0x8011, */

        }
    }
}
