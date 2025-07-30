use crate::volatile::Volatile;
use crate::bitfield;
use super::timer::Timer;
use super::mmio::MMIOController;
use super::gpio::{GPIOController, Pin, Pull, Mode};
use crate::aarch64::cpu::wait_for_cycles;
use super::uart::CONSOLE;
use crate::println;

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

#[derive(Copy, Clone)]
pub enum StatusSetting {
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
    pub command: Volatile<SDCommand>,
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
    const EMMC_CONTROLLER_BASE: usize = 0x3F30_0000;

    const InterruptErrorMask: u32 = 0x017E_8000;

        // INTERRUPT register settings
    const InterruptDataTimeout: u32 = 0x00100000;
    const InterruptCommandTimeout: u32 = 0x00010000;
    const InterruptReadReady: u32 = 0x00000020;
    const InterruptCommandDone: u32 = 0x00000001;

    const HOST_SPEC_NUM: u32 = 0xff_0000;
    const HOST_SPEC_NUM_SHIFT: u32 = 16;

    const C0_HCTL_DWIDTH: u32 = 0x2;
    
    const C1_SRST_HC: u32 = 0x0100_0000;
    const C1_CLK_INTLEN: u32 = 0x1;
    const C1_TOUNIT_MAX: u32 = 0xe_0000;

    const C1_CLK_EN: u32 = 0x4;
    const C1_CLK_STABLE: u32 = 0x2;

    const SCR_SD_BUS_WIDTH_4: u32 = 0x400;
    const SCR_SUPP_SET_BLKCNT: u32 = 0x0200_0000;

    const SCR_SUPP_CCS: u32 = 0x1;

    const ACMD41_CMD_COMPLETE: u32 = 0x8000_0000;
    const ACMD41_CMD_CCS: u32 = 0x40000000;
    const ACMD41_ARG_HC: u32 = 0x51ff8000;
    const ACMD41_VOLTAGE: u32 = 0x0ff_8000;

    const INT_READ_RDY: u32 = 0x20;


    const HOST_SPEC_V2: u64 = 1;

    pub fn get() -> &'static mut Self {
        unsafe {
            &mut *(Self::EMMC_CONTROLLER_BASE as *mut Self)
        } 
    }

    // Returns true is success
    pub fn sd_status(&mut self, status: StatusSetting, timer: &Timer) -> bool {
        let mut count: u32 = 500_000;

        while self.status.get().get_status(status) 
            && !self.interrupt.get().is_err()
            && count != 0
        {
            count -= 1;
            timer.delay(1);
        }

        if count == 0 {
            return false;
        } else {
            return !self.interrupt.get().is_err();
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

        let interrupt = self.interrupt.get();

        // TODO these should just be bits on the bitfield
        if count <= 0
            || interrupt.is_command_timeout_error()
            || interrupt.is_command_timeout_error()
            || interrupt.is_err()
        {
            // Is this necessary?
            self.interrupt.set(interrupt);
            return false;
        } else {
            self.interrupt.set(Interrupt{value: next_mask});
            return true;
        }
    }

    pub fn sd_command(&mut self, mut command: SDCommand, arg: u32, timer: &Timer) -> u32 {
        //println!("Sending command {:#x}, arg {:#x}", command, arg);
        let mut r = 0;

        if command.get_is_application_specific() == 1 {
            println!("App specific: {:#x}", SDCommand::APPPLICATION_SPECIFIC_COMMAND.value);
            let mut new_command = SDCommand::APPPLICATION_SPECIFIC_COMMAND;
            
            if unsafe{ sd_rca } != 0 {
                new_command = new_command.set_response_type(ResponseType::Response48Bit as u32);
            }
            r = self.sd_command(new_command, unsafe {sd_rca as u32}, timer);

            if(unsafe { sd_rca != 0} && r == 0) {
                panic!("ERROR: failed to send SD APP command");
            }

            command = command.set_is_application_specific(0);
            }

        if !self.sd_status(StatusSetting::CommandInhibit, timer) {
            panic!("ERROR: EMMC busy");
        }
        
        // Do we really need to do this?--It seems redundant
        self.interrupt.set(self.interrupt.get());
        self.arg1.set(arg);
        self.command.set(command);

        if command == SDCommand::SEND_OP_COND {
            timer.delay(1000);
        } else if command == SDCommand::SEND_INTERFACE_CONDITIONS
            || command == SDCommand::APPPLICATION_SPECIFIC_COMMAND {
            timer.delay(100);
        }

        if(!self.sd_int(Self::InterruptCommandDone, timer)) {
            panic!("ERROR: failed to send EMMC command");
        }

        r = self.resp0.get();

        if(command == SDCommand::GO_IDLE || command == SDCommand::APPPLICATION_SPECIFIC_COMMAND) {
            return 0
        } else if command == SDCommand::APPPLICATION_SPECIFIC_COMMAND.set_response_type(ResponseType::Response48Bit as u32) {
            return r & StatusSetting::AppCommand as u32;
        } else if(command == SDCommand::SEND_OP_COND) {
            return r;
        } else if(command == SDCommand::SEND_INTERFACE_CONDITIONS) {
            if r == arg {
                return 0;
            } else {
                return 1;
            }
        } else if (command == SDCommand::SEND_CARD_IDENTIFICATION) {
            r |= self.resp3.get();
            r |= self.resp2.get();
            r |= self.resp1.get();
            return r;
        } else if (command == SDCommand::SEND_RELATIVE_ADDRESS) {
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

        println!("sd_clk divisor {}, shift {}", d, s);
        
        if(unsafe {sd_hv > Self::HOST_SPEC_V2}) {
            h = (d&0x300) >> 2;
        }

        d = (((d & 0x0ff) << 8) | h);

        self.control1.set(
            Control1 {
                value: (self.control1.get().as_u32() & 0xffff_003f) | d,
            }
        );

        println!("Control set for the first time");

        timer.delay(10);

        self.control1.set(
            Control1 {
                value: self.control1.get().as_u32() | Self::C1_CLK_EN
            }
        );

        println!("Control set for the second time");

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

    pub fn sd_init(&mut self, timer: &Timer, gpio: &GPIOController) {
        let cd = Pin::new(47).unwrap();

        println!("Initializing cd");

        gpio.set_mode(cd, Mode::AF3);

        gpio.pull(cd, Pull::Up);
        gpio.set_gphen(cd, 1);

        println!("Done!");

        println!("Initializing clk and cmd");

        let clk = Pin::new(48).unwrap();
        let cmd = Pin::new(49).unwrap();

        gpio.set_mode(clk, Mode::AF3);
        gpio.set_mode(cmd, Mode::AF3);

        gpio.pull(clk, Pull::Up);
        gpio.pull(cmd, Pull::Up);

        println!("Done!");

        println!("Initializing data pins");

        let dat0 = Pin::new(50).unwrap();
        let dat1 = Pin::new(51).unwrap();
        let dat2 = Pin::new(52).unwrap();
        let dat3 = Pin::new(53).unwrap();

        gpio.set_mode(dat0, Mode::AF3);
        gpio.set_mode(dat1, Mode::AF3);
        gpio.set_mode(dat2, Mode::AF3);
        gpio.set_mode(dat3, Mode::AF3);

        gpio.pull(dat0, Pull::Up);
        gpio.pull(dat1, Pull::Up);
        gpio.pull(dat2, Pull::Up);
        gpio.pull(dat3, Pull::Up);

        println!("Pins initialized");

        unsafe {
            sd_hv = ((self.slotisr_ver.get().as_u32() & Self::HOST_SPEC_NUM) >> Self::HOST_SPEC_NUM_SHIFT) as u64;
        }

        println!("sd_hv is: {}", unsafe {sd_hv});

        println!("Resetting card");

        // Resetting the card
        self.control0.set(Control0{value: 0});
        self.control1.set(Control1{
            value: self.control1.get().as_u32() | Self::C1_SRST_HC
        });

        let mut count = 10000;

        timer.delay(10);

        println!("Waiting");

        println!("Control1 is at: {:p}", &self.control1);

        while (self.control1.get().as_u32() & Self::C1_SRST_HC) != 0
        && count > 0 {
            //println!("Control0: {}, Control1: {:#x}", self.control0.get().as_u32(), self.control1.get().as_u32());
            count -= 1;
            timer.delay(10);
        }

        if(count <= 0) {
            panic!("ERROR: failed to reset EMMC");
        }
        
        println!("Reset successful");

        // At this point, reset has succeeded
        self.control1.set(Control1{
            value: self.control1.get().as_u32() | Self::C1_CLK_INTLEN | Self::C1_TOUNIT_MAX
        });

        timer.delay(10);

        // Set clock frequency
        self.sd_clk(400_000, timer);

        println!("irpt_en is at {:p} and irpt_mask is at {:p}", &self.irpt_en, &self.irpt_mask);

        self.irpt_en.set(InterruptEnable{
            value: 0xffff_ffff
        });

        self.irpt_mask.set(InterruptMask{
            value: 0xffff_ffff
        });

        // This should be redundant because of initialization value
        unsafe {
            sd_scr[0] = 0;
            sd_scr[1] = 0;
            sd_rca = 0;
            sd_err = 0;
        }

        println!("Going idle");

        // TODO: this might not be the correct error checking
        if(self.sd_command(SDCommand::GO_IDLE, 0, timer) != 0) {
            panic!("Unable to go idle");
        }

        println!("IF command: {:#x}", SDCommand::SEND_INTERFACE_CONDITIONS.value);
        if(self.sd_command(SDCommand::SEND_INTERFACE_CONDITIONS, 0x1AA, timer) != 0) {
            panic!("Unable to send conditions")
        }

        println!("Done!");

        let mut cnt = 6;
        let mut r = 0;

        while(r & Self::ACMD41_CMD_COMPLETE == 0) && cnt >= 0 {
            cnt -= 1;
            wait_for_cycles(400);

            println!("Sending... {:#x}", SDCommand::SEND_OP_COND.value);
            r = self.sd_command(SDCommand::SEND_OP_COND, Self::ACMD41_ARG_HC, timer);

            println!("returned: {:#x}", r);

            // TODO: check for errors
        }

        if(r & Self::ACMD41_CMD_COMPLETE == 0 || cnt <= 0) {
            panic!("SD timed out");
        }

        if(r & Self::ACMD41_VOLTAGE == 0) {
            panic!("Voltage not set correctly")
        }

        let mut ccs = 0;

        if(r & Self::ACMD41_CMD_CCS == 0) {
            ccs = Self::SCR_SUPP_CCS;
        } 

        println!("CID: {:#x}", SDCommand::SEND_CARD_IDENTIFICATION.value);
        self.sd_command(SDCommand::SEND_CARD_IDENTIFICATION, 0, timer);

        unsafe {
            println!("Rel addr: {:#x}", SDCommand::SEND_RELATIVE_ADDRESS.value);
            sd_rca = self.sd_command(SDCommand::SEND_RELATIVE_ADDRESS, 0, timer) as u64;
        }

        self.sd_clk(25_000_000, timer);

        println!("Clock set");

        println!("Card select: {:#x}", SDCommand::CARD_SELECT.value);
        // TODO error checking
        self.sd_command(SDCommand::CARD_SELECT, unsafe {sd_rca as u32}, timer);

        if(!self.sd_status(StatusSetting::DataInhibit, timer)) {
            panic!("Timeout");
        }

        self.blockSizeAndCount.set(BlockSizeAndCount{ value: (1 << 16) | 8});

        println!("SCR: {:#x}", SDCommand::SEND_SD_CONFIGURATION_REGISTER.value);
        //TODO: check errors
        self.sd_command(SDCommand::SEND_SD_CONFIGURATION_REGISTER, 0, timer);
        
        if(!self.sd_int(Self::INT_READ_RDY, timer)) {
            panic!("Something failed");
        }

        let mut r = 0;
        let mut cnt = 100_000;

        while(r < 2 && cnt > 0) {
            if(self.status.get().as_u32() & StatusSetting::ReadAvailable as u32!= 0) {
                unsafe {
                    sd_scr[r] = self.data.get() as u64;
                    r += 1;
                }
            } else {
                timer.delay(1);
            }
        }

        if(r != 2) {
            panic!("Could not read scr");
        }

        println!("SCR read");
        
        if(unsafe{sd_scr[0] as u32} & Self::SCR_SD_BUS_WIDTH_4 != 0) {
            self.sd_command(SDCommand::SET_BUS_WIDTH, unsafe {sd_rca as u32} | 2, timer);
            self.control0.set(
                Control0 {
                    value: self.control0.get().as_u32() | Self::C0_HCTL_DWIDTH
                }
            );
        }

        unsafe {
            // TODO: figure out types and where negation goes
            sd_scr[0] &= !(Self::SCR_SUPP_CCS as u64);
            sd_scr[0] |= ccs as u64;
        }
    }

    pub fn sd_readblock(&mut self, start: u32, buffer: &mut [u8], num: u32, timer: &Timer) -> u32 {
        let mut r: u32;
        let mut c = 0;
        let mut d: u32;

        let length = buffer.len() / 4;
        // TODO; this is awful
        let buffer = unsafe { core::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u32, length)};

        if(!self.sd_status(StatusSetting::DataInhibit, timer)) {
            panic!("Data is inhibited");
        }

        if(unsafe {sd_scr[0]} as u32 & Self::SCR_SUPP_CCS != 0) {
            if(num > 1 && (unsafe {sd_scr[0]} as u32 & Self::SCR_SUPP_SET_BLKCNT != 0)) {
                self.sd_command(SDCommand::SET_BLOCK_COUNT, num, timer);
            }

            self.blockSizeAndCount.set(BlockSizeAndCount{
                value: (num << 16) | 512
            });

            let command = if num == 1 { SDCommand::READ_SINGLE_BLOCK } else {SDCommand::READ_MULTIPLE_BLOCKS };

            self.sd_command(command, start, timer);
        } else {
            self.blockSizeAndCount.set(BlockSizeAndCount {
                value: (1 << 16) | 512
            })
        }

        let mut buffer_offset = 0;
        while(c < num) {
            if(unsafe {sd_scr[0] as u32 & Self::SCR_SUPP_CCS == 0}) {
                self.sd_command(SDCommand::READ_SINGLE_BLOCK, (start + c), timer);
            }

            if(!self.sd_int(Self::INT_READ_RDY, timer)) {
                panic!("Timeout waiting to read sd");
            }

            for d in 0..128 {
                buffer[buffer_offset + d] = self.data.get();
            }
            
            c += 1;
            buffer_offset += 128;
        }

        if(num > 1 && (unsafe {sd_scr[0] as u32 & Self::SCR_SUPP_SET_BLKCNT} == 0) )
            && unsafe {sd_scr[0] as u32} & Self::SCR_SUPP_CCS != 0
        {
            self.sd_command(SDCommand::STOP_TRANSMISSION, 0, timer);
        }

        return if c!= num {0} else {num *512};
    }
}

bitfield! {
    BlockSizeAndCount(u32) {
        blockSize: 0-9,
        numberOfBlocks: 16-31
    }
}

bitfield! {
    SDCommand(u32) {
        enable_block_counter: 1-1,
        auto_command: 2-3,
        data_direction: 4-4,
        multiple_blocks: 5-5,
        response_type: 16-17,
        check_response_CRC: 19-19,
        check_response_index: 20-20,
        data_transfer: 21-21,
        command_type: 22-23,
        command_index: 24-29,

        // always write as 0. Useful to store metadata
        is_application_specific: 31-31
    } with {
        const GO_IDLE: Self = Self::with_command_index(0);

        const SEND_CARD_IDENTIFICATION: Self = Self::with_command_index(2)
            .set_response_type(ResponseType::Response136Bit as u32);

        const SEND_RELATIVE_ADDRESS: Self = Self::with_command_index(3)
            .set_response_type(ResponseType::Response48Bit as u32);

        const CARD_SELECT: Self = Self::with_command_index(7)
            .set_response_type(ResponseType::Response48BitUsingBusy as u32);

        const SEND_INTERFACE_CONDITIONS: Self = Self::with_command_index(8)
            .set_response_type(ResponseType::Response48Bit as u32);

        const STOP_TRANSMISSION: Self = Self::with_command_index(12)
            .set_response_type(ResponseType::Response48Bit as u32);

        const READ_SINGLE_BLOCK: Self = Self::with_command_index(17)
            .set_response_type(ResponseType::Response48Bit as u32)
            .set_data_direction(1) // Card to host
            .set_data_transfer(1);

        const READ_MULTIPLE_BLOCKS: Self = Self::with_command_index(18)
            .set_response_type(ResponseType::Response48Bit as u32)
            .set_enable_block_counter(1)
            .set_data_direction(1)
            .set_multiple_blocks(1)
            .set_data_transfer(1);

        const SET_BLOCK_COUNT: Self = Self::with_command_index(23)
            .set_response_type(ResponseType::Response48Bit as u32);

        const APPPLICATION_SPECIFIC_COMMAND: Self = Self::with_command_index(55);

        const SET_BUS_WIDTH: Self = Self::with_command_index(6)
            .set_response_type(ResponseType::Response48Bit as u32)
            .set_is_application_specific(1);

        const SEND_OP_COND: Self = Self::with_command_index(41)
            .set_response_type(ResponseType::Response48Bit as u32)
            .set_is_application_specific(1);

        const SEND_SD_CONFIGURATION_REGISTER: Self = Self::with_command_index(51)
            .set_response_type(ResponseType::Response48Bit as u32)
            .set_data_direction(1)
            .set_data_transfer(1)
            .set_is_application_specific(1);

        const fn empty_command() -> Self {
            Self { value: 0 }
        }

        const fn with_command_index(index: u32) -> Self {
            Self::empty_command().set_command_index(index)
        }
    }
}

enum AutoCommand {
    None = 0,
    CMD12 = 0b01,
    CMD23 = 0b10
}

enum ResponseType {
    NoResponse = 0,
    Response136Bit = 01,
    Response48Bit = 10,
    Response48BitUsingBusy = 11
}

enum CommandType {
    Normal = 00,
    Suspend = 01,
    Resume = 10,
    Abort = 11
}

bitfield! {
    Status(u32) {
        command_inhibit: 0-0,
        data_inhibit: 1-1,
        DAT_ACTIVE: 2-2,
        app_command: 5-5,
        WRITE_TRANSFER: 8-8,
        READ_TRANSFER: 9-9,
        read_available: 11-11,
        DAT_LEVEL0: 20-23,
        CMD_LEVEL: 24-24,
        DAT_LEVEL1: 25-28
    } with {
        pub fn as_u32(&self) -> u32 {
            self.value
        }

        pub fn get_status(&self, status: StatusSetting) -> bool {
            match(status) {
                StatusSetting::CommandInhibit => self.get_command_inhibit() != 0,
                StatusSetting::DataInhibit => self.get_data_inhibit() != 0,
                StatusSetting::AppCommand => self.get_app_command() != 0,
                StatusSetting::ReadAvailable => self.get_read_available() != 0
            }
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
    } with {
        pub fn as_u32(&self) -> u32 {
            self.value
        }
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
        command_done: 0-0,
        DATA_DONE: 1-1,
        BLOCK_GAP: 2-2,
        WRITE_RDY: 4-4,
        READ_RDY: 5-5,
        CARD: 8-8,
        RETUNE: 12-12,
        BOOTACK: 13-13,
        ENDBOOT: 14-14,
        ERR: 15-15,
        command_timeout_error: 16-16,
        CCRC_ERR: 17-17,
        CEND_ERR: 18-18,
        CBAD_ERR: 19-19,
        data_timeout_error: 20-20,
        DCRC_ERR: 21-21,
        DEND_ERR: 22-22,
        ACMD_ERR: 24-24
    } with {
        const INTERRUPT_ERROR_MASK: u32 = 0x017E_8000;

        pub fn as_u32(&self) -> u32 {
            self.value
        }

        pub fn is_err(&self) -> bool {
            self.value & Self::INTERRUPT_ERROR_MASK != 0
        }

        pub fn is_command_done(&self) -> bool {
            self.get_command_done() != 0
        }

        pub fn is_command_timeout_error(&self) -> bool {
            self.get_command_timeout_error() != 0
        }

        pub fn is_data_timeout_error(&self) -> bool {
            self.get_data_timeout_error() != 0
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
    } with {
        pub fn as_u32(&self) -> u32 {
            self.value
        }
    }
}