use crate::platform::{self,
    emmc::{self, EMMCController, EMMCRegisters},
    gpio::{GPIOController, GPIORegisters, StatusLight},
    mailbox::{MailboxBuffer, MailboxController, MailboxRegisters}, timer::TimerRegisters
};

use super::{
    mini_uart::MiniUARTRegisters,
    mmio,
};

use core::{cell::{Cell, Ref, RefCell, RefMut, UnsafeCell}, fmt::Arguments, mem::MaybeUninit};
use alloc::rc::Rc;
use alloc::boxed::Box;

use crate::device::{
    console::Console,
    timer::Timer
};

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        PLATFORM.get_console().writef(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
       PLATFORM.get_console().newline();
    };
    ($($args:tt)*) => {
        crate::platform::raspi3::platform_devices::PLATFORM
            .get_console()
            .writefln(format_args!($($args)*));
    }
}

pub const PLATFORM: Platform = Platform::uninitialized();

pub type BoxedDevice<T> = Option<Rc<RefCell<T>>>;

pub struct Platform<'a> {
    devices: Devices<'a>
}

impl<'a> Platform<'a> {
    const fn uninitialized() -> Self {
        Self {
            devices: Devices::uninitialized()
        }
    }

    pub fn init(&self) {
       self.devices.init();
    }

    pub fn get_gpio_controller(&self) -> &dyn GPIOController {
        self.devices.get_gpio_controller()
    }

    pub fn get_console(&self) -> &dyn Console {
        self.devices.get_console()
    }
}

pub struct Devices<'a> {
    gpio: RefCell<&'a mut GPIORegisters>,
    timer: RefCell<&'a mut TimerRegisters>,
    mini_uart: RefCell<&'a mut MiniUARTRegisters>,
    mailbox: RefCell<&'a mut MailboxRegisters>,
    emmc_controller: BoxedDevice<EMMCController<'a>>
}

impl<'a> Devices<'a> {
    fn box_device<T>(device: T) -> Rc<RefCell<T>> {
        Rc::new(RefCell::new(device))
    }

    pub const fn uninitialized() -> Self {
        Self {
            timer: RefCell::new(mmio::get_timer_registers()),
            mini_uart: RefCell::new(mmio::get_miniuart_registers()),
            gpio: RefCell::new(mmio::get_gpio_registers()),
            mailbox: RefCell::new(mmio::get_mailbox_registers()),
            emmc_controller: None,
        }
    }

    pub fn init(&self) {
        self.mini_uart.borrow_mut().init(self.get_gpio_controller())
    }

    pub fn get_gpio_controller(&self) -> &dyn GPIOController {
        self
    }

    pub fn get_console(&self) -> &dyn Console {
        self
    }
}

impl GPIOController for Devices<'_> {
    fn set_pin_mode(&self, pin: super::gpio::Pin, mode: super::gpio::Mode) {
        self.gpio.borrow_mut().set_pin_mode(pin, mode);
    }

    fn set_pin_output(&self, pin: super::gpio::Pin, output: super::gpio::OutputLevel) {
        self.gpio.borrow_mut().set_out(pin, output);
    }

    fn set_pin_pull(&self, pin: super::gpio::Pin, pull_mode: super::gpio::Pull) {
        self.gpio.borrow_mut().pull(pin, pull_mode);
    }
}

impl Console for Devices<'_> {
    fn newline(&self) {
        self.mini_uart.borrow_mut().newline();
    }

    fn writef(&self, args: Arguments) {
        self.mini_uart.borrow_mut().writef(args);
    }

    fn writefln(&self, args: Arguments) {
        self.mini_uart.borrow_mut().writefln(args);
    }
}

impl MailboxController for Devices<'_> {
    fn send_message_on_channel(&self, buffer: &MailboxBuffer, channel: super::mailbox::Channel) -> u32 {
        self.mailbox.borrow_mut().send_message(buffer as *const MailboxBuffer as u32, channel)
    }
}