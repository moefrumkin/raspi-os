use super::mailbox::{Channel, MailboxController, MBOX};
use super::mmio::MMIOController;

pub const MBOX_REQUEST: u32 = 0;

#[allow(dead_code)]
pub struct GPUController<'a> {
    mmio: &'a MMIOController,
    mailbox: &'a MailboxController<'a>,
    fb: &'a mut [u32],
    fb_config: FBConfig,
}

impl<'a> GPUController<'a> {
    pub fn init(mmio: &'a MMIOController, mailbox: &'a MailboxController, _: FBConfig) -> Self {
        unsafe {
            mailbox.call(MBOX.data.as_ptr() as u32, Channel::Prop);
        }

        let fb_config = unsafe {
            FBConfig {
                phy_width: MBOX.data[5],
                phy_height: MBOX.data[6],

                vir_width: MBOX.data[10],
                vir_height: MBOX.data[11],

                width_off: MBOX.data[15],
                height_off: MBOX.data[16],

                depth: MBOX.data[20],

                pxl_order: MBOX.data[24],

                pitch: MBOX.data[33],
            }
        };

        let fb = unsafe {
            core::slice::from_raw_parts_mut(
                (MBOX.data[28] & 0x3FFFFFFF) as *mut u32,
                (1920 * 1080) as usize,
            )
        };

        GPUController {
            mmio,
            mailbox,
            fb,
            fb_config,
        }
    }
    pub fn set(&mut self, x: u32, y: u32, color: u32) {
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
