use crate::{
    aarch64::{
        self,
        mmu::{Address, TableDescriptor, TableEntry},
    },
    allocator::page_allocator::PAGE_SIZE,
    platform::platform_devices::PLATFORM,
};

pub type Table = [usize; 512];

#[derive(Debug, Copy, Clone)]
pub struct PageTable {
    pgd: *mut Table,
}

impl PageTable {
    const TABLE_LENGTH: usize = 512;

    pub fn new_unmapped() -> Self {
        let page = PLATFORM.allocate_page();
        let page_ptr = page.page as usize as *mut [usize; Self::TABLE_LENGTH];

        for i in 0..Self::TABLE_LENGTH {
            unsafe {
                (*page_ptr)[i] = 0;
            }
        }

        Self { pgd: page_ptr }
    }

    pub fn get_ttbr(&self) -> usize {
        (self.pgd) as usize
    }

    pub fn kernel_mapped() -> Self {
        let pgd_addr = PLATFORM.allocate_zeroed_page().page;
        let pgd = pgd_addr as *mut Table;

        //let pud = PLATFORM.allocate_zeroed_page();

        Self { pgd }
    }

    pub fn from(ttbr: usize) -> Self {
        Self {
            pgd: ttbr as *mut Table,
        }
    }

    // TODO: how to handle errors/preconditions?
    pub fn map_user_address(&mut self, virtual_address: u64, physical_address: u64) {
        // Assumes 48 bit address space with 4k page.
        let vaddr = Address::new(virtual_address);
        let paddr = Address::new(physical_address);

        assert!(vaddr.get_offset() == 0, "Vaddr offset is not 0");
        assert!(paddr.get_offset() == 0, "Paddr offset is not 0");

        let pgd_index = vaddr.get_pgd() as usize;
        let pud_index = vaddr.get_pud() as usize;
        let pld_index = vaddr.get_pld() as usize;
        let pte_index = vaddr.get_pte() as usize;

        let pgd_entry = TableDescriptor::new(unsafe { (*self.pgd)[pgd_index] as u64 });

        let pud;

        if pgd_entry.is_valid() {
            pud = unsafe { pgd_entry.get_next_table_address() as *mut Table }
        } else {
            let page = PLATFORM.allocate_zeroed_page();

            let pud_addr = page.page as usize;
            pud = page.page as *mut Table;

            let descriptor = TableDescriptor::new(pud_addr as u64);

            unsafe {
                (*self.pgd)[pgd_index] = descriptor.get_value() as usize;
            }
        }

        let pud_entry = TableDescriptor::new(unsafe { (*pud)[pud_index] } as u64);

        let pld;

        if pud_entry.is_valid() {
            pld = pgd_entry.get_next_table_address() as *mut Table;
        } else {
            let page = PLATFORM.allocate_zeroed_page();

            let pld_addr = page.page as usize;
            pld = pld_addr as *mut Table;

            let descriptor = TableDescriptor::new(pld_addr as u64);

            unsafe { (*pud)[pud_index] = descriptor.get_value() as usize };
        }

        let pld_entry = TableDescriptor::new(unsafe { (*pld)[pld_index] } as u64);

        let pte;
        if pld_entry.is_valid() {
            pte = pld_entry.get_next_table_address() as *mut Table;
        } else {
            let page = PLATFORM.allocate_zeroed_page();

            let pte_addr = page.page as usize;
            pte = pte_addr as *mut Table;

            let descriptor = TableDescriptor::new(pte_addr as u64);

            unsafe { (*pld)[pld_index] = descriptor.get_value() as usize };
        }

        //let pte_entry = TableEntry::from(unsafe { (*pte)[pte_index] });

        // TODO: should we overwrite previous mappings?
        let pte_entry = TableEntry::from(paddr.get_pte_entry())
            .set_id(0b11)
            .set_access_flag(1);

        unsafe {
            (*pte)[pte_index] = pte_entry.get_value() as usize;
        }
    }
}
