use super::{
    mailbox::MailboxController,
    mailbox_property::{
        FromMailboxBuffer, MailboxBufferSlice, MessageBuilder, SimpleRequest, ToMailboxBuffer,
    },
};
use crate::bitfield;
use core::fmt;

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum Device {
    SDCard = 0x0,
    UART0 = 0x1,
    UART1 = 0x2,
    USBHCD = 0x3,
    I2C0 = 0x4,
    I2C1 = 0x5,
    I2C2 = 0x6,
    SPI = 0x7,
    CCP2TX = 0x8,
}

pub const DEVICES: [Device; 9] = [
    Device::SDCard,
    Device::UART0,
    Device::UART1,
    Device::USBHCD,
    Device::I2C0,
    Device::I2C1,
    Device::I2C2,
    Device::SPI,
    Device::CCP2TX,
];

impl Device {
    const GET_POWER_STATE: u32 = 0x20001;
    const GET_TIMING: u32 = 0x20002;

    pub fn to_u32(self) -> u32 {
        self as u32
    }

    pub fn from_u32(val: u32) -> Self {
        match val {
            0x0 => Device::SDCard,
            0x1 => Device::UART0,
            0x2 => Device::UART1,
            0x3 => Device::USBHCD,
            0x4 => Device::I2C0,
            0x5 => Device::I2C1,
            0x6 => Device::I2C2,
            0x7 => Device::SPI,
            0x8 => Device::CCP2TX,
            _ => panic!("Unknown device id"),
        }
    }

    pub fn get_power_state(self, mailbox: &dyn MailboxController) -> PowerState {
        let mut request =
            SimpleRequest::<Device, PowerStateResponse, { Self::GET_POWER_STATE }>::with_request(
                self,
            );

        let mut message = MessageBuilder::new().request(&mut request);

        message.send(mailbox);

        return request.get_response().1;
    }

    pub fn get_timing(self, mailbox: &dyn MailboxController) -> u32 {
        let mut request =
            SimpleRequest::<Device, TimingResponse, { Self::GET_TIMING }>::with_request(self);

        let mut message = MessageBuilder::new().request(&mut request);

        message.send(mailbox);

        return request.get_response().1;
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Device::SDCard => "SD Card Reader",
                Device::UART0 => "UART Controller 0",
                Device::UART1 => "UART Controller 1",
                Device::USBHCD => "USB HCD",
                Device::I2C0 => "I2C Controller 0",
                Device::I2C1 => "I2C Controller 1",
                Device::I2C2 => "I2C Controller 2",
                Device::SPI => "SPI Controller",
                Device::CCP2TX => "CCP2TX",
            }
        )
    }
}

impl ToMailboxBuffer for Device {
    fn write_to_mailbox_buffer(&self, buffer: &mut MailboxBufferSlice) {
        buffer[0].set(self.to_u32());
    }
}

bitfield! {
    PowerState(u32) {
        on: 0-0,
        exists: 1-1
    } with {
        pub fn is_on(&self) -> bool {
            self.get_on() == 1
        }

        pub fn exists(&self) -> bool {
            self.get_exists() == 1
        }

        pub fn from_u32(value: u32) -> Self {
            Self {
                value
            }
        }
    }
}

#[derive(Copy, Clone)]
struct PowerStateResponse(Device, PowerState);

impl FromMailboxBuffer for PowerStateResponse {
    fn read_from_mailbox_buffer(buffer: &MailboxBufferSlice) -> Self {
        Self(
            Device::from_u32(buffer[0].get()),
            PowerState::from_u32(buffer[1].get()),
        )
    }
}

#[derive(Copy, Clone)]
struct TimingResponse(Device, u32);

impl FromMailboxBuffer for TimingResponse {
    fn read_from_mailbox_buffer(buffer: &MailboxBufferSlice) -> Self {
        Self(Device::from_u32(buffer[0].get()), buffer[1].get())
    }
}
