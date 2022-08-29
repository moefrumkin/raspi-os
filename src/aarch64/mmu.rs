use core::arch::asm;

pub unsafe fn init(table_start: *mut usize) {
        let table = core::slice::from_raw_parts_mut(table_start, 512);

        let tcr_el1 = 
            (0b10 << 30) | //4kb Granule
            34; //Amount to shrink virtual address space

        let sctlr_el1 = 1; //Enable MMU

        let mair = 0;

        write!("ttbr0_el1", table_start);

        write!("mair_el1", mair);

        write!("tcr_el1", tcr_el1);

        asm!("isb");

        for i in 0..512 {
            table[i] = 
                (i << 21) | // Block Pointer
                (1 << 10) | // Access Bit
                1; // Valid Entry
        }
        
        asm!("dsb sy");

        write!("sctlr_el1", sctlr_el1);

        asm!("isb");
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