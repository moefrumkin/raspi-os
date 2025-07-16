use crate::volatile::Volatile;
use crate::bitfield;
use super::timer::Timer;
use super::mmio::MMIOController;

enum CommandFlag {
    NeedApp = 0x8000_0000,
    Response48 = 0x0002_0000,
    ErrorsMask = 0xfff9_c004,
    RcaMask = 0xffff_0000
}

enum Command {
    GoIdle = 0x0,
    AllSendCid = 0x0201_0000,
    SendRelAddr = 0x0302_0000,
    CardSelect = 0x0703_0000,
    SendIfCond = 0x0802_0000,
    StopTrans = 0x0C02_0000,
    ReadSingle = 0x1122_0010,
    ReadMulti = 0x1222_0032,
    SetBlockCount = 0x1702_0000,
    AppCommand = 0x3700_0000,
    SetBusWidth = (0x0602_0000 | 0x8000_0000),
    SendOpCommand = (0x2902_0000 | 0x8000_0000),
    SendScr = (0x3322_0010 | 0x8000_0000)
}

enum StatusSetting {
    ReadAvailable = 0x0000_0800,
    DataInhibit = 0x0000_0002,
    CommandInhibit = 0x0000_0001,
    AppCommand = 0x0000_0020
}

static mut sd_scr: [u64; 2] = [0, 0];
static mut sd_ocr: u64 = 0;
static mut sd_rca: u64 = 0;
static mut sd_err: u64 = 0;
static mut sd_hv: u64 = 0;

#[repr(C)]
pub struct EMMCRegisters {
    pub arg2: Volatile<u32>,
    pub blockSizeAndCount: Volatile<BlockSizeAndCount>,
    pub arg1: Volatile<u32>,
    pub cmdtm: Volatile<CMDTM>,
    pub resp0: Volatile<u32>,
    pub resp1: Volatile<u32>,
    pub resp2: Volatile<u32>,
    pub resp3: Volatile<u32>,
    pub data: Volatile<u32>,
    pub status: Volatile<Status>,
    pub control0: Volatile<Control0>,
    pub control1: Volatile<Control1>,
    pub interrupt: Volatile<Interrupt>,
    pub irpt_mask: Volatile<InterruptMask>,
    pub irpt_en: Volatile<InterruptEnable>,
    pub control2: Volatile<u32>,
    pub force_irpt: Volatile<ForceInterrupt>,
    pub boot_timeout: Volatile<u32>,
    pub dbg_sgl: Volatile<DBG_SEL>,
    pub exrdfifo_cfg: Volatile<EXRDFIFO_CFG>,
    pub exrdfifo_en: Volatile<EXRDFIFO_EN>,
    pub tune_step: Volatile<TUNE_STEP>,
    pub tune_steps_std: Volatile<TUNE_STEPS_STD>,
    pub tune_steps_ddr: Volatile<TUNE_STEPS_DDR>,
    pub spi_int_spt: Volatile<SPI_INT_SPT>,
    pub slotisr_ver: Volatile<SLOTISR_VER>
}

impl EMMCRegisters {
    const EMMC_CONTROLLER_BASE: usize = 0x7E30_0000;

    const InterruptErrorMask: u32 = 0x017E_8000;

        // INTERRUPT register settings
    const InterruptDataTimeout: u32 = 0x00100000;
    const InterruptCommandTimeout: u32 = 0x00010000;
    const InterruptReadReady: u32 = 0x00000020;
    const InterruptCommandDone: u32 = 0x00000001;

    const C1_CLK_EN: u32 = 0x4;
    const C1_CLK_STABLE: u32 = 0x2;

    const HOST_SPEC_V2: u64 = 1;

    pub fn get() -> &'static mut Self {
        unsafe {
            &mut *(Self::EMMC_CONTROLLER_BASE as *mut Self)
        } 
    }

    pub fn sd_status(&mut self, mask: u32, timer: &Timer) -> bool {
        let mut count: u32 = 500_000;

        while (self.status.get().as_u32() & mask) != 0
            && (self.interrupt.get().as_u32() & Self::InterruptErrorMask) == 0
            && count != 0
        {
            count -= 1;
            timer.delay(1);
        }

        if count == 0 {
            return false;
        } else {
            let intr_val = self.interrupt.get().as_u32() & Self::InterruptErrorMask;
            return intr_val == 0
        }
    }

    pub fn sd_int(&mut self, mask: u32, timer: &Timer) -> bool {
        let next_mask = mask;
        let mask = mask | Self::InterruptErrorMask;
        let mut count = 1_000_000;

        while (self.interrupt.get().as_u32() & mask) == 0 && count > 0 {
            count -= 1;
            timer.delay(1);
        }

        let interrupt = self.interrupt.get().as_u32();

        // TODO these should just be bits on the bitfield
        if count <= 0
            || interrupt & Self::InterruptCommandTimeout != 0
            || interrupt & Self::InterruptDataTimeout != 0
            || interrupt & Self::InterruptErrorMask != 0
        {
            // Is this necessary?
            self.interrupt.set(Interrupt{value:  interrupt });
            return false;
        } else {
            self.interrupt.set(Interrupt{value: next_mask});
            return true;
        }
    }

    pub fn sd_command(&mut self, mut command: u32, arg: u32, timer: &Timer) -> u32 {
        let mut r = 0;

        if (command & CommandFlag::NeedApp as u32) != 0 {
            let new_command = Command::AppCommand  as u32 | (if unsafe {sd_rca } != 0 {CommandFlag::Response48 as u32} else {0});
            r = self.sd_command(new_command, unsafe {sd_rca as u32}, timer);

            if(unsafe { sd_rca != 0} && r == 0) {
                panic!("ERROR: failed to send SD APP command");
            }

            command &= !(CommandFlag::NeedApp as u32);
        }

        if(self.sd_status(StatusSetting::CommandInhibit as u32, timer)) {
            panic!("ERROR: EMMC busy");
        }
        
        // Do we really need to do this?--It seems redundant
        self.interrupt.set(self.interrupt.get());
        self.arg1.set(arg);
        self.arg2.set(command);

        if(command == Command::SendOpCommand as u32) {
            timer.delay(1000);
        } else if(command == Command::SendIfCond as u32 || command == Command::AppCommand as u32) {
            timer.delay(100);
        }

        if(!self.sd_int(Self::InterruptCommandDone, timer)) {
            panic!("ERROR: failed to send EMMC command");
        }

        r = self.resp0.get();

        if(command == Command::GoIdle as u32 || command == Command::AppCommand as u32) {
            return 0
        } else if(command == Command::AppCommand as u32 | CommandFlag::Response48 as u32) {
            return r & StatusSetting::AppCommand as u32;
        } else if(command == Command::SendOpCommand as u32) {
            return r;
        } else if(command == Command::SendIfCond as u32) {
            if r == arg {
                return 0;
            } else {
                return 1;
            }
        } else if (command == Command::AllSendCid as u32) {
            r |= self.resp3.get();
            r |= self.resp2.get();
            r |= self.resp1.get();
            return r;
        } else if (command == Command::SendRelAddr as u32) {
            unsafe {
                sd_err = ((((r&0x1fff))|((r&0x2000)<<6)|((r&0x4000)<<8)|((r&0x8000)<<8))& CommandFlag::ErrorsMask as u32) as u64;
                return r & CommandFlag::RcaMask as u32;
            }
        }

        return r & CommandFlag::ErrorsMask as u32;
    }

    pub fn sd_clk(&mut self, f: u32, timer: &Timer) {
        let mut d: u32;
        let c = 41666666/f;
        let mut x: u32;
        let mut s = 32;
        let mut h = 0;

        let mut count = 100_000;

        while (self.status.get().as_u32() & (StatusSetting::CommandInhibit as u32 | StatusSetting::DataInhibit as u32) != 0)
            && count > 0 {
                count -= 1;
                timer.delay(1);
        }

        if count <= 0 {
            panic!("ERROR: timeout waiting for inhibit flag");
        }

        let control1_value = self.control1.get().as_u32();
        self.control1.set(Control1{ value: control1_value & !Self::C1_CLK_EN});
        timer.delay(10);

        x = c - 1;
        if(x == 0) {
            s = 0;
        } else {
            if((x & 0xffff0000) == 0) { x <<= 16; s -= 16; }
            if((x & 0xff000000) == 0) { x <<= 8;  s -= 8; }
            if((x & 0xf0000000) == 0) { x <<= 4;  s -= 4; }
            if((x & 0xc0000000) == 0) { x <<= 2;  s -= 2; }
            if((x & 0x80000000) == 0) { x <<= 1;  s -= 1; }
            if(s>0) {
                s -= 1;
            }
            if(s>7) {
                s=7;
            }
        }

        if(unsafe {sd_hv} > Self::HOST_SPEC_V2) {
            d = c;
        } else {
            d = (1 << s);
        }

        if(d <= 2) {
            d = 2;
            s = 0;
        }
        
        if(unsafe {sd_hv > Self::HOST_SPEC_V2}) {
            h = (d&0x300) >> 2;
        }

        d = (((d & 0x0ff) << 8) | h);

        self.control1.set(
            Control1 {
                value: (self.control1.get().as_u32() & 0xffff_003f) | d,
            }
        );

        timer.delay(10);

        self.control1.set(
            Control1 {
                value: self.control1.get().as_u32() | Self::C1_CLK_EN
            }
        );

        timer.delay(10);

        count = 10_000;

        while(self.control1.get().as_u32() & Self::C1_CLK_STABLE == 0) && count > 0 {
            count -= 1;
            timer.delay(10);
        }

        if(count <= 0) {
            panic!("ERROR: failed to get stable clock");
        }
    }
}

bitfield! {
    BlockSizeAndCount(u32) {
        blockSize: 0-9,
        numberOfBlocks: 16-31
    }
}

bitfield! {
    CMDTM(u32) {
        TM_BLKCNT_EN: 1-1,
        TM_AUTO_CMD_END: 2-3,
        TM_DAT_DIR: 4-4,
        TM_MULTI_BLOCK: 5-5,
        CMD_RSPNS_TYPE: 16-17,
        CMD_CRCCHK_EN: 19-19,
        CMD_IXCHK_EN: 20-20,
        CMD_ISDATA: 21-21,
        CMD_TYPE: 22-23,
        CMD_INDEX: 24-29
    }
}

bitfield! {
    Status(u32) {
        CMD_INHIBIT: 0-0,
        DAT_INHIBIT: 1-1,
        DAT_ACTIVE: 2-2,
        WRITE_TRANSFER: 8-8,
        READ_TRANSFER: 9-9,
        DAT_LEVEL0: 20-23,
        CMD_LEVEL: 24-24,
        DAT_LEVEL1: 25-28
    } with {
        pub fn as_u32(&self) -> u32 {
            self.value
        }
    }
}

bitfield! {
    Control0(u32) {
        HCTL_DWIDTH: 1-1,
        HCTL_HS_EN: 2-2,
        HCTL_8BIT: 5-5,
        GAP_STOP: 16-16,
        GAP_RESTART: 17-17,
        READWAIT_EN: 18-18,
        GAP_IEN: 19-19,
        SPI_MODE: 20-20,
        BOOT_EN: 21-21,
        ALT_BOOT_EN: 22-22
    }
}

bitfield! {
    Control1(u32) {
        CLK_INTLEN: 0-0,
        CLK_STABLE: 1-1,
        CLK_EN: 2-2,
        CLK_GENSEL: 5-5,
        CLK_FREQ_MS2: 6-7,
        CLK_FREQ8: 8-15,
        DATA_TOUNIT: 16-19,
        SRST_HC: 24-24,
        SRST_CMD: 25-25,
        SRST_DATA: 26-26
    } with {
        pub fn as_u32(&self) -> u32 {
            self.value
        }
    }
}

bitfield! {
    Interrupt(u32) {
        CMD_DONE: 0-0,
        DATA_DONE: 1-1,
        BLOCK_GAP: 2-2,
        WRITE_RDY: 4-4,
        READ_RDY: 5-5,
        CARD: 8-8,
        RETUNE: 12-12,
        BOOTACK: 13-13,
        ENDBOOT: 14-14,
        ERR: 15-15,
        CTO_ERR: 16-16,
        CCRC_ERR: 17-17,
        CEND_ERR: 18-18,
        CBAD_ERR: 19-19,
        DTO_ERR: 20-20,
        DCRC_ERR: 21-21,
        DEND_ERR: 22-22,
        ACMD_ERR: 24-24
    } with {
        pub fn as_u32(&self) -> u32 {
            self.value
        }
    }
}

bitfield! {
    InterruptMask(u32) {
        CMD_DONE: 0-0,
        DATA_DONE: 1-1,
        BLOCK_GAP: 2-2,
        WRITE_RDY: 4-4,
        READ_RDY: 5-5,
        CARD: 8-8,
        RETUNE: 12-12,
        BOOTACK: 13-13,
        ENDBOOT: 14-14,
        CTO_ERR: 16-16,
        CRRC_ERR: 17-17,
        CBAD_ERR: 19-19,
        DTO_ERR: 20-20,
        DCRC_ERR: 21-21,
        DEND_ERR: 22-22,
        ACMD_ERR: 24-24
    }
}

bitfield! {
    InterruptEnable(u32) {
        CMD_DONE: 0-0,
        DATA_DONE: 1-1,
        BLOCK_GAP: 2-2,
        WRITE_RDY: 4-4,
        READ_RDY: 5-5,
        CARD: 8-8,
        RETUNE: 12-12,
        BOOTACK: 13-13,
        ENDBOOT: 14-14,
        CTO_ERR: 16-16,
        CRRC_ERR: 17-17,
        CBAD_ERR: 19-19,
        DTO_ERR: 20-20,
        DCRC_ERR: 21-21,
        DEND_ERR: 22-22,
        ACMD_ERR: 24-24
    }
}

bitfield! {
    Control2(u32) {
        ACNOX_ERR: 0-0,
        ACTO_ERR: 1-1,
        ACCRC_ERR: 2-2,
        ACEND_ERR: 3-3,
        ACBAD_ERR: 4-4,
        NOTC12_ERR: 7-7,
        UHSMODE: 16-18,
        TUNEON: 22-22,
        TUNED: 23-23
    }
}

bitfield! {
    ForceInterrupt(u32) {
        CMD_DONE: 0-0,
        DATA_DONE: 1-1,
        BLOCK_GAP: 2-2,
        WRITE_RDY: 4-4,
        READ_RDY: 5-5,
        CARD: 8-8,
        RETUNE: 12-12,
        BOOTACK: 13-13,
        ENDBOOT: 14-14,
        CTO_ERR: 16-16,
        CRRC_ERR: 17-17,
        CBAD_ERR: 19-19,
        DTO_ERR: 20-20,
        DCRC_ERR: 21-21,
        DEND_ERR: 22-22,
        ACMD_ERR: 24-24
    }
}

bitfield! {
    DBG_SEL(u32) {
        SELECT: 0-0
    }
}

bitfield! {
    EXRDFIFO_CFG(u32) {
        RD_THRSH: 0-2
    }
}

bitfield! {
    EXRDFIFO_EN(u32) {
        ENABLE: 0-0
    }
}

bitfield! {
    TUNE_STEP(u32) {
        DELAY: 0-2
    }
}

bitfield! {
    TUNE_STEPS_STD(u32) {
        STEPS: 0-5
    }
}

bitfield! {
    TUNE_STEPS_DDR(u32) {
        STEPS: 0-5
    }
}

bitfield! {
    SPI_INT_SPT(u32) {
        SELECT: 0-7
    }
}

bitfield! {
    SLOTISR_VER(u32) {
        VENDOR: 24-31,
        SDVERSION: 16-23,
        SLOT_STATUS: 0-7
    }
}