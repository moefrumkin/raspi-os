use crate::platform::{self, emmc::{self, EMMCController, EMMCRegisters}, gpio::{GPIOController, StatusLight},
mailbox::MailboxController};

use super::{
    mini_uart::MiniUARTController,
    mmio,
    timer::Timer
};

use core::{cell::{Cell, Ref, RefCell, RefMut, UnsafeCell}, mem::MaybeUninit};
use alloc::rc::Rc;
use alloc::boxed::Box;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let console = PLATFORM.get_console().expect("Console not initialized");
        console.borrow_mut().writef(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
       let console = PLATFORM.get_console().expect("Console not initialized");
       console.newline()
    };
    ($($args:tt)*) => {
        let console = crate::platform::raspi3::hardware_devices::PLATFORM.get_console().expect("Console not initialized");
        console.borrow_mut().writefln(format_args!($($args)*))
    }
}

pub const PLATFORM: Platform = Platform::uninitialized();

pub type BoxedDevice<T> = Option<Rc<RefCell<T>>>;

pub struct Platform<'a> {
    devices: RefCell<Devices<'a>>
}

impl<'a> Platform<'a> {
    const fn uninitialized() -> Self {
        Self {
            devices: RefCell::new(Devices::uninitialized())
        }
    }

    pub fn init(&self) {
       self.devices.borrow_mut().init();
    }

    pub fn get_console(&self) -> BoxedDevice<MiniUARTController<'a>> {
        self.devices.borrow().get_console()
    }

    pub fn get_status_light(&self) -> BoxedDevice<StatusLight> {
        self.devices.borrow().get_status_light()
    }

    pub fn get_timer(&self) -> BoxedDevice<Timer<'a>> {
        self.devices.borrow().get_timer()
    }

    pub fn get_mailbox_controller(&self) -> BoxedDevice<MailboxController> {
        self.devices.borrow().get_mailbox_controller()
    }

    pub fn get_emmc_controller(&self) -> BoxedDevice<EMMCController<'a>> {
        self.devices.borrow().get_emmc_controller()
    }
}

pub struct Devices<'a> {
    timer: BoxedDevice<Timer<'a>>,
    console: BoxedDevice<MiniUARTController<'a>>,
    gpio: BoxedDevice<GPIOController>,
    status_light: BoxedDevice<StatusLight>,
    mailbox_controller: BoxedDevice<MailboxController>,
    emmc_controller: BoxedDevice<EMMCController<'a>>
}

impl<'a> Devices<'a> {
    fn box_device<T>(device: T) -> Rc<RefCell<T>> {
        Rc::new(RefCell::new(device))
    }

    pub const fn uninitialized() -> Self {
        Self {
            timer: None,
            console: None,
            gpio: None,
            status_light: None,
            mailbox_controller: None,
            emmc_controller: None
        }
    }

    pub fn init(&mut self) {
        let gpio = Self::box_device(GPIOController::new(mmio_controller.clone()));
        let console = Self::box_device(MiniUARTController::new(gpio.clone()));
        //let status_light = Self::box_device(StatusLight::new(gpio.clone()));
        let timer = Self::box_device(Timer::new());
        let mailbox_controller = Self::box_device(MailboxController::new(mmio_controller.clone()));
        let emmc_controller = Self::box_device(
            EMMCController::new(
                mmio::get_emmc_registers(),
                gpio.clone(),
                timer.clone()
            )
        );

        //status_light.borrow_mut().init();
        //console.borrow_mut().init();
        emmc_controller.borrow_mut().initialize();

        self.gpio = Some(gpio);
        self.console = Some(console);
        //self.status_light = Some(status_light);
        self.timer = Some(timer);
        self.mailbox_controller = Some(mailbox_controller);
        self.emmc_controller = Some(emmc_controller);
    }

    pub fn get_console(&self) -> BoxedDevice<MiniUARTController<'a>> {
        self.console.clone()
    }

    pub fn get_status_light(&self) -> BoxedDevice<StatusLight> {
        self.status_light.clone()
    }

    pub fn get_timer(&self) -> BoxedDevice<Timer<'a>> {
        self.timer.clone()
    }

    pub fn get_mailbox_controller(&self) -> BoxedDevice<MailboxController> {
        self.mailbox_controller.clone()
    }

    pub fn get_emmc_controller(&self) -> BoxedDevice<EMMCController<'a>> {
        self.emmc_controller.clone()
    }
}
