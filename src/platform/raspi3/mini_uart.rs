use super::{
    gpio::{GPIOController, Mode, Pin},
};
use crate::{aarch64::cpu, sync::SpinMutex, volatile::Volatile, bitfield};

use core::{
    arch::asm, cell::RefCell, fmt::{self, Arguments, Error, Write}
};

use alloc::rc::Rc;

#[repr(C)]
struct MiniUARTRegisters {
    interrupt: Volatile<InterruptSource>,
    enables: Volatile<InterruptEnable>,
    res: [u8; 56],
    io_data: Volatile<MiniUARTIO>,
    interrupt_enable: Volatile<MiniUARTInterruptEnable>,
    interrupt_identify: Volatile<MiniUARTInterruptStatus>,
    line_control: Volatile<LineControl>,
    modem_control: Volatile<ModemControl>,
    line_status: Volatile<LineStatus>,
    modem_status: Volatile<ModemStatus>,
    scratch: Volatile<Scratch>,
    extra_control: Volatile<ExtraControl>,
    extra_status: Volatile<ExtraStatus>,
    baud_rate: Volatile<BaudRate>
}

impl MiniUARTRegisters {
    const MINI_UART_REGISTER_BASE: usize = 0x3F21_5000;
    pub const fn get() -> &'static mut Self {
        unsafe {
            &mut *{Self::MINI_UART_REGISTER_BASE as *mut Self}
        }
    }
}

const UART_BASE_OFFSET: u32 = 0x215000;

const AUX_ENABLE: u32 = UART_BASE_OFFSET + 0x4;

const AUX_MU_IO: u32 = UART_BASE_OFFSET + 0x40;
const AUX_MU_IER: u32 = UART_BASE_OFFSET + 0x44;
const AUX_MU_IIR: u32 = UART_BASE_OFFSET + 0x48;
const AUX_MU_LCR: u32 = UART_BASE_OFFSET + 0x4c;
const AUX_MU_MCR: u32 = UART_BASE_OFFSET + 0x50;
const AUX_MU_LSR: u32 = UART_BASE_OFFSET + 0x54;
const AUX_MU_CNTL: u32 = UART_BASE_OFFSET + 0x60;
const AUX_MU_BAUD: u32 = UART_BASE_OFFSET + 0x68;

#[allow(dead_code)]
pub struct MiniUARTController<'a> {
    gpio: Rc<RefCell<GPIOController>>,
    registers: &'a mut MiniUARTRegisters,
    config: UARTConfig,
}

pub struct UARTConfig {
    level: LogLevel,
    lines: u64,
}

impl UARTConfig {
    pub const fn new() -> Self {
        Self {
            level: LogLevel::Plain,
            lines: 0,
        }
    }
}

#[derive(PartialEq)]
pub enum LogLevel {
    Plain,
    Debug,
}

impl<'a> Write for MiniUARTController<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        self.write(s);
        Ok(())
    }
}

impl<'a> MiniUARTController<'a> {
    pub const fn new(gpio: Rc<RefCell<GPIOController>>) -> Self {
        Self {
            gpio,
            registers: MiniUARTRegisters::get(),
            config: UARTConfig::new(),
        }
    }

    pub fn init(&mut self) {
        self.registers.enables.map(|enables|
            enables.set_mini_uart(1)
        );

        self.registers.extra_control.set(ExtraControl::empty());

        // TODO: fix
        // Data is 8 bit
        self.registers.line_control.set(LineControl{value: 0b11});

        self.registers.modem_control.map(|modem_control|
            modem_control.set_request_to_send(0)
        );

        // Disable Interrupts
        self.registers.interrupt_enable.set(MiniUARTInterruptEnable::enabled());

        // Clear fifo bits
        self.registers.interrupt_identify.map(|line_control|
            line_control.set_interrupt_id(0b11)
                .set_fifo_enables(0b11)
        );

        self.registers.baud_rate.set(BaudRate::with_baud_rate(270));

        let tx = Pin::new(14).unwrap();
        let rx = Pin::new(15).unwrap();


        self.gpio.borrow_mut().set_mode(tx, Mode::AF5);
        self.gpio.borrow_mut().set_mode(rx, Mode::AF5);

        self.registers.extra_control.set(ExtraControl::enabled());
    }

    pub fn putc(&mut self, c: char) {
        while self.registers.line_status.get().get_transmitter_empty() == 0 {
            unsafe {
                asm!("nop");
            }
        }

        // TODO: update to use registers
        self.registers.io_data.set(MiniUARTIO::with_data(c));
    }

    pub fn newline(&mut self) {
        self.putc('\n');
        self.putc('\r');
    }

    #[allow(dead_code)]
    pub fn write(&mut self, s: &str) {
        for c in s.chars() {
            self.putc(c);
        }
    }

    pub fn writeln(&mut self, s: &str) {
        self.update_debug();
        self.write(s);
        self.newline();
    }

    #[allow(dead_code)]
    pub fn writef(&mut self, args: Arguments) {
        #[allow(unused_must_use)]
        let _ = fmt::write(self, args); // TODO: Handle errors?
    }

    pub fn writefln(&mut self, args: Arguments) {
        self.update_debug();
        #[allow(unused_must_use)]
        let _ = fmt::write(self, args); // TODO: Handle errors
        self.newline();
    }

    #[allow(dead_code)]
    pub fn read(&self) -> Result<char, ()> {
        unimplemented!();
        /* TODO: is this the right check?
        while self.mmio.borrow().read_at_offset(AUX_MU_LSR as usize) & 0b1 == 0 {
            unsafe {
                asm!("nop");
            }
        }
        */

        core::char::from_u32(self.registers.io_data.get().get_data()).ok_or(())
    }

    pub fn set_log_level(&mut self, level: LogLevel) {
        self.config.level = level;
    }

    fn update_debug(&mut self) {
        self.config.lines += 1;
        if self.config.level == LogLevel::Debug {
            let lines = self.config.lines;
            self.writef(format_args!(
                "{}](EL{}@{}): ",
                lines,
                cpu::el(),
                cpu::core_id()
            ));
        }
    }
}

bitfield! {
    InterruptSource(u32) {
        mini_uart: 0-0,
        spi1: 1-1,
        spi2: 2-2
    }
}

bitfield! {
    InterruptEnable(u32) {
        mini_uart: 0-0,
        spi1: 1-1,
        spi2: 2-2
    } with {
        pub fn disabled() -> Self {
            Self { value: 0}
        }
    }
}

bitfield! {
    MiniUARTIO(u32) {
        data: 0-7,
        baud_rate_lower_half: 0-7
    } with {
        pub fn with_data(data: char) -> Self {
            Self {
                value: data as u32
            }
        }
    }
}

bitfield! {
    MiniUARTInterruptEnable(u32) {
        received_enabled: 0-0,
        transmitted_enabled: 1-1,
        baud_rate_upper_half: 0-7
    } with {
        pub fn disabled() -> Self {
            Self { value: 0 }
        }

        pub fn enabled() -> Self {
            Self { value: 0b01 }
        }
    }
}

bitfield! {
    MiniUARTInterruptStatus(u32) {
        interrupt_pending: 0-0,
        interrupt_id: 1-2,
        fifo_enables: 6-7
    }
}

bitfield! {
    LineControl(u32) {
        data_size: 0-1,
        break_condition: 6-6,
        dlab_access: 7-7
    }
}

bitfield! {
    ModemControl(u32) {
        request_to_send: 1-1
    }
}

bitfield! {
    LineStatus(u32) {
        data_ready: 0-0,
        receiver_overrun: 1-1,
        transmitter_empty: 5-5,
        transmitter_idle: 6-6
    }
}

bitfield! {
    ModemStatus(u32) {
        clear_to_send: 5-5
    }
}

bitfield! {
    Scratch(u32) {
        scratch: 0-7
    }
}

bitfield! {
    ExtraControl(u32) {
        receiver_enabled: 0-0,
        transmitter_enabled: 1-1
    } with {
        pub fn empty() -> Self {
            Self { value: 0 }
        }

        pub fn enabled() -> Self {
            Self { value: 0b11 }
        }
    }
}

bitfield! {
    ExtraStatus(u32) {

    }
}

bitfield! {
    BaudRate(u32) {
        baud_rate: 0-15
    } with {
        pub fn with_baud_rate(rate: u32) -> Self {
            Self { value: rate }
        }
    }
}