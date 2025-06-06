use crate::platform::raspi3::mailbox::{MailboxController, Channel, MBOX_REQUEST, MailboxBuffer, AlignedWord};
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;

pub struct MessageBuilder<'a> {
    pub instructions: Vec<(&'a mut dyn MailboxInstruction, u32)>,
    pub word_length: u32
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

            let response = MailboxResponse::new(buffer.read((*offset + 1) as isize),
                buffer.read((*offset + 2) as isize));

            req.set_response(response);
            req.read_data_at_offset(&buffer, *offset + 3);
        }
    }

    pub fn to_buffer(&mut self) -> MailboxBuffer {
       let mut buffer = MailboxBuffer::with_capacity(self.word_length as usize);

       // TODO: add padding and end tag at end
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

pub trait MailboxInstruction {
    fn get_encoding(&self) -> u32;

    fn get_buffer_bytes(&self) -> u32 {
        4 * self.get_buffer_words()
    }

    fn get_buffer_words(&self) -> u32;

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
    } // TODO: is it ok to not initialize the buffer on requests with not data?

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32);

    fn set_response(&mut self, response: MailboxResponse) {
    }
}

pub struct SimpleRequest {
    pub encoding: u32,
    pub response: u32
}

impl SimpleRequest {
    pub fn with_encoding(encoding: u32) -> Self {
        Self {
            encoding,
            response: 0
        }
    }

    pub fn get_response(&self) -> u32 {
        self.response
    }
}

impl MailboxInstruction for SimpleRequest {
    fn get_encoding(&self) -> u32 {
        self.encoding
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
        self.response = buffer.read(offset as isize);
    }
}

pub struct GetFirmwareRevision {
    pub revision: u32
}


impl GetFirmwareRevision {
    pub fn new() -> Self {
        Self {
            revision: 0
        }
    }

    pub fn get_response(&self) -> u32 {
        self.revision
    }
}

impl MailboxInstruction for GetFirmwareRevision {
    fn get_encoding(&self) -> u32 {
        0x1
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

pub struct GetBoardModel {
    pub model: u32
}


impl GetBoardModel {
    pub fn new() -> Self {
        Self {
            model: 0
        }
    }

    pub fn get_response(&self) -> u32 {
        self.model
    }
}

impl MailboxInstruction for GetBoardModel {
    fn get_encoding(&self) -> u32 {
        0x10001
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
        self.model = buffer.read(offset as isize);
    }
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

pub struct GetBoardSerial {
    pub serial: u64
}


impl GetBoardSerial {
    pub fn new() -> Self {
        Self {
            serial: 0
        }
    }

    pub fn get_response(&self) -> u64 {
        self.serial
    }
}

impl MailboxInstruction for GetBoardSerial {
    fn get_encoding(&self) -> u32 {
        0x10004
    }

    fn get_buffer_bytes(&self) -> u32 {
        8
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
        buffer.write(offset as isize, 0);
        buffer.write((offset + 1) as isize, 0);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        // TODO: check endianness
        let first_half = buffer.read(offset as isize) as u64;
        let second_half = buffer.read((offset + 1) as isize) as u64;
        self.serial = (first_half << 32) | second_half;
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

    fn get_buffer_bytes(&self) -> u32 {
        8
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
        buffer.write(offset as isize, 0);
        buffer.write((offset + 1) as isize, 0);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        self.base = buffer.read(offset as isize);
        self.size = buffer.read((offset + 1) as isize);
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

    fn get_buffer_bytes(&self) -> u32 {
        8
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
        buffer.write(offset as isize, 0);
        buffer.write((offset + 1) as isize, 0);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        self.base = buffer.read(offset as isize);
        self.size = buffer.read((offset + 1) as isize);
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

    fn get_buffer_bytes(&self) -> u32 {
        8
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
        buffer.write(offset as isize, self.alignment);
        buffer.write((offset + 1) as isize, 0);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        self.start = buffer.read(offset as isize);
        self.size = buffer.read((offset + 1) as isize);
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

    fn get_buffer_bytes(&self) -> u32 {
        0
    }

    fn get_buffer_words(&self) -> u32 {
        0
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
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

    fn get_buffer_bytes(&self) -> u32 {
        4
    }

    fn get_buffer_words(&self) -> u32 {
        1 
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
        let word = if self.state {0x1} else {0x0};
        buffer.write(offset as isize, word);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        let word = buffer.read(offset as isize);
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
}

impl MailboxInstruction for GetPhysicalDimensions {
    fn get_encoding(&self) -> u32 {
        0x40003
    }

    fn get_buffer_bytes(&self) -> u32 {
        8
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
        buffer.write(offset as isize, 0);
        buffer.write((offset + 1) as isize, 0);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        self.width= buffer.read(offset as isize);
        self.height = buffer.read((offset + 1) as isize);
    }
}

pub struct SetPhysicalDimensions {
    pub width: u32,
    pub height: u32
}

impl SetPhysicalDimensions {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }
}

impl MailboxInstruction for SetPhysicalDimensions {
    fn get_encoding(&self) -> u32 {
        0x48003
    }

    fn get_buffer_bytes(&self) -> u32 {
        8
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
        buffer.write(offset as isize, self.width);
        buffer.write((offset + 1) as isize, self.height);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        self.width= buffer.read(offset as isize);
        self.height = buffer.read((offset + 1) as isize);
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
}

impl MailboxInstruction for GetVirtualDimensions {
    fn get_encoding(&self) -> u32 {
        0x40004
    }

    fn get_buffer_bytes(&self) -> u32 {
        8
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
        buffer.write(offset as isize, 0);
        buffer.write((offset + 1) as isize, 0);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        self.width= buffer.read(offset as isize);
        self.height = buffer.read((offset + 1) as isize);
    }
}

pub struct SetVirtualDimensions {
    pub width: u32,
    pub height: u32
}

impl SetVirtualDimensions {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }
}

impl MailboxInstruction for SetVirtualDimensions {
    fn get_encoding(&self) -> u32 {
        0x48004
    }

    fn get_buffer_bytes(&self) -> u32 {
        8
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
        buffer.write(offset as isize, self.width);
        buffer.write((offset + 1) as isize, self.height);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        self.width= buffer.read(offset as isize);
        self.height = buffer.read((offset + 1) as isize);
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
        self.depth = buffer.read(offset as isize)
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
}

impl MailboxInstruction for SetDepth {
    fn get_encoding(&self) -> u32 {
        0x48005
    }

    fn get_buffer_bytes(&self) -> u32 {
        4
    }

    fn get_buffer_words(&self) -> u32 {
        1
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
        buffer.write(offset as isize, self.depth);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        self.depth = buffer.read(offset as isize)
    }
}

#[derive(Copy, Clone)]
pub enum PixelOrder {
    BGR,
    RGB
}

impl PixelOrder {
    pub fn to_u32(self) -> u32 {
        match self {
            PixelOrder::BGR => 0x0,
            PixelOrder::RGB => 0x1
        }
    }

    pub fn from_u32(int: u32) -> Self {
        match int {
            0 => PixelOrder::BGR,
            1 => PixelOrder::RGB,
            _ => panic!("Unknown pixel order") // Better error handling
        }
    }
}

pub struct GetPixelOrder {
    pub order: PixelOrder
}

impl GetPixelOrder {
    pub fn new() -> Self {
        Self {
            order: PixelOrder::RGB
        }
    }

    pub fn get_order(&self) -> PixelOrder {
        self.order
    }
}

impl MailboxInstruction for GetPixelOrder {
    fn get_encoding(&self) -> u32 {
        0x40006
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
        self.order = PixelOrder::from_u32(buffer.read(offset as isize));
    }
}

pub struct SetPixelOrder {
    pub order: PixelOrder
}

impl SetPixelOrder {
    pub fn new(order: PixelOrder) -> Self {
        Self {
            order
        }
    }

    pub fn get_order(&self) -> PixelOrder {
        self.order
    }
}

impl MailboxInstruction for SetPixelOrder {
    fn get_encoding(&self) -> u32 {
        0x48006
    }

    fn get_buffer_bytes(&self) -> u32 {
        4
    }

    fn get_buffer_words(&self) -> u32 {
        1
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
        buffer.write(offset as isize, self.order.to_u32());
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        self.order = PixelOrder::from_u32(buffer.read(offset as isize));
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
}

impl MailboxInstruction for GetPitch {
    fn get_encoding(&self) -> u32 {
        0x40008
    }

    fn get_buffer_words(&self) -> u32 {
        1
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
        buffer.write(offset as isize, 0);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        self.pitch = buffer.read(offset as isize)
    }
}

pub struct GetVirtualOffset {
    pub x: u32,
    pub y: u32
}

impl GetVirtualOffset {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0
        }
    }

    pub fn get_x(&self) -> u32 { self.x }
    pub fn get_y(&self) -> u32 { self.y }
}

impl MailboxInstruction for GetVirtualOffset {
    fn get_encoding(&self) -> u32 { 0x40009 }
    fn get_buffer_words(&self) -> u32 { 2 }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        self.x = buffer.read(offset as isize);
        self.y = buffer.read((offset + 1) as isize);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Overscan {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32
}

impl Overscan {
    pub fn none() -> Self {
        Self {
            top: 0,
            bottom: 0,
            left: 0,
            right: 0
        }
    }
    
    pub fn new(top: u32, bottom: u32, left: u32, right: u32) -> Self {
        Self { top, bottom, left, right }
    }
}

pub struct GetOverscan {
    pub overscan: Overscan
}

impl GetOverscan {
    pub fn new() -> Self {
        Self { overscan: Overscan::none() }
    }

    pub fn get_overscan(&self) -> Overscan {
        self.overscan
    }
}

impl MailboxInstruction for GetOverscan {
    fn get_encoding(&self) -> u32 { 0x4000a }
    fn get_buffer_words(&self) -> u32 { 4 }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        self.overscan.top = buffer.read(offset as isize);
        self.overscan.bottom = buffer.read((offset + 1) as isize);
        self.overscan.left = buffer.read((offset + 2) as isize);
        self.overscan.right = buffer.read((offset + 3) as isize);
    }
}

pub struct SetOverscan {
    pub overscan: Overscan
}

impl SetOverscan {
    pub fn new(overscan: Overscan) -> Self {
        Self { overscan }
    }

    pub fn get_overscan(&self) -> Overscan {
        self.overscan
    }
}

impl MailboxInstruction for SetOverscan {
    fn get_encoding(&self) -> u32 { 0x4800a }
    fn get_buffer_words(&self) -> u32 { 4 }

    fn write_data_at_offset(&self, buffer: &mut MailboxBuffer, offset: u32) {
        buffer.write(offset as isize, self.overscan.top);
        buffer.write((offset + 1) as isize, self.overscan.bottom);
        buffer.write((offset + 2) as isize, self.overscan.left);
        buffer.write((offset + 3) as isize, self.overscan.right);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBuffer, offset: u32) {
        self.overscan.top = buffer.read(offset as isize);
        self.overscan.bottom = buffer.read((offset + 1) as isize);
        self.overscan.left = buffer.read((offset + 2) as isize);
        self.overscan.right = buffer.read((offset + 3) as isize);
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
