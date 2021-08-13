use super::mailbox::{Channel, MailboxController};
use super::mmio::MMIOController;

static mut MESSAGE_BUFFER: GPUMessageBuffer =  GPUMessageBuffer::new();

#[allow(dead_code)]
pub struct GPUController<'a> {
    mmio: &'a MMIOController,
    mailbox: &'a MailboxController<'a>,
    fb: &'a mut [u32],
    fb_config: FBConfig,
}

pub const MBOX_REQUEST: u32 = 0;

impl<'a> GPUController<'a> {
    pub fn init(mmio: &'a MMIOController, mailbox: &'a MailboxController, _: FBConfig) -> Self {
        
        unsafe {
            mailbox.call(MESSAGE_BUFFER.start() as u32, Channel::Prop);
        }
        
        let fb_config = unsafe {
            FBConfig {
                phy_width: MESSAGE_BUFFER.get(5),
                phy_height: MESSAGE_BUFFER.get(6),

                vir_width: MESSAGE_BUFFER.get(10),
                vir_height: MESSAGE_BUFFER.get(11),

                width_off: MESSAGE_BUFFER.get(15),
                height_off: MESSAGE_BUFFER.get(16),

                depth: MESSAGE_BUFFER.get(20),

                pxl_order: MESSAGE_BUFFER.get(24),

                pitch: MESSAGE_BUFFER.get(33),
            }
        };

        let fb = unsafe {
            core::slice::from_raw_parts_mut(
                (MESSAGE_BUFFER.get(28) & 0x3FFFFFFF) as *mut u32,
                (1920 * 1080) as usize,
            )
        };
        
        return GPUController {
            mmio,
            mailbox,
            fb,
            fb_config,
        };
    }

    pub fn set_pxl(&mut self, x: u32, y: u32, color: u32) {
        self.fb[self.address(x, y)] = color;
    }

    fn address(&self, x: u32, y: u32) -> usize {
        (self.fb_config.vir_width * y + x) as usize
    }
}

#[allow(dead_code)]
pub struct FBConfig {
    phy_width: u32,
    phy_height: u32,

    vir_width: u32,
    vir_height: u32,

    width_off: u32,
    height_off: u32,

    //Bits per pixel
    depth: u32,

    //ARGB or BGRA?
    pxl_order: u32,

    //Bytes per line, provided by GPU
    pitch: u32,
}

impl Default for FBConfig {
    fn default() -> Self {
        FBConfig {
            phy_width: 1920,
            phy_height: 1080,

            vir_width: 1920,
            vir_height: 1080,

            width_off: 0,
            height_off: 0,

            depth: 32,

            pxl_order: 1,

            pitch: 0,
        }
    }
}

#[repr(C)]
#[repr(align(16))]
#[allow(dead_code)]
pub struct GPUMessageBuffer {
    data: [u32; GPUMessageBuffer::BUFFER_LENGTH]
}

impl GPUMessageBuffer {
    const BUFFER_LENGTH: usize = 36;

    pub const fn new() -> Self {
        GPUMessageBuffer {
            data: [
                //Headers: message size and type
                35 * 4,
                MBOX_REQUEST,
                Tag::SetPhyDim as u32,
                8,
                8,
                1920,
                1080,
                Tag::SetVirDim as u32,
                8,
                8,
                1920,
                1080,
                Tag::SetVirOff as u32,
                8,
                8,
                0,
                0,
                Tag::SetDepth as u32,
                4,
                4,
                32,
                Tag::SetPxlOrdr as u32,
                4,
                4,
                1,
                Tag::GetFB as u32,
                8,
                8,
                4096,
                0,
                Tag::GetPitch as u32,
                4,
                4,
                0,
                Tag::EndOfMessage as u32,
                0,
            ]
        }
    }

    pub fn get(&self, i: usize) -> u32 {
        self.data[i]
    }

    pub fn set(&mut self, i: usize, n: u32) {
        self.data[i] = n;
    }

    pub fn start(&self) -> usize {
        self as *const Self as usize
    }
}

pub struct GPUMessageBuilder<'a> {
    buffer: &'a mut GPUMessageBuffer,
    add_index: usize
}

impl<'a> GPUMessageBuilder<'a> {
    #[allow(dead_code)]
    pub fn new(buffer: &'a mut GPUMessageBuffer) -> Self {
        Self {
            buffer,
            add_index: 0
        }
    }

    #[allow(dead_code)]
    pub fn add(&mut self, n: u32) {
        self.buffer.set(self.add_index, n);
        self.add_index += 1;
    }

    #[allow(dead_code)]
    pub fn add_tag(&mut self, tag: Tag) {
        self.add(tag as u32);
    }

    #[allow(dead_code)]
    pub fn add_size(&mut self, size: u32) {
        self.add(size);
        self.add(size);
    }
}

#[allow(dead_code)]
pub enum Tag {
    SetPower = 0x28001,
    SetClkRate = 0x38002,

    SetPhyDim = 0x48003,
    SetVirDim = 0x48004,
    SetVirOff = 0x48009,
    SetDepth = 0x48005,
    SetPxlOrdr = 0x48006,
    GetFB = 0x40001,
    GetPitch = 0x40008,

    EndOfMessage = 0,
}
