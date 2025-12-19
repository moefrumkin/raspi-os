use super::registers::{
    SystemControlRegister, TranslationControlRegister, UserTranslationTableBaseRegister,
};
use crate::aarch64::cpu;
use crate::bitfield;
use core::arch::asm;

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
        pub fn set_address(mut self, _address: usize) -> Self {
            self.value &= !Self::ADDRESS_MASK;
            self
        }

        pub fn clone(&self) -> Self {
            Self {
                value: self.value
            }
        }
    }
}
