use crate::{
    aarch64::syscall::SyscallArgs,
    allocator::page_allocator::{Page, PageAllocator, PageRef},
    device::sector_device::{Sector, SectorDevice},
    platform::{
        self,
        emmc::{self, EMMCConfiguration, EMMCController, EMMCRegisters},
        gpio::{GPIOController, GPIORegisters, StatusLight},
        hardware_config::HardwareConfig,
        interrupt::InterruptRegisters,
        kernel::Kernel,
        mailbox::{MailboxBuffer, MailboxController, MailboxRegisters},
        timer::TimerRegisters,
    },
};

use super::{mini_uart::MiniUARTRegisters, mmio};

use alloc::boxed::Box;
use alloc::rc::Rc;
use core::{
    cell::{Cell, Ref, RefCell, RefMut, UnsafeCell},
    fmt::Arguments,
    mem::MaybeUninit,
};

use crate::device::{console::Console, timer::Timer};

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

pub type BoxedDevice<T> = Option<Rc<RefCell<T>>>;

pub static PLATFORM: Platform = Platform::uninitialized();

pub fn get_platform() -> &'static Platform<'static> {
    &PLATFORM
}

unsafe extern "C" {
    unsafe static PAGE_SECTION_START: usize;
    unsafe static PAGE_SECTION_SIZE: usize;
}

pub struct Platform<'a> {
    devices: Devices<'a>,
    interrupt_handlers: InterruptHandler,
    kernel: RefCell<Option<Kernel<'a>>>,
}

impl<'a> Platform<'a> {
    const fn uninitialized() -> Self {
        Self {
            devices: Devices::uninitialized(),
            interrupt_handlers: InterruptHandler::new(),
            kernel: RefCell::new(None),
        }
    }

    pub fn init(&'a self) {
        self.devices.init();
    }

    pub fn get_gpio_controller(&self) -> &dyn GPIOController {
        self.devices.get_gpio_controller()
    }

    pub fn get_console(&self) -> &dyn Console {
        self.devices.get_console()
    }

    pub fn get_timer(&self) -> &dyn Timer {
        self.devices.get_timer()
    }

    pub fn get_emmc_controller(&'a self) -> &'a dyn SectorDevice<'a> {
        self.devices.get_emmc_controller()
    }

    pub fn get_mailbox_controller(&self) -> &dyn MailboxController {
        self.devices.get_mailbox_controller()
    }

    pub fn get_hardware_config(&self) -> HardwareConfig {
        HardwareConfig::from_mailbox(self.get_mailbox_controller())
    }

    pub fn handle_interrupt(&self) {
        crate::println!("Handling Interrupt");
        let interrupt_type = self.devices.interrupts.borrow().get_interrupt_type();
    }

    pub fn handle_syscall(&self, syscall_number: usize, args: SyscallArgs) {
        if let Some(ref mut kernel) = *self.kernel.borrow_mut() {
            kernel.handle_syscall(syscall_number, args);
        }
    }

    pub fn register_kernel(&self, kernel: Kernel<'a>) {
        self.kernel.replace(Some(kernel));
    }
}

pub struct Devices<'a> {
    gpio: RefCell<&'a mut GPIORegisters>,
    timer: RefCell<&'a mut TimerRegisters>,
    mini_uart: RefCell<&'a mut MiniUARTRegisters>,
    mailbox: RefCell<&'a mut MailboxRegisters>,
    emmc: RefCell<&'a mut EMMCRegisters>,
    emmc_configuration: RefCell<EMMCConfiguration>,
    interrupts: RefCell<&'a mut InterruptRegisters>,
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
            emmc: RefCell::new(mmio::get_emmc_registers()),
            emmc_configuration: RefCell::new(EMMCConfiguration::new()),
            interrupts: RefCell::new(mmio::get_interrupt_registers()),
        }
    }

    pub fn init(&'a self) {
        self.mini_uart.borrow_mut().init(self.get_gpio_controller());

        let emmc_configuration =
            EMMCController::initialize(&self.emmc, self.get_timer(), self.get_gpio_controller());

        self.emmc_configuration.replace(emmc_configuration);
    }

    pub fn get_gpio_controller(&self) -> &dyn GPIOController {
        self
    }

    pub fn get_console(&self) -> &dyn Console {
        self
    }

    pub fn get_timer(&self) -> &dyn Timer {
        self
    }

    pub fn get_mailbox_controller(&self) -> &dyn MailboxController {
        self
    }

    pub fn get_emmc_controller(&'a self) -> &'a dyn SectorDevice<'a> {
        self
    }
}

unsafe impl<'a> Sync for Platform<'a> {}

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

    fn set_pin_high_detect_enable(&self, pin: super::gpio::Pin) {
        self.gpio.borrow_mut().set_high_detect_enable(pin, 1);
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
    fn send_message_on_channel(
        &self,
        buffer: &MailboxBuffer,
        channel: super::mailbox::Channel,
    ) -> u32 {
        self.mailbox
            .borrow_mut()
            .send_message(buffer.as_ptr() as u32, channel)
    }
}

impl Timer for Devices<'_> {
    fn delay_micros(&self, micros: u64) {
        self.timer.borrow_mut().delay_microseconds(micros);
    }

    fn get_micros(&self) -> u64 {
        self.timer.borrow().time()
    }

    fn set_timeout(&self, micros: u32) {
        self.timer.borrow_mut().set_timeout(micros);
    }
}

impl<'a> SectorDevice<'a> for Devices<'a> {
    fn read_sector(&'a self, address: crate::device::sector_device::SectorAddress) -> Sector {
        let mut emmc_controller = EMMCController::with_configuration(
            &self.emmc,
            self.get_gpio_controller(),
            self.get_timer(),
            self.emmc_configuration.borrow().clone(),
        );

        let mut buffer: [u8; 512] = [0; 512];

        emmc_controller.read_blocks(address, &mut buffer, 1);

        Sector::from(buffer)
    }
}

pub struct InterruptHandler {}

impl InterruptHandler {
    pub const fn new() -> Self {
        Self {}
    }
}
