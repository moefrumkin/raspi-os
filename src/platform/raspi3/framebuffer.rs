
use core::cell::RefCell;
use core::fmt::{Display, Formatter};
use core::fmt;
use alloc::rc::Rc;
use crate::platform::raspi3::mailbox::{MailboxController};
use crate::platform::raspi3::mailbox_property::{
    MessageBuilder,
    GetFrameBuffer,
    SetDepth,
    SetPhysicalDimensions,
    SetVirtualDimensions,
    GetPitch,
    MailboxInstruction,
    MailboxBufferSlice
};
use crate::volatile::Volatile;

pub struct FrameBuffer<'a> {
    config: FrameBufferConfig,
    buffer: &'a mut [Volatile<u32>]
}

impl<'a> FrameBuffer<'a> {
    pub fn from_config(config: FrameBufferConfig, mailbox: &dyn MailboxController) -> Self {
        // TODO: make sure all are setters
        let mut depth = SetDepth::new(config.depth);
        let mut overscan = FramebufferPropertyRequest::<Overscan>::set(config.overscan);
        let mut physical_dimensions = SetPhysicalDimensions::new(config.physical_dimensions);
        let mut pitch = GetPitch::new();
        let mut pixel_order = FramebufferPropertyRequest::<PixelOrder>::set(config.pixel_order);
        let mut virtual_dimensions = SetVirtualDimensions::new(config.virtual_dimensions);
        let mut virtual_offset = FramebufferPropertyRequest::<Offset>::get();
        let mut frame_buffer_request = GetFrameBuffer::with_aligment(32); 

        let mut frame_buffer_message = MessageBuilder::new()
            .request(&mut depth)
            .request(&mut overscan)
            .request(&mut physical_dimensions)
            .request(&mut pitch)
            .request(&mut pixel_order)
            .request(&mut virtual_dimensions)
            .request(&mut virtual_offset)
            .request(&mut frame_buffer_request);

        frame_buffer_message.send(mailbox);

        // TODO remove magic number
        let start_addr = (frame_buffer_request.get_start() &0x3fffffff) as u64;

        let buffer = unsafe {
            core::slice::from_raw_parts_mut(start_addr as *mut Volatile<u32>, (config.virtual_dimensions.get_width() * config.virtual_dimensions.get_height()) as usize)
        };

        let actual_config = FrameBufferConfig {
            depth: depth.get(),
            overscan: overscan.get_response(),
            physical_dimensions: physical_dimensions.get(),
            pitch: pitch.get(),
            pixel_order: pixel_order.get_response(),
            virtual_dimensions: virtual_dimensions.get(),
            virtual_offset: Offset::none()
        };



        Self {
            config: actual_config,
            buffer
        }
    }

    pub fn write_idx(&mut self, idx: u32, color: u32) {
        self.buffer[idx as usize].set(color);
    }

    pub fn write_pixel(&mut self, x: u32, y: u32, color: u32) {
        self.buffer[(y * self.config.physical_dimensions.get_width() + x) as usize].set(color);
    }

    pub fn get_config(&self) -> FrameBufferConfig {
        self.config
    }
}

pub trait FramebufferProperty {
    const SIZE: u32;
    const BASE_ENCODING: u32;

    fn write_to_buffer(&self, buffer: &mut MailboxBufferSlice);

    fn read_from_buffer(buffer: &MailboxBufferSlice) -> Self;
}

enum FramebufferPropertyRequestType<T: FramebufferProperty> {
    Get,
    Test(T),
    Set(T)
}

struct FramebufferPropertyRequest<T: FramebufferProperty> {
    request: FramebufferPropertyRequestType<T>,
    response: Option<T>
}

impl<T: FramebufferProperty> FramebufferPropertyRequest<T> {
    fn with_request(request: FramebufferPropertyRequestType<T>) -> Self {
        Self {
            request,
            response: Option::None
        }
    }

    pub fn get() -> Self {
        Self::with_request(FramebufferPropertyRequestType::Get)
    }

    pub fn test(value: T) -> Self {
        Self::with_request(FramebufferPropertyRequestType::Test(value))
    }

    pub fn set(value: T) -> Self {
        Self::with_request(FramebufferPropertyRequestType::Set(value))
    }

}

impl<T: FramebufferProperty + Copy> FramebufferPropertyRequest<T> {
    pub fn get_response(&self) -> T {
        self.response.expect("Attempted to get response for an unsent message")
    }

}

const GET_ENCODING: u32 = 0x0;
const TEST_ENCODING: u32 = 0x4000;
const SET_ENCODING: u32 = 0x8000;

impl<T: FramebufferProperty> MailboxInstruction for FramebufferPropertyRequest<T> {
    fn get_encoding(&self) -> u32 {
        let modifier = match self.request {
            FramebufferPropertyRequestType::Get => GET_ENCODING,
            FramebufferPropertyRequestType::Test(_) => TEST_ENCODING,
            FramebufferPropertyRequestType::Set(_) => SET_ENCODING
        };

        T::BASE_ENCODING | modifier
    }

    fn get_buffer_words(&self) -> u32 {
        T::SIZE
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBufferSlice) {
        match &self.request {
            FramebufferPropertyRequestType::Get => {}
            FramebufferPropertyRequestType::Test(value) | FramebufferPropertyRequestType::Set(value) =>  {
                value.write_to_buffer(buffer);
            }
        };
    } 

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.response = Some(T::read_from_buffer(buffer));
    }
}

pub type Depth = u32;

#[derive(Copy, Clone, Debug)]
pub struct Dimensions {
    width: u32,
    height: u32
}

impl Dimensions {
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

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
    }
}

impl Display for Dimensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} x {}", self.width, self.height) 
    }
}

pub type Pitch = u32;

#[derive(Copy, Clone, Debug)]
pub enum PixelOrder {
    BGR,
    RGB
}

impl PixelOrder {
    // TODO: should these functions use conversion traits?
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

impl FramebufferProperty for PixelOrder {
    const SIZE: u32 = 1;
    const BASE_ENCODING: u32 = 0x40006;

    fn write_to_buffer(&self, buffer: &mut MailboxBufferSlice) {
        buffer[0].set(self.to_u32());
    }

    fn read_from_buffer(buffer: &MailboxBufferSlice) -> Self {
        Self::from_u32(buffer[0].get())
    }
}

impl Display for PixelOrder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            PixelOrder::BGR => "BGR",
            PixelOrder::RGB => "RGB"
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Offset {
    x: u32,
    y: u32
}

impl Offset {
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            x, 
            y
        }
    }

    pub fn none() -> Self {
        Self::new(0, 0)
    }
}

impl FramebufferProperty for Offset {
    const SIZE: u32 = 2;
    const BASE_ENCODING: u32 = 0x40009;

    fn write_to_buffer(&self, buffer: &mut MailboxBufferSlice) {
        buffer[0].set(self.x);
        buffer[1].set(self.y);
    }

    fn read_from_buffer(buffer: &MailboxBufferSlice) -> Self {
        Self::new(buffer[0].get(), buffer[1].get())
    }
}

impl Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(x: {}, y: {})", self.x, self.y)
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
    pub fn new(top: u32, bottom: u32, left: u32, right: u32) -> Self {
        Self { top, bottom, left, right }
    }

    pub fn none() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

impl FramebufferProperty for Overscan {
    const SIZE: u32 = 4;
    const BASE_ENCODING: u32 = 0x4000a;

    fn write_to_buffer(&self, buffer: &mut MailboxBufferSlice) {
        buffer[0].set(self.top);
        buffer[1].set(self.bottom);
        buffer[2].set(self.left);
        buffer[3].set(self.right);
    }

    fn read_from_buffer(buffer: &MailboxBufferSlice) -> Self {
        Self::new(buffer[0].get(), buffer[1].get(), buffer[2].get(), buffer[3].get())
    }
}

impl Display for Overscan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(top: {}, bottom: {}, left: {}, right: {})",
            self.top,
            self.bottom,
            self.left,
            self.right)
    }
}


#[derive(Copy, Clone, Debug)]
pub struct FrameBufferConfig {
    pub depth: Depth,
    pub overscan: Overscan,
    pub pitch: Pitch,
    pub pixel_order: PixelOrder,
    pub physical_dimensions: Dimensions,
    pub virtual_dimensions: Dimensions,
    pub virtual_offset: Offset
}

impl Display for FrameBufferConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n\
        \t -depth: {} \n\
        \t -overscan: {} \n\
        \t -pitch: {} \n\
        \t -pixel order: {} \n\
        \t -physical dimensions: {} \n\
        \t -virtual dimensions: {} \n\
        \t -virtual offset: {} \n",
        self.depth,
        self.overscan,
        self.pitch,
        self.pixel_order,
        self.physical_dimensions,
        self.virtual_dimensions,
        self.virtual_offset)
    }
}

pub struct FrameBufferConfigBuilder {
    depth: Option<Depth>,
    overscan: Option<Overscan>,
    pixel_order: Option<PixelOrder>,
    physical_dimensions: Option<Dimensions>,
    virtual_dimensions: Option<Dimensions>,
    virtual_offset: Option<Offset>
}

impl FrameBufferConfigBuilder {
    pub fn new() -> Self {
        Self {
            depth: Option::None,
            overscan: Option::None,
            pixel_order: Option::None,
            physical_dimensions: Option::None,
            virtual_dimensions: Option::None,
            virtual_offset: Option::None
        }
    }

    pub fn build(&self) -> FrameBufferConfig {
        FrameBufferConfig {
            depth: self.depth.unwrap(),
            overscan: self.overscan.unwrap(),
            pitch: 0, // Pitch can not be set
            pixel_order: self.pixel_order.unwrap(),
            physical_dimensions: self.physical_dimensions.unwrap(),
            virtual_dimensions: self.virtual_dimensions.unwrap(),
            virtual_offset: self.virtual_offset.unwrap()
        }
    }

    pub fn depth(mut self, depth: Depth) -> Self {
        self.depth = Some(depth);
        self
    }

    pub fn overscan(mut self, overscan: Overscan) -> Self {
        self.overscan = Some(overscan);
        self
    }

    pub fn pixel_order(mut self, order: PixelOrder) -> Self {
        self.pixel_order = Some(order);
        self
    }

    pub fn physical_dimensions(mut self, dimensions: Dimensions) -> Self {
        self.physical_dimensions = Some(dimensions);
        self
    }

    pub fn virtual_dimensions(mut self, dimensions: Dimensions) -> Self {
        self.virtual_dimensions = Some(dimensions);
        self
    }

    pub fn virtual_offset(mut self, offset: Offset) -> Self {
        self.virtual_offset = Some(offset);
        self
    }
}
