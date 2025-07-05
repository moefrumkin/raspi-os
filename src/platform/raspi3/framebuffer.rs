
use core::fmt::{Display, Formatter};
use core::fmt;
use crate::platform::raspi3::mailbox::{MailboxController};
use crate::platform::raspi3::mailbox_property::{
    MessageBuilder,
    GetFrameBuffer,
    SetDepth,
    SetPhysicalDimensions,
    SetVirtualDimensions,
    GetPitch,
    SetPixelOrder,
    GetVirtualOffset,
    SetOverscan
};
use crate::volatile::Volatile;

pub struct FrameBuffer<'a> {
    config: FrameBufferConfig,
    buffer: &'a mut [Volatile<u32>]
}

impl<'a> FrameBuffer<'a> {
    pub fn from_config(config: FrameBufferConfig, mailbox: &mut MailboxController) -> Self {
        // TODO: make sure all are setters
        let mut frame_buffer_request = GetFrameBuffer::with_aligment(32); 
        let mut depth = SetDepth::new(config.depth);
        let mut physical_dimensions = SetPhysicalDimensions::new(config.physical_dimensions);
        let mut virtual_dimensions = SetVirtualDimensions::new(config.virtual_dimensions);
        let mut pitch = GetPitch::new();
        let mut pixel_order = SetPixelOrder::new(config.pixel_order);
        let mut virtual_offset = GetVirtualOffset::new();
        let mut overscan = SetOverscan::new(Overscan::none());

        let mut frame_buffer_message = MessageBuilder::new()
            .request(&mut frame_buffer_request)
            .request(&mut depth)
            .request(&mut physical_dimensions)
            .request(&mut virtual_dimensions)
            .request(&mut pitch)
            .request(&mut pixel_order)
            .request(&mut virtual_offset)
            .request(&mut overscan);

        frame_buffer_message.send(mailbox);

        let buffer_size = frame_buffer_request.get_size();

        let expected_buffer_size = config.virtual_dimensions.get_width() * config.virtual_dimensions.get_height() * (config.depth / 8);

        if(buffer_size != expected_buffer_size) {
            //panic!("Requested a buffer with size {}, got size {}", expected_buffer_size, buffer_size);
        }

        let start_addr = (frame_buffer_request.get_start() &0x3fffffff) as u64;

        let buffer = unsafe {
            core::slice::from_raw_parts_mut(start_addr as *mut Volatile<u32>, (config.virtual_dimensions.get_width() * config.virtual_dimensions.get_height()) as usize)
        };

        let actual_config = FrameBufferConfig {
            depth: depth.get(),
            physical_dimensions: physical_dimensions.get(),
            virtual_dimensions: virtual_dimensions.get(),
            pitch: pitch.get(),
            pixel_order: pixel_order.get_order(),
            virtual_offset: Offset::none(),
            overscan: overscan.get_overscan()
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

pub struct FrameBufferController {

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

pub type Pitch = u32;

#[derive(Copy, Clone, Debug)]
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


#[derive(Copy, Clone, Debug)]
pub struct FrameBufferConfig {
    pub depth: Depth,
    pub physical_dimensions: Dimensions,
    pub virtual_dimensions: Dimensions,
    pub pitch: Pitch,
    pub pixel_order: PixelOrder,
    pub virtual_offset: Offset,
    pub overscan: Overscan
}
