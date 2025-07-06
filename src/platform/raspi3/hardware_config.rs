use super::{
    mailbox::MailboxController,
    mailbox_property::{
        MessageBuilder, 
        MailboxInstruction, 
        MailboxBufferSlice
    },
};

use core::fmt;
 


pub struct HardwareConfig {
    firmware_revision: u32,
    board_model: u32,
    board_revision: u32,
    MAC_address: MACAddress,
    board_serial: u64
}

impl fmt::Display for HardwareConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n\
            \t-firmware revision: {} \n\
            \t-board model: {} \n\
            \t-board revision: {}\n\
            \t-MAC address: {}\n\
            \t-board serial: {}",
            self.firmware_revision,
            self.board_model,
            self.board_revision,
            self.MAC_address,
            self.board_serial
        )
    }
}

#[derive(Copy, Clone)]
pub struct MACAddress {
    pub bytes: [u8; 6]
}

impl fmt::Display for MACAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}", self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3], self.bytes[4], self.bytes[5])
    }
}


impl HardwareConfig {
    pub fn from_mailbox(mailbox: &mut MailboxController) -> Self {
        let mut firmware_revision = GetFirmwareRevision::new();
        let mut board_model = GetBoardModel::new();
        let mut board_revision = GetBoardRevision::new();
        let mut mac_address = GetBoardMAC::new();
        let mut board_serial = GetBoardSerial::new();


        let mut config_request = MessageBuilder::new()
            .request(&mut firmware_revision)
            .request(&mut board_model)
            .request(&mut board_revision)
            .request(&mut mac_address)
            .request(&mut board_serial);

        config_request.send(mailbox);

        Self {
            firmware_revision: firmware_revision.get_response(),
            board_model: board_revision.get_response(),
            board_revision: board_revision.get_response(),
            MAC_address: mac_address.get_address(),
            board_serial: board_serial.get_response()
        }
    }
}

struct GetFirmwareRevision {
    revision: u32
}


impl GetFirmwareRevision {
    fn new() -> Self {
        Self {
            revision: 0
        }
    }

    fn get_response(&self) -> u32 {
        self.revision
    }
}

impl MailboxInstruction for GetFirmwareRevision {
    fn get_encoding(&self) -> u32 {
        0x1
    }

    fn get_buffer_words(&self) -> u32 {
        1
    }

    fn write_data_at_offset(&self, buffer: &mut MailboxBufferSlice) {
        buffer[0].set(0);
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.revision = buffer[0].get();
    }
}

pub struct GetBoardModel {
    model: u32
}


impl GetBoardModel {
    fn new() -> Self {
        Self {
            model: 0
        }
    }

    fn get_response(&self) -> u32 {
        self.model
    }
}

impl MailboxInstruction for GetBoardModel {
    fn get_encoding(&self) -> u32 {
        0x10001
    }

    fn get_buffer_words(&self) -> u32 {
        1
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.model = buffer[0].get();
    }
}


pub struct GetBoardRevision {
    revision: u32
}


impl GetBoardRevision {
    fn new() -> Self {
        Self {
            revision: 0
        }
    }

    fn get_response(&self) -> u32 {
        self.revision
    }
}

impl MailboxInstruction for GetBoardRevision {
    fn get_encoding(&self) -> u32 {
        0x10002
    }

    fn get_buffer_words(&self) -> u32 {
        1
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        self.revision = buffer[0].get();
    }
}

pub struct GetBoardMAC {
    address: MACAddress
}

impl GetBoardMAC {
    fn new() -> Self {
        Self {
            address: MACAddress { bytes: [0; 6] }
        }
    }

    fn get_address(&self) -> MACAddress {
        self.address
    }
}

impl MailboxInstruction for GetBoardMAC {
    fn get_encoding(&self) -> u32 { 0x10003 }
    fn get_buffer_words(&self) -> u32 { 2}

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        let first_word = buffer[0].get();
        let second_word = buffer[1].get();

        // TODO: abstract
        self.address.bytes[0] = (first_word >> 24) as u8;
        self.address.bytes[1] = ((first_word >> 16) & 0xFF) as u8;
        self.address.bytes[2] = ((first_word >> 8) & 0xFF) as u8;
        self.address.bytes[3] = (first_word & 0xFF) as u8;
        
        self.address.bytes[4] = (second_word >> 24) as u8;
        self.address.bytes[5] = ((second_word >> 16) & 0xFF) as u8;
    }
}

pub struct GetBoardSerial {
    serial: u64
}


impl GetBoardSerial {
    fn new() -> Self {
        Self {
            serial: 0
        }
    }

    fn get_response(&self) -> u64 {
        self.serial
    }
}

impl MailboxInstruction for GetBoardSerial {
    fn get_encoding(&self) -> u32 {
        0x10004
    }

    fn get_buffer_words(&self) -> u32 {
        2
    }

    fn read_data_at_offset(&mut self, buffer: &MailboxBufferSlice) {
        // TODO: check endianness
        let first_half = buffer[0].get() as u64;
        let second_half = buffer[1].get() as u64;
        self.serial = (first_half << 32) | second_half;
    }
}


