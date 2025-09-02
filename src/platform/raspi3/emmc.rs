use core::cell::{RefCell, RefMut};

use crate::volatile::Volatile;
use crate::bitfield;
use crate::device::timer::Timer;
use super::gpio::{GPIOController, Pin, Pull, Mode};
use crate::aarch64::cpu::wait_for_cycles;
use crate::device::sector_device::{
    Sector,
    SectorAddress,
    SectorDevice
};
use alloc::rc::Rc;

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

pub struct EMMCConfiguration {
    configuration: SDConfigurationRegister,
    relative_card_address: u32,
    hardware_version: u32
}

impl EMMCConfiguration {
    pub const fn new() -> Self {
        Self {
            configuration: SDConfigurationRegister::uninitialized(),
            relative_card_address: 0,
            hardware_version: 0
        }
    }
}

pub trait EMMCSlot {
    fn wait_for_interrupt(&self, interrupt: InterruptType) -> Result<(), &str>;

    fn wait_for_status(&self, status: StatusSetting) -> Result<(), &str>;

    fn wait_for_command_response(&self) -> Result<u32, &str>;

    fn parse_response(&self, response: u32, command: SDCommand, argument: u32) -> Result<u32, &str>;


    fn send_command(&self, command: SDCommand, argument: u32);


    fn is_clock_enabled(&self) -> bool;

    fn enable_clock(&self);

    fn disable_clock(&self);

    fn configure_clock(&self, d: u32);

    fn configure_internal_clocks(&self, data_timeout_exponent: u32);
    
    fn start_reset(&self);

    fn is_reset_complete(&self) -> bool;

    
    fn enable_interrupts(&self);

    
    fn set_block_size_and_count(&self, size: u32, count: u32);

    fn is_read_available(&self) -> bool;

    fn read_data(&self) -> u32;


    fn get_host_controller_specification_version(&self) -> u32;

    
    fn use_four_data_lines(&self);

    fn is_inhibited(&self) -> bool;

    fn rewrite_interrupt(&self);
}

pub struct EMMCController<'a> {
    slot: &'a RefCell<EMMCRegisters>,
    gpio: &'a dyn GPIOController,
    timer: &'a dyn Timer,

    configuration: EMMCConfiguration
}

impl<'a> SectorDevice for EMMCController<'a> {
    fn read_sector(&mut self, address: SectorAddress) -> Sector {
        let mut buffer: [u8; 512] = [0; 512];

        self.read_blocks(address, &mut buffer, 1);

        Sector::from(buffer)
    }
}

impl<'a> EMMCController<'a> {
    pub fn new(registers: &'a RefCell<EMMCRegisters>,
        gpio: &'a dyn GPIOController,
        timer: &'a dyn Timer
    ) -> Self {
        Self {
            slot: registers, gpio, timer,
            configuration: EMMCConfiguration::new()
        }
    }

    fn send_application_specific_command(&mut self) -> Result<(), &str> {
        let mut application_specific_command = SDCommand::APPPLICATION_SPECIFIC_COMMAND;

        let relative_card_address = self.configuration.relative_card_address;

        if relative_card_address != 0 {
            application_specific_command = application_specific_command.set_response_type(
                ResponseType::Response48Bit as u32
            );
        }

        let result = self.send_command(application_specific_command, self.configuration.relative_card_address);

        match result {
            Err(error) => Err(error),
            Ok(value) => {
                if relative_card_address != 0 && value == 0 {
                    Err("ERROR: failed to send SD APP command.")
                } else {
                    Ok(())
                }
            }
        }
    }

    fn send_command(&mut self, mut command: SDCommand, argument: u32) -> Result<u32, &str> {

        if command.get_is_application_specific() == 1 {
            self.send_application_specific_command();

            command = command.set_is_application_specific(0);
        }

        self.slot.borrow().wait_for_status(StatusSetting::CommandInhibit).expect("ERROR: EMMC busy");

        // TODO: is this necessary?
        self.slot.borrow_mut().rewrite_interrupt();

        self.slot.borrow_mut().send_command(command, argument);

        /* TODO: Do we really need this delay? */
        if command == SDCommand::SEND_OP_COND {
            self.timer.delay_millis(1000);
        } else if command == SDCommand::SEND_INTERFACE_CONDITIONS
            || command == SDCommand::APPPLICATION_SPECIFIC_COMMAND {
            self.timer.delay_millis(100);
        }

        let response = self.slot.borrow().wait_for_command_response().unwrap();

        return self.slot.borrow().parse_response(response, command, argument)
    }

    const SET_CLOCK_FREQUENCY_TIMEOUT: u32 = 100_000;

    fn wait_until_uninhibited(&self) -> Result<(), &str> {
        let mut count = Self::SET_CLOCK_FREQUENCY_TIMEOUT;

        while self.slot.borrow().is_inhibited()
            || count > 0 {
                self.timer.delay_millis(1);
                count -= 1;
            }

        if count <= 0 {
            return Err("Time out waiting for card to be uninhibited");
        } else {
            return Ok(());
        }
    }

    fn calculate_d(&self, f: u32) -> u32 {
        let mut d: u32;
        let c = 41666666/f;
        let mut x: u32;
        let mut s = 32;
        let mut h = 0;
        
        x = c - 1;
        if x == 0  {
            s = 0;
        } else {
            if (x & 0xffff0000) == 0 { x <<= 16; s -= 16; }
            if (x & 0xff000000) == 0 { x <<= 8;  s -= 8; }
            if (x & 0xf0000000) == 0 { x <<= 4;  s -= 4; }
            if (x & 0xc0000000) == 0 { x <<= 2;  s -= 2; }
            if (x & 0x80000000) == 0 { x <<= 1;  s -= 1; }
            if s>0 {
                s -= 1;
            }
            if s>7 {
                s=7;
            }
        }

        if self.configuration.hardware_version > EMMCRegisters::HOST_SPEC_V2 {
            d = c;
        } else {
            d = 1 << s;
        }

        if d <= 2  {
            d = 2;
            s = 0;
        }

        if self.configuration.hardware_version > EMMCRegisters::HOST_SPEC_V2  {
            h = (d&0x300) >> 2;
        }

        d =  ((d & 0x0ff) << 8) | h;

        return d;
    }

    fn set_clock_frequency(&mut self, f: u32) {
        self.wait_until_uninhibited(); 

        self.slot.borrow_mut().disable_clock();
        self.timer.delay_micros(10);

        self.slot.borrow_mut().configure_clock(self.calculate_d(f));

        self.timer.delay_micros(10);

        self.slot.borrow_mut().enable_clock();

        self.timer.delay_micros(10);

        let mut count = 10_000;

        while  !self.slot.borrow().is_clock_enabled() && count > 0 {
            count -= 1;
            self.timer.delay_micros(10);
        }

        if count <= 0  {
            panic!("ERROR: failed to get stable clock");
        }
    }

    fn initialize_pins(gpio: &dyn GPIOController) {
        let cd = Pin::new(47).unwrap();

        gpio.set_pin_mode(cd, Mode::AF3);

        gpio.set_pin_pull(cd, Pull::Up);
        gpio.set_pin_high_detect_enable(cd);

        let clk = Pin::new(48).unwrap();
        let cmd = Pin::new(49).unwrap();

        gpio.set_pin_mode(clk, Mode::AF3);
        gpio.set_pin_mode(cmd, Mode::AF3);

        gpio.set_pin_pull(clk, Pull::Up);
        gpio.set_pin_pull(cmd, Pull::Up);

        let dat0 = Pin::new(50).unwrap();
        let dat1 = Pin::new(51).unwrap();
        let dat2 = Pin::new(52).unwrap();
        let dat3 = Pin::new(53).unwrap();

        gpio.set_pin_mode(dat0, Mode::AF3);
        gpio.set_pin_mode(dat1, Mode::AF3);
        gpio.set_pin_mode(dat2, Mode::AF3);
        gpio.set_pin_mode(dat3, Mode::AF3);

        gpio.set_pin_pull(dat0, Pull::Up);
        gpio.set_pin_pull(dat1, Pull::Up);
        gpio.set_pin_pull(dat2, Pull::Up);
        gpio.set_pin_pull(dat3, Pull::Up);

    }

    fn reset_card(&mut self) {
        self.slot.start_reset();

        let mut count = 10000;

        self.timer.delay_micros(10);

        while !self.slot.borrow().is_reset_complete()
        && count > 0 {
            count -= 1;
            self.timer.delay_micros(10);
        }

        if count <= 0 {
            panic!("ERROR: failed to reset EMMC");
        }
    }

    /*pub fn initialize(registers: RefMut<&'a mut EMMCRegisters>, timer: &'a dyn Timer, gpio: &'a dyn GPIOController) -> EMMCConfiguration {
        let mut emmc_controller = Self::new(registers, gpio, timer);

        Self::initialize_pins(gpio);

        emmc_controller.initialize_card();

        return emmc_controller.configuration;
    }*/

    pub fn initialize_card(&mut self) {
        self.configuration.hardware_version = self.slot.get_host_controller_specification_version();

        self.reset_card();

        // At this point, reset has succeeded
        self.slot.configure_internal_clocks(0b1110);
        
        self.timer.delay_micros(10);

        // Set clock frequency
        self.set_clock_frequency(400_000);

        self.slot.enable_interrupts();

        // TODO: this might not be the correct error checking
        if self.send_command(SDCommand::GO_IDLE, 0).unwrap() != 0 {
            panic!("Unable to go idle");
        }

        if self.send_command(SDCommand::SEND_INTERFACE_CONDITIONS, 0x1AA).unwrap() != 0 {
            panic!("Unable to send conditions")
        }

        let mut cnt = 6;
        let mut acmd41_response = ACMD41Response::empty();

        while acmd41_response.get_complete() == 0 && cnt >= 0 {
            cnt -= 1;
            wait_for_cycles(400);

            acmd41_response = ACMD41Response::from(
                self.send_command(SDCommand::SEND_OP_COND, EMMCRegisters::ACMD41_ARG_HC).unwrap()
            );


            // TODO: check for errors
        }

        if acmd41_response.get_complete() == 0 || cnt <= 0 {
            panic!("SD timed out");
        }

        if acmd41_response.get_voltage() == 0 {
            panic!("Voltage not set correctly")
        }

        let mut command_support_bits = 0;

        if acmd41_response.get_command_support_bits() == 0 {
            command_support_bits = EMMCRegisters::SCR_SUPP_CCS;
        } 

        self.send_command(SDCommand::SEND_CARD_IDENTIFICATION, 0).unwrap();

        self.configuration.relative_card_address = self.send_command(SDCommand::SEND_RELATIVE_ADDRESS, 0).unwrap();

        self.set_clock_frequency(25_000_000);


        // TODO error checking
        self.send_command(SDCommand::CARD_SELECT, self.configuration.relative_card_address as u32).unwrap();

        self.slot.borrow().wait_for_status(StatusSetting::DataInhibit).expect("ERROR: Timeout");

        self.slot.set_block_size_and_count(8, 1);

        self.configuration.configuration = self.read_configuration().unwrap();

        if self.configuration.configuration.get_spec_version_4_or_higher() != 0 {
            self.send_command(SDCommand::SET_BUS_WIDTH, self.configuration.relative_card_address as u32 | 2).unwrap();
            self.slot.use_four_data_lines();
        }

        self.configuration.configuration = self.configuration.configuration.set_command_support_bits(command_support_bits as u64);

    }
    
    pub fn read_blocks(&mut self, start: u32, buffer: &mut [u8], num: u32) -> u32 {
        let mut c = 0;

        let length = buffer.len() / 4;
        // TODO; this is awful
        let buffer = unsafe { core::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u32, length)};

        self.slot.wait_for_status(StatusSetting::DataInhibit).expect("Data is inhibited?");

        if self.configuration.configuration.get_command_support_bits() != 0 {
            if num > 1 && self.configuration.configuration.get_support_set_block_count() != 0 {
                self.send_command(SDCommand::SET_BLOCK_COUNT, num).unwrap();
            }

            self.slot.set_block_size_and_count(512, 16);

            let command = if num == 1 { SDCommand::READ_SINGLE_BLOCK } else {SDCommand::READ_MULTIPLE_BLOCKS };

            self.send_command(command, start).unwrap();
        } else {
            self.slot.set_block_size_and_count(512, 1);
        }

        let mut buffer_offset = 0;
        while c < num {
            if self.configuration.configuration.get_command_support_bits() == 0 {
                self.send_command(SDCommand::READ_SINGLE_BLOCK,  start + c).unwrap();
            }

            self.slot.wait_for_interrupt(InterruptType::ReadReady).expect("Timeout waiting to read sd");

            for d in 0..128 {
                buffer[buffer_offset + d] = self.slot.read_data();
            }
            
            c += 1;
            buffer_offset += 128;
        }

        if num > 1
            && self.configuration.configuration.get_support_set_block_count() == 0
            && self.configuration.configuration.get_command_support_bits() != 0
        {
            self.send_command(SDCommand::STOP_TRANSMISSION, 0).unwrap();
        }

        return if c!= num {0} else {num *512};
    }

    fn read_configuration(&mut self) -> Result<SDConfigurationRegister, &str> {    //TODO: check errors
        self.send_command(SDCommand::SEND_SD_CONFIGURATION_REGISTER, 0).unwrap();
        
        self.slot.wait_for_interrupt(InterruptType::ReadReady).expect("Something failed");

        let mut r = 0;
        let mut cnt = 100_000;
        let mut scr: [u32; 2] = [0, 0];

        while r < 2 && cnt > 0 {
            cnt -= 1;
            if self.slot.is_read_available() {
                // Note: Little Endian
                scr[r] = self.slot.read_data();
                r += 1;
            }
        }

        if r != 2 {
            return Err("Could not read scr register");
        }

        // TODO: error handling
        let configuration = SDConfigurationRegister::from_array(scr);

        return Ok(configuration)
    }
}

#[repr(C)]
pub struct EMMCRegisters {
    pub arg2: Volatile<u32>,
    pub block_size_and_count: Volatile<BlockSizeAndCount>,
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
    pub irpt_mask: Volatile<Interrupt>,
    pub irpt_en: Volatile<Interrupt>,
    pub control2: Volatile<u32>,
    pub force_irpt: Volatile<Interrupt>,
    pub boot_timeout: Volatile<u32>,
    pub dbg_sgl: Volatile<DebugSelect>,
    pub exrdfifo_cfg: Volatile<ExrdfifoCfg>,
    pub exrdfifo_en: Volatile<ExrdfifoEn>,
    pub tune_step: Volatile<TuneStep>,
    pub tune_steps_std: Volatile<TuneStepsStd>,
    pub tune_steps_ddr: Volatile<TuneStepsDdr>,
    pub spi_int_spt: Volatile<SpiIntSpt>,
    pub slotisr_ver: Volatile<SlotInterruptStatusAndVersion>
}

impl EMMCRegisters {
    const SCR_SUPP_CCS: u32 = 0x1;

    const ACMD41_ARG_HC: u32 = 0x51ff8000;

    const HOST_SPEC_V2: u32 = 1;

    const STATUS_TRIES: u32 = 500_000;

    fn wait_for_status(&mut self, status: StatusSetting) -> Result<(), &str> {
        for _ in 0..Self::STATUS_TRIES {
            if self.interrupt.get().is_err() {
                return Err("Interrupt error");
            }

            if !self.status.get().get_status(status) {
                return Ok(());
            }
        }

        Err("Timed out while waiting for status")
    }
    
    const INTERRUPT_WAIT_TIMEOUT: u32 = 1_000_000;

    pub fn wait_for_interrupt(&mut self, interrupt_type: InterruptType) -> Result<(), &str> {
        for _ in 0..Self::INTERRUPT_WAIT_TIMEOUT {
            let interrupt = self.interrupt.get();
            if interrupt.is_interrupt_triggered(interrupt_type) {
                // TODO: check error handling
                if interrupt.is_command_timeout_error()
                    || interrupt.is_data_timeout_error()
                    || interrupt.is_err()
                {
                    self.interrupt.set(interrupt);
                    return Err("Error in interrupt");
                } else {
                    self.interrupt.set(Interrupt::new().set_interrupt_mask(interrupt_type));
                    return Ok(());
                }
            }
        }

        return Err("Timed out waiting for interrupt");
    }

    fn parse_response(&self, response: u32, command: SDCommand, argument: u32) -> Result<u32, &str> {
        if command == SDCommand::GO_IDLE
            || command == SDCommand::APPPLICATION_SPECIFIC_COMMAND  {
            return Ok(0);
        } else if command == SDCommand::APPPLICATION_SPECIFIC_COMMAND
            .set_response_type(ResponseType::Response48Bit as u32) {
            return Ok(response & StatusSetting::AppCommand as u32);
        } else if command == SDCommand::SEND_OP_COND  {
            return Ok(response);
        } else if command == SDCommand::SEND_INTERFACE_CONDITIONS  {
            if response == argument {
                return Ok(0);
            } else {
                return Err("?");
            }
        } else if  command == SDCommand::SEND_CARD_IDENTIFICATION  {
            let response = response
                | self.resp3.get()
                | self.resp2.get()
                | self.resp1.get();
            return Ok(response);
        } else if  command == SDCommand::SEND_RELATIVE_ADDRESS  {
            return Ok(response & CommandFlag::RcaMask as u32);
        }

        // What does this case mean?
        return Ok(response & CommandFlag::ErrorsMask as u32);
    }

    fn wait_for_response(&mut self) -> Result<u32, &str> {
        self.wait_for_interrupt(InterruptType::CommandDone).expect("ERROR: Error while waiting for command response.");

        Ok(self.resp0.get()) 
    }

    fn enable_interrupts(&mut self) {
        self.irpt_en.set(Interrupt::ALL_ENABLED);
        self.irpt_mask.set(Interrupt::ALL_ENABLED);
    }

    fn disable_clock(&mut self) {
        self.control1.map(|control1|
            control1.set_clock_enabled(0)
        )
    }

    fn enable_clock(&mut self) {
        self.control1.map(|control1|
            control1.set_clock_enabled(1)
        )
    }

    fn is_clock_enabled(&self) -> bool {
        self.control1.get().get_clock_enabled() == 1
    }

    fn configure_clock(&mut self, d: u32) {
        // TODO: should this be a set or a map?
        self.control1.set(
            Control1 {
                value: (self.control1.get().as_u32() & 0xffff_003f) | d,
            }
        );
    }

    fn start_reset(&mut self) {
        self.control0.set(Control0::empty());
        self.control1.map(|control1|
            control1.set_reset_complete_host_circuit(1)
        );
    }

    fn reset_complete(&self) -> bool {
        self.control1.get().get_reset_complete_host_circuit() == 0
    }

    fn send_command(&mut self, command: SDCommand, argument: u32) {
        self.arg1.set(argument);
        self.command.set(command);
    }
}

bitfield! {
    BlockSizeAndCount(u32) {
        block_size: 0-9,
        number_of_blocks: 16-31
    } with {
        fn empty() -> Self {
            Self { value: 0 }
        }
        pub fn with_size_and_count(size: u32, count: u32) -> Self {
            Self::empty().set_block_size(size).set_number_of_blocks(count)
        }
    }
}

bitfield! {
    SDCommand(u32) {
        enable_block_counter: 1-1,
        auto_command: 2-3,
        data_direction: 4-4,
        multiple_blocks: 5-5,
        response_type: 16-17,
        check_response_crc: 19-19,
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
        data_active: 2-2,
        app_command: 5-5,
        write_transfer: 8-8,
        read_transfer: 9-9,
        read_available: 11-11,
        data_level0: 20-23,
        command_level: 24-24,
        data_level1: 25-28
    } with {
        pub fn as_u32(&self) -> u32 {
            self.value
        }

        pub fn get_status(&self, status: StatusSetting) -> bool {
            match status {
                StatusSetting::CommandInhibit => self.get_command_inhibit() != 0,
                StatusSetting::DataInhibit => self.get_data_inhibit() != 0,
                StatusSetting::AppCommand => self.get_app_command() != 0,
                StatusSetting::ReadAvailable => self.get_read_available() != 0
            }
        }

        pub fn is_inhibited(&self) -> bool {
            self.get_command_inhibit() == 1
                || self.get_data_inhibit() == 1
        }
    }
}

bitfield! {
    Control0(u32) {
        use_four_data_lines: 1-1,
        hctl_hs_en: 2-2,
        hctl_8bit: 5-5,
        gap_stop: 16-16,
        gap_restart: 17-17,
        readwait_en: 18-18,
        gap_ien: 19-19,
        spi_mode: 20-20,
        boot_en: 21-21,
        alt_boot_en: 22-22
    } with {
        pub fn as_u32(&self) -> u32 {
            self.value
        }

        pub fn empty() -> Self {
            Self { value: 0 }
        }
    }
}

bitfield! {
    Control1(u32) {
        enable_internal_clocks: 0-0,
        clock_stable: 1-1,
        clock_enabled: 2-2,
        clk_gensel: 5-5,
        clk_freq_ms2: 6-7,
        clk_freq8: 8-15,
        data_timeout_unit_exponent: 16-19,
        reset_complete_host_circuit: 24-24,
        srst_cmd: 25-25,
        srst_data: 26-26
    } with {
        pub fn as_u32(&self) -> u32 {
            self.value
        }
    }
}

#[derive(Copy, Clone)]
pub enum InterruptType {
    CommandDone,
    ReadReady
}

bitfield! {
    Interrupt(u32) {
        command_done: 0-0,
        data_done: 1-1,
        block_gap: 2-2,
        write_ready: 4-4,
        read_ready: 5-5,
        card: 8-8,
        retune: 12-12,
        bootack: 13-13,
        end_boot: 14-14,
        error: 15-15,
        command_timeout_error: 16-16,
        ccrc_err: 17-17,
        cend_err: 18-18,
        cbad_err: 19-19,
        data_timeout_error: 20-20,
        dcrc_err: 21-21,
        dend_err: 22-22,
        acmd_err: 24-24
    } with {
        const INTERRUPT_ERROR_MASK: u32 = 0x017E_8000;

        const ALL_ENABLED: Self = Self { value: 0xffff_ffff };

        pub fn new() -> Self {
            Self { value: 0 }
        }

        pub fn as_u32(&self) -> u32 {
            self.value
        }

        pub fn get_interrupt_error_status(&self) -> u32 {
            self.value & Self::INTERRUPT_ERROR_MASK
        }

        pub fn set_interrupt_error_status(&self, value: u32) -> Self {
            Self { value: self.value | value }
        }

        pub fn is_err(&self) -> bool {
            self.get_interrupt_error_status() != 0
        }

        pub fn is_interrupt_triggered(&self, interrupt_type: InterruptType) -> bool {
            match interrupt_type {
                InterruptType::CommandDone => self.get_command_done() == 1,
                InterruptType::ReadReady => self.get_read_ready() == 1
            }
        }

        pub fn set_interrupt_mask(&self, interrupt_type: InterruptType) -> Self {
            match interrupt_type {
                InterruptType::CommandDone => self.set_command_done(1),
                InterruptType::ReadReady => self.set_read_ready(1)
            }
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
    Control2(u32) {
        acnox_err: 0-0,
        acto_err: 1-1,
        accrc_err: 2-2,
        acend_err: 3-3,
        acbad_err: 4-4,
        notc12_err: 7-7,
        uhsmode: 16-18,
        tuneon: 22-22,
        tuned: 23-23
    }
}

bitfield! {
    DebugSelect(u32) {
        select: 0-0
    }
}

bitfield! {
    ExrdfifoCfg(u32) {
        rd_thrsh: 0-2
    }
}

bitfield! {
    ExrdfifoEn(u32) {
        enable: 0-0
    }
}

bitfield! {
    TuneStep(u32) {
        delay: 0-2
    }
}

bitfield! {
    TuneStepsStd(u32) {
        steps: 0-5
    }
}

bitfield! {
    TuneStepsDdr(u32) {
        steps: 0-5
    }
}

bitfield! {
    SpiIntSpt(u32) {
        select: 0-7
    }
}

bitfield! {
    SlotInterruptStatusAndVersion(u32) {
        vendor_version_number: 24-31,
        host_controller_specification_version: 16-23,
        slot_status: 0-7
    }
}

bitfield! {
    SDConfigurationRegister(u64) {
        command_support_bits: 32-35,
        spec_version_4_or_higher: 42-42,
        extended_security_support: 43-46,
        spec_version_3_or_higher: 47-47,
        dat_bus_widths_supported: 48-51,
        cprm_security_support: 52-54,
        data_status_after_erases: 55-55,
        sd_card_spec_version: 56-59,
        support_set_block_count: 57-57,
        scr_structure: 60-63    
    } with {
        pub const fn uninitialized() -> Self {
            Self { value: 0 }
        }
        pub fn from_array(values: [u32; 2]) -> Self {
            Self {
                value: (values[0] as u64) << 32 | (values[1] as u64)
            }
        }
    }
}

bitfield! {
    ACMD41Response(u32) {
        command_support_bits: 30-30,
        complete: 31-31,
        voltage: 15-23
    } with {
        pub fn from(value: u32) -> Self {
            Self {value}
        }

        pub fn empty() -> Self {
            Self { value: 0 }
        }
    }
}