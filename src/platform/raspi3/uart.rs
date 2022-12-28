use super::{
    gpio::{GPIOController, Mode, Pin},
    mmio::MMIOController,
};
use crate::{aarch64::cpu, sync::SpinMutex};
use core::{
    fmt,
    fmt::{Arguments, Error, Write},
    arch::asm,
};

pub static mut CONSOLE: SpinMutex<Option<UARTController>> = SpinMutex::new(None);

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        unsafe {
            CONSOLE.execute(|console|
                match console {
                    Some(console) => console.writef(format_args!($($arg)*)),
                    None => panic!("Print to Uninitialized Console")
                }
            );
        }
    };
}

#[macro_export]
macro_rules! println {
    () => {
        unsafe {
            CONSOLE.execute(|console|
                match console {
                    Some(console) => console.newline(),
                    None => panic!("Print to Uninitialized Console")
                }
            );
        }
    };
    ($($arg:tt)*) => {
        unsafe {
            CONSOLE.execute(|console|
                match console {
                    Some(console) => console.writefln(format_args!($($arg)*)),
                    None => panic!("Print to Uninitialized Console")
                }
            );
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
pub struct UARTController<'a> {
    gpio: &'a GPIOController<'a>,
    mmio: &'a MMIOController,
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

impl<'a> Write for UARTController<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        self.write(s);
        Ok(())
    }
}

impl<'a> UARTController<'a> {
    pub fn new(gpio: &'a GPIOController, mmio: &'a MMIOController) -> Self {
        Self {
            gpio,
            mmio,
            config: UARTConfig::new(),
        }
    }

    pub fn init(gpio: &'a GPIOController, mmio: &'a MMIOController) -> Self {
        //Enable UART
        mmio.write_at_offset(
            mmio.read_at_offset(AUX_ENABLE as usize) | 1,
            AUX_ENABLE as usize,
        );

        //Disable Tx and Rx
        mmio.write_at_offset(0, AUX_MU_CNTL as usize);

        //Set data format to 8 bit
        mmio.write_at_offset(0b11, AUX_MU_LCR as usize);

        //Set rts line high
        mmio.write_at_offset(0, AUX_MU_MCR as usize);

        //Disable interrupts
        mmio.write_at_offset(0, AUX_MU_IER as usize);

        //Clear fifo bits
        mmio.write_at_offset(0b11000110, AUX_MU_IIR as usize);

        //Set baud rate to 115,200
        mmio.write_at_offset(270, AUX_MU_BAUD as usize);

        let tx = Pin::new(14).unwrap();
        let rx = Pin::new(15).unwrap();

        gpio.set_mode(tx, Mode::AF5);
        gpio.set_mode(rx, Mode::AF5);

        //enable Tx and Rx
        mmio.write_at_offset(3, AUX_MU_CNTL as usize);

        Self {
            gpio,
            mmio,
            config: UARTConfig::new(),
        }
    }

    pub fn putc(&self, c: char) {
        while self.mmio.read_at_offset(AUX_MU_LSR as usize) & 0b100000 == 0 {
            unsafe {
                asm!("nop");
            }
        }
        self.mmio.write_at_offset(c as u32, AUX_MU_IO as usize);
    }

    pub fn newline(&mut self) {
        self.putc('\n');
        self.putc('\r');
    }

    #[allow(dead_code)]
    pub fn write(&self, s: &str) {
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
        fmt::write(self, args);
    }

    pub fn writefln(&mut self, args: Arguments) {
        self.update_debug();
        #[allow(unused_must_use)]
        fmt::write(self, args);
        self.newline();
    }

    #[allow(dead_code)]
    pub fn read(&self) -> Result<char, ()> {
        while self.mmio.read_at_offset(AUX_MU_LSR as usize) & 0b1 == 0 {
            unsafe {
                asm!("nop");
            }
        }

        core::char::from_u32(self.mmio.read_at_offset(AUX_MU_IO as usize)).ok_or(())
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
