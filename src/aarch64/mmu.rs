use super::registers::{
    SystemControlRegister, TranslationControlRegister, UserTranslationTableBaseRegister,
};
use crate::aarch64::cpu;
use crate::aarch64::registers::KernelTranslationTableBaseRegister;
use crate::bitfield;
use core::arch::asm;

pub fn get_user_table() -> usize {
    UserTranslationTableBaseRegister::read_to_buffer().value()
}

pub fn get_kernel_table() -> usize {
    KernelTranslationTableBaseRegister::read_to_buffer().value()
}

pub unsafe fn init(table_start: *mut usize) {
    let table = core::slice::from_raw_parts_mut(table_start, 512);

    let mair = 0;

    UserTranslationTableBaseRegister::read_to_buffer()
        .set_table_pointer(table_start as usize)
        .write_to_register();

    write!("mair_el1", mair);

    TranslationControlRegister::read_to_buffer()
        .set_granule_size(TranslationControlRegister::GranuleSize::Kb4 as usize)
        .set_table_offset(33)
        .write_to_register();

    cpu::instruction_buffer();

    let attributes = MemoryAttributes::new()
        .set_access_flag(1)
        .set_entry_type(MemoryAttributes::BLOCK_ENTRY);

    for i in 0..512 {
        table[i] = attributes.clone().set_address(i << 21).value;
    }

    cpu::data_buffer();

    SystemControlRegister::read_to_buffer()
        .set_translation_state(SystemControlRegister::TranslationState::Enabled as usize)
        .set_cache_enable(1)
        .write_to_register();

    cpu::instruction_buffer();
}

pub unsafe fn init_tested(table_start: *mut usize) -> Result<(), ()> {
    for i in (0..0x60_000).step_by(0x1000) {
        core::ptr::write_volatile(i as *mut usize, i);
    }

    for i in (0x256_000..0x20000000).step_by(0x8000) {
        core::ptr::write_volatile(i as *mut usize, i);
    }

    init(table_start);

    for i in (0..0x60_000).step_by(0x1000) {
        if core::ptr::read_volatile(i as *const usize) != i {
            return Err(());
        }
    }

    for i in (0x256_000..0x20000000).step_by(0x8000) {
        if core::ptr::read_volatile(i as *const usize) != i {
            return Err(());
        }
    }

    Ok(())
}

struct TranslationTable {
    table: [usize; Self::TABLE_LENGTH],
}

impl TranslationTable {
    const TABLE_LENGTH: usize = 512;
}

bitfield! {
    MemoryAttributes(usize) {
        entry_type: 0-1,
        attribute_index: 2-4,
        security_bit: 5-5,
        access_permission: 6-7,
        shareability: 8-9,
        access_flag: 10-10,

        address: 12-52,

        privileged_execution: 53-53,
        unprivileged_execution: 54-54,
        software_values: 55-58
    } with {
        const ADDRESS_MASK: usize = ((1 << (52 - 11 + 1)) - 1) << 11;

        const BLOCK_ENTRY: usize = 0b001;
        const TABLE_ENTRY: usize = 0b011;

        pub const fn new() -> Self {
            Self { value: 0 }
        }

        // TODO: shouldn't address be used?
        /*pub fn set_address(mut self, _address: usize) -> Self {
            self.value &= !Self::ADDRESS_MASK;
            self
        }*/

        pub fn clone(&self) -> Self {
            Self {
                value: self.value
            }
        }
    }
}

bitfield! {
    Address(u64) {
        offset: 0-11,
        pte: 12-20,
        pld: 21-29,
        pud: 30-38,
        pgd: 39-47
    } with {
        pub fn new (value: u64) -> Self {
            Self {value}
        }

        pub fn get_pte_entry(self) -> u64 {
            self.value & 0x0000_FFFF_FFFF_F000
        }
    }
}

bitfield! {
    TableDescriptor(u64) {
        valid: 0-0,
        identifier: 0-1,
        address: 12-51,
        attributes: 52-63 // TODO: check bits
    } with {
        pub fn new(value: u64) -> Self {
            Self {value}
        }

        pub fn is_valid(self) -> bool {
            self.get_valid() == 1
        }

        pub fn get_value(self) -> u64 {
            self.value
        }

        pub fn get_next_table_address(self) -> u64 {
            self.get_address() << 12
        }
    }
}

bitfield! {
    TableEntry(u64) {
        id: 0-1,
        attribute_index: 2-4,
        access_permission: 6-7,
        access_flag: 10-10,
        address: 12-47
    } with {
        pub fn from(value: u64) -> Self {
            Self {
                value
            }
        }

        pub fn get_value(self) -> u64 {
            self.value
        }
    }
}
