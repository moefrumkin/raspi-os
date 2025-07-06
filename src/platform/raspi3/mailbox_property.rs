use crate::platform::raspi3::mailbox::{MailboxController, Channel, MBOX_REQUEST, MailboxBuffer, AlignedWord};
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use crate::volatile::{AlignedBuffer, Volatile};
use crate::platform::raspi3::framebuffer::{PixelOrder, Overscan, Dimensions};

pub struct MessageBuilder<'a> {
    pub instructions: Vec<(&'a mut dyn MailboxInstruction, usize)>,
    pub word_length: usize
}

impl<'a> MessageBuilder<'a> {
    pub fn new() -> Self {
        Self {
            // First element is 0. Second element signifies this is a request We will fill it with the size later
            instructions: Vec::new(),
            word_length: 2 // First element is length, second signifies request
        }
    }

    const TAG_METADATA_WORDS: usize = 3;

    pub fn request(mut self, request: &'a mut dyn MailboxInstruction) -> Self {
        let offset = self.word_length;
        self.word_length += Self::TAG_METADATA_WORDS; // Tag, Size, Req Code
        self.word_length += request.get_buffer_words() as usize;
        self.instructions.push((request, offset));
       self
    }

    pub fn send(&mut self, mailbox: &mut MailboxController) {
        let buffer = self.to_buffer(); 

        mailbox.property_message(&buffer);

        for i in 0..self.instructions.len() {
            let (req, offset) = &mut self.instructions[i];

            let response = MailboxResponse::new(buffer[*offset + 1].get(),
                buffer[*offset + 2].get());

            req.set_response(response);

            let buffer_start = *offset + 3;
            req.read_data_at_offset(&buffer[buffer_start..buffer_start + req.get_buffer_words() as usize]);
        }
    }

    fn to_buffer(&mut self) -> MailboxBuffer {
       let mut buffer: MailboxBuffer = AlignedBuffer::with_length_align(self.word_length, 16);

       // TODO: add padding and end tag at end
       buffer[0].set((4 * self.word_length) as u32);
       buffer[1].set(MBOX_REQUEST);

       for i in 0..self.instructions.len() {
           let (req, offset) = &self.instructions[i];
           buffer[*offset].set(req.get_encoding());
           buffer[offset + 1].set(req.get_buffer_bytes());
           buffer[offset + 2].set(0);

           let buffer_start = *offset + 3;
           req.write_data_at_offset(&mut buffer[buffer_start..buffer_start + req.get_buffer_words() as usize]);
       }

       buffer
    }
}

#[derive(Copy, Clone)]
pub struct MailboxResponse {
    code: u32,
    size: u32
}

impl MailboxResponse {
    pub fn new(code: u32, size: u32) -> Self {
        Self { code, size }
    }

    pub fn get_code(&self) -> u32 {
        self.code
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

    // TODO: find a better way of representing and empty response
    pub fn none() -> Self {
        Self { code: 0, size : 0 }
    }
}

pub type MailboxBufferSlice = [Volatile<u32>];

pub trait MailboxInstruction {
    fn get_encoding(&self) -> u32;

    fn get_buffer_bytes(&self) -> u32 {
        4 * self.get_buffer_words()
    }

    fn get_buffer_words(&self) -> u32;

    fn write_data_at_offset(&self,
        #[allow(unused_variables)] buffer: &mut MailboxBufferSlice) {
    } // TODO: is it ok to not initialize the buffer on requests with no data?

    fn read_data_at_offset(&mut self,
        #[allow(unused_variables)] buffer: &MailboxBufferSlice) {
    }

    fn set_response(&mut self, #[allow(unused_variables)] response: MailboxResponse) {
    }
}

pub trait ToMailboxBuffer {
    fn write_to_mailbox_buffer(&self, buffer: &mut MailboxBufferSlice);
}

pub trait FromMailboxBuffer {
    fn read_from_mailbox_buffer(buffer: &MailboxBufferSlice) -> Self;
}

pub struct SimpleRequest<T, U, const E: u32>
where
    T: ToMailboxBuffer + Copy,
    U: FromMailboxBuffer + Copy
{
    request: T,
    response: Option<U>
}

impl<T, U, const E: u32> SimpleRequest<T, U, E>
where
    T: ToMailboxBuffer + Copy,
    U: FromMailboxBuffer + Copy
{
    pub fn with_request(request: T) -> Self {
        Self {
            request,
            response: Option::None
        }
    }

    pub fn get_response(&self) -> U {
        self.response.unwrap()
    }
}

impl<T, U, const E: u32> MailboxInstruction for SimpleRequest<T, U, E>
where
    T: ToMailboxBuffer + Copy,
    U: FromMailboxBuffer + Copy
{
    fn get_encoding(&self) -> u32 {
        E
    }

    // Maximum size of request and response type
    fn get_buffer_words(&self) -> u32 {
        let bytes = core::cmp::max(core::mem::size_of::<T>(), core::mem::size_of::<U>());

        bytes.div_ceil(4) as u32
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBufferSlice) {
        self.request.write_to_mailbox_buffer(buffer);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.response = Option::Some(U::read_from_mailbox_buffer(buffer));
    }
}

pub struct GetARMMemory {
    pub base: u32,
    pub size: u32
}

impl GetARMMemory {
    pub fn new() -> Self {
        Self {
            base: 0,
            size: 0
        }
    }

    pub fn get_base(&self) -> u32 {
        self.base
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }
}

impl MailboxInstruction for GetARMMemory {
    fn get_encoding(&self) -> u32 {
        0x10005
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.base = buffer[0].get();
        self.size = buffer[1].get();
    }
}

pub struct GetVCMemory {
    pub base: u32,
    pub size: u32
}

impl GetVCMemory {
    pub fn new() -> Self {
        Self {
            base: 0,
            size: 0
        }
    }

    pub fn get_base(&self) -> u32 {
        self.base
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }
}

impl MailboxInstruction for GetVCMemory {
    fn get_encoding(&self) -> u32 {
        0x10006
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.base = buffer[0].get();
        self.size = buffer[1].get();
    }
}

pub struct GetFrameBuffer {
    pub alignment: u32,
    pub start: u32,
    pub size: u32,
    pub response: MailboxResponse
}

impl GetFrameBuffer {
    pub fn with_aligment(alignment: u32) -> Self {
        Self {
            alignment,
            start: 0,
            size: 0,
            response: MailboxResponse::none()
        }
    }

    pub fn get_start(&self) -> u32 {
        self.start
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }
}

impl MailboxInstruction for GetFrameBuffer {
    fn get_encoding(&self) -> u32 {
        0x40001
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBufferSlice) {
        buffer[0].set(self.alignment);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.start = buffer[0].get();
        self.size = buffer[1].get();
    }

    fn set_response(&mut self, response: MailboxResponse) {
        self.response = response;
    }
}

pub struct ReleaseBuffer {
}

impl ReleaseBuffer {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl MailboxInstruction for ReleaseBuffer {
    fn get_encoding(&self) -> u32 {
        0x48001
    }

    fn get_buffer_words(&self) -> u32 {
        0
    }
}

pub struct BlankScreen {
    state: bool
}

impl BlankScreen {
    pub fn new(state: bool) -> Self {
        Self {
            state
        }
    }
}

impl MailboxInstruction for BlankScreen {
    fn get_encoding(&self) -> u32 {
        0x40002
    }

    fn get_buffer_words(&self) -> u32 {
        1 
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBufferSlice) {
        let word = if self.state {0x1} else {0x0};

        buffer[0].set(word);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        let word = buffer[0].get();
        self.state = if word == 0 {false} else {true};
    }
}


pub struct GetPhysicalDimensions {
    pub width: u32,
    pub height: u32
}

impl GetPhysicalDimensions {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get(&self) -> Dimensions {
        Dimensions::new(self.width, self.height)
    }
}

impl MailboxInstruction for GetPhysicalDimensions {
    fn get_encoding(&self) -> u32 {
        0x40003
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.width = buffer[0].get();
        self.height = buffer[1].get();
    }
}

pub struct SetPhysicalDimensions {
    pub dimensions: Dimensions,
}

impl SetPhysicalDimensions {
    pub fn new(dimensions: Dimensions) -> Self {
        Self {
            dimensions
        }
    }

    pub fn get_width(&self) -> u32 {
        self.dimensions.get_width()
    }

    pub fn get_height(&self) -> u32 {
        self.dimensions.get_height()
    }

    pub fn get(&self) -> Dimensions {
        self.dimensions
    }
}

impl MailboxInstruction for SetPhysicalDimensions {
    fn get_encoding(&self) -> u32 {
        0x48003
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBufferSlice) {
        buffer[0].set(self.dimensions.get_width());
        buffer[1].set(self.dimensions.get_height());
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.dimensions.set_width(buffer[0].get());
        self.dimensions.set_height(buffer[1].get());
    }
}

pub struct GetVirtualDimensions {
    pub width: u32,
    pub height: u32
}

impl GetVirtualDimensions {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get(&self) -> Dimensions {
        Dimensions::new(self.width, self.height)
    }
}

impl MailboxInstruction for GetVirtualDimensions {
    fn get_encoding(&self) -> u32 {
        0x40004
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.width = buffer[0].get();
        self.height = buffer[0].get();
    }
}

pub struct SetVirtualDimensions {
    dimensions: Dimensions
}

impl SetVirtualDimensions {
    pub fn new(dimensions: Dimensions) -> Self {
        Self {
            dimensions
        }
    }

    pub fn get_width(&self) -> u32 {
        self.dimensions.get_width()
    }

    pub fn get_height(&self) -> u32 {
        self.dimensions.get_height()
    }

    pub fn get(&self) -> Dimensions {
        self.dimensions
    }
}

impl MailboxInstruction for SetVirtualDimensions {
    fn get_encoding(&self) -> u32 {
        0x48004
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBufferSlice) {
        buffer[0].set(self.dimensions.get_width());
        buffer[1].set(self.dimensions.get_height());
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.dimensions.set_width(buffer[0].get());
        self.dimensions.set_height(buffer[1].get());
    }
}


pub struct GetDepth {
    pub depth: u32
}

impl GetDepth {
    pub fn new() -> Self {
        Self {
            depth: 0
        }
    }

    pub fn get_depth(&self) -> u32 {
        self.depth
    }
}

impl MailboxInstruction for GetDepth {
    fn get_encoding(&self) -> u32 {
        0x40005
    }

    fn get_buffer_words(&self) -> u32 {
        1
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.depth = buffer[0].get();
    }
}

pub struct SetDepth {
    pub depth: u32
}

impl SetDepth {
    pub fn new(depth: u32) -> Self {
        Self {
            depth
        }
    }

    pub fn get_depth(&self) -> u32 {
        self.depth
    }

    pub fn get(&self) -> u32 {
        self.depth 
    }
}

impl MailboxInstruction for SetDepth {
    fn get_encoding(&self) -> u32 {
        0x48005
    }

    fn get_buffer_words(&self) -> u32 {
        1
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBufferSlice) {
        buffer[0].set(self.depth);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.depth = buffer[0].get();
    }
}

pub struct GetPitch {
    pub pitch: u32
}

impl GetPitch {
    pub fn new() -> Self {
        Self {
            pitch: 0
        }
    }

    pub fn get_pitch(&self) -> u32 {
        self.pitch
    }

    pub fn get(&self) -> u32 {
        self.pitch
    }
}

impl MailboxInstruction for GetPitch {
    fn get_encoding(&self) -> u32 {
        0x40008
    }

    fn get_buffer_words(&self) -> u32 {
        1
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.pitch = buffer[0].get();
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
