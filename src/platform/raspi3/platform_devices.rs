use crate::{
    aarch64::{interrupt::IRQLock, syscall::SyscallArgs},
    allocator::page_allocator::{PageRef, PAGE_SIZE},
    device::sector_device::{Sector, SectorDevice},
    platform::{
        emmc::{EMMCConfiguration, EMMCController, EMMCRegisters},
        gpio::{GPIOController, GPIORegisters},
        hardware_config::HardwareConfig,
        interrupt::{InterruptRegisters, InterruptType},
        kernel::{Kernel, TICK},
        mailbox::{MailboxBuffer, MailboxController, MailboxRegisters},
        raspi3::exception::InterruptFrame,
        thread::Thread,
        timer::TimerRegisters,
    },
};

use alloc::sync::Arc;

use super::{mini_uart::MiniUARTRegisters, mmio};

use alloc::rc::Rc;
use core::{cell::RefCell, fmt::Arguments};

use crate::device::{console::Console, timer::Timer};

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        crate::platform::raspi3::platform_devices::PLATFORM.get_console().writef(format_args!($($arg)*))
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
    unsafe static PAGE_SECTION_SIZE: &'static usize;
}

pub struct Platform<'a> {
    devices: Devices<'a>,
    interrupt_handlers: InterruptHandler,
    kernel: IRQLock<Option<Kernel<'a>>>,
}

impl<'a> Platform<'a> {
    const fn uninitialized() -> Self {
        Self {
            devices: Devices::uninitialized(),
            interrupt_handlers: InterruptHandler::new(),
            kernel: IRQLock::new(None),
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

    pub fn set_kernel_timeout(&self, millis: u32) {
        let mut timer_regs = self.devices.timer.lock();

        timer_regs.set_kernel_timeout(millis);
    }

    pub fn get_current_thread(&self) -> Option<Arc<Thread<'a>>> {
        self.kernel
            .lock()
            .as_ref()
            .map(|kernel| Arc::clone(&kernel.scheduler.current_thread))
    }

    pub fn handle_interrupt(&self) {
        let interrupt_type = self.devices.interrupts.borrow().get_interrupt_type();
        if let Some(InterruptType::KernelTimerInterrupt) = interrupt_type {
            if let Some(ref mut kernel) = *self.kernel.lock() {
                // TODO: are the clears necessary?
                kernel.tick();
                self.get_timer().clear_matches();

                self.set_kernel_timeout(TICK);

                kernel.return_from_exception();
            }
        }

        // TODO: cleanup
        // Note: we could also just wake thread as part of the tick?
        if let Some(InterruptType::TimerInterrupt) = interrupt_type {
            panic!("Non-kernel timer interrupt occured");
            //if let Some(ref mut kernel) = *self.kernel.lock() {}
        }
    }

    pub fn exec(&self, program: &str) {
        if let Some(ref mut kernel) = *self.kernel.lock() {
            kernel.exec(program);
        }
    }

    pub fn allocate_zeroed_page(&self) -> PageRef {
        let page_ref = self.allocate_page();

        for i in 0..PAGE_SIZE {
            unsafe {
                (*page_ref.page)[i] = 0;
            }
        }

        page_ref
    }

    pub fn allocate_page(&self) -> PageRef {
        if let Some(ref mut kernel) = *self.kernel.lock() {
            kernel.allocate_page()
        } else {
            panic!();
        }
    }

    pub fn handle_syscall(&self, syscall_number: usize, args: SyscallArgs) {
        if let Some(ref mut kernel) = *self.kernel.lock() {
            kernel.handle_syscall(syscall_number, args);
            kernel.return_from_exception();
        }
    }

    pub fn register_kernel(&self, kernel: Kernel<'a>) {
        *self.kernel.lock() = Some(kernel);
    }

    pub fn save_frame(&self, frame: &mut InterruptFrame) {
        if let Some(ref mut kernel) = *self.kernel.lock() {
            kernel.save_frame(frame);
        }
    }
}

#[derive(Debug)]
pub struct Devices<'a> {
    gpio: RefCell<&'a mut GPIORegisters>,
    timer: IRQLock<&'a mut TimerRegisters>,
    mini_uart: IRQLock<&'a mut MiniUARTRegisters>,
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
            timer: IRQLock::new(mmio::get_timer_registers()),
            mini_uart: IRQLock::new(mmio::get_miniuart_registers()),
            gpio: RefCell::new(mmio::get_gpio_registers()),
            mailbox: RefCell::new(mmio::get_mailbox_registers()),
            emmc: RefCell::new(mmio::get_emmc_registers()),
            emmc_configuration: RefCell::new(EMMCConfiguration::new()),
            interrupts: RefCell::new(mmio::get_interrupt_registers()),
        }
    }

    pub fn init(&'a self) {
        self.mini_uart.lock().init(self.get_gpio_controller());

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
        self.mini_uart.lock().newline();
    }

    fn writef(&self, args: Arguments) {
        self.mini_uart.lock().writef(args);
    }

    fn writefln(&self, args: Arguments) {
        self.mini_uart.lock().writefln(args);
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
        self.timer.lock().delay_microseconds(micros);
    }

    fn get_micros(&self) -> u64 {
        self.timer.lock().time()
    }

    fn set_timeout(&self, micros: u32) {
        self.timer.lock().set_timeout(micros);
    }

    fn clear_matches(&self) {
        self.timer.lock().clear_matches();
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
