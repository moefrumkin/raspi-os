use core::arch::asm;
use crate::bitfield;
use super::registers::{TranslationControlRegister, SystemControlRegister, TranslationTableBaseRegister};

pub unsafe fn init(table_start: *mut usize) {
        let table = core::slice::from_raw_parts_mut(table_start, 512);
        
        let mair = 0;

        TranslationTableBaseRegister::read_to_buffer()
            .set_table_pointer(table_start as usize)
            .write_to_register();

        write!("mair_el1", mair);

        TranslationControlRegister::read_to_buffer()
            .set_granule_size(TranslationControlRegister::GranuleSize::Kb4 as usize)
            .set_table_offset(33)
            .write_to_register();

        cpu::instruction_buffer();

        for i in 0..512 {
            table[i] = 
                (i << 21) | // Block Pointer
                (1 << 10) | // Access Bit
                1; // Valid Entry
        }
        
        cpu::data_buffer();

        SystemControlRegister::read_to_buffer()
            .set_translation_state(SystemControlRegister::TranslationState::Enabled as usize)
            .write_to_register();

        cpu::instruction_buffer();
}

pub unsafe fn init_tested(table_start: *mut usize) -> Result<(), ()> {
    for i in (0..0x60_000).step_by(0x1000) {
        core::ptr::write_volatile(i as *mut usize, i);
    }

    for i in (0x256_000 .. 0x20000000).step_by(0x8000) {
        core::ptr::write_volatile(i as *mut usize, i);
    }

    init(table_start);

    for i in (0..0x60_000).step_by(0x1000) {
        if core::ptr::read_volatile(i as *const usize) != i {
            return Err(());
        }
    }

    for i in (0x256_000 .. 0x20000000).step_by(0x8000) {
        if core::ptr::read_volatile(i as *const usize) != i {
            return Err(());
        }
    }

    Ok(())
}

struct TranslationTable {
    table: [usize; Self::TABLE_LENGTH]
}

impl TranslationTable {
    const TABLE_LENGTH: usize = 512;
}

bitfield! {
    BlockEntry(usize) {
        attribute_index: 2-4,
        security_bit: 5-5,
        access_permission: 6-7,
        shareability: 8-9,
        access_flag: 10-10,

        privileged_execution: 53-53,
        unprivileged_execution: 54-54,
        software_values: 55-58
    }
}