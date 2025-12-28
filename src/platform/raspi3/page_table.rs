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
        let page = PLATFORM.allocate_zeroed_page();
        let page_ptr = page.page as usize as *mut [usize; Self::TABLE_LENGTH];

        Self { pgd: page_ptr }
    }

    pub fn get_ttbr(&self) -> usize {
        (self.pgd) as usize & 0xFFFF_FFFF_FFFF
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
            pud = (pgd_entry.get_next_table_address() | 0xFFFF_0000_0000_0000) as *mut Table
        } else {
            let page = PLATFORM.allocate_zeroed_page();

            let pud_addr = page.page as usize;
            pud = page.page as *mut Table;

            let descriptor =
                TableDescriptor::new(pud_addr as u64 & 0xFFFF_FFFF_FFFF).set_identifier(0b11);

            unsafe {
                (*self.pgd)[pgd_index] = descriptor.get_value() as usize;
            }
        }

        let pud_entry = TableDescriptor::new(unsafe { (*pud)[pud_index] } as u64);

        let pld;

        if pud_entry.is_valid() {
            pld = (pud_entry.get_next_table_address() | 0xFFFF_0000_0000_0000) as *mut Table;
        } else {
            let page = PLATFORM.allocate_zeroed_page();

            let pld_addr = page.page as usize;
            pld = pld_addr as *mut Table;

            let descriptor =
                TableDescriptor::new(pld_addr as u64 & 0xFFFF_FFFF_FFFF).set_identifier(0b11);

            unsafe { (*pud)[pud_index] = descriptor.get_value() as usize };
        }

        let pld_entry = TableDescriptor::new(unsafe { (*pld)[pld_index] as u64 });

        let pte;
        if pld_entry.is_valid() {
            pte = (pld_entry.get_next_table_address() | 0xFFFF_0000_0000_0000) as *mut Table;
        } else {
            let page = PLATFORM.allocate_zeroed_page();

            let pte_addr = page.page as usize;
            pte = pte_addr as *mut Table;

            let descriptor =
                TableDescriptor::new(pte_addr as u64 & 0xFFFF_FFFF_FFFF).set_identifier(0b11);

            unsafe { (*pld)[pld_index] = descriptor.get_value() as usize };
        }

        //let pte_entry = TableEntry::from(unsafe { (*pte)[pte_index] });

        // TODO: should we overwrite previous mappings?
        let entry = paddr.get_pte_entry();
        let pte_entry = TableEntry::from(entry & 0xFFFF_FFFF_FFFF)
            .set_id(0b11)
            .set_access_permission(0b01)
            .set_access_flag(1);

        unsafe {
            (*pte)[pte_index] = pte_entry.get_value() as usize;
        }
    }

    pub fn is_addr_mapped(&self, addr: u64) -> bool {
        let addr = Address::new(addr);

        let pgd_index = addr.get_pgd() as usize;
        let pud_index = addr.get_pud() as usize;
        let pld_index = addr.get_pld() as usize;
        let pte_index = addr.get_pte() as usize;

        let pgd_entry = TableDescriptor::new(unsafe { (*self.pgd)[pgd_index] as u64 });

        if !pgd_entry.is_valid() {
            return false;
        }

        let pud = (pgd_entry.get_next_table_address() | 0xFFFF_0000_0000_0000) as *mut Table;

        let pud_entry = TableDescriptor::new(unsafe { (*pud)[pud_index] } as u64);

        if !pud_entry.is_valid() {
            return false;
        }

        let pld = (pud_entry.get_next_table_address() | 0xFFFF_0000_0000_0000) as *mut Table;

        let pld_entry = TableDescriptor::new(unsafe { (*pld)[pld_index] } as u64);

        if !pld_entry.is_valid() {
            return false;
        }

        let pte = (pld_entry.get_next_table_address() | 0xFFFF_0000_0000_0000) as *mut Table;

        let pte_entry = TableDescriptor::new(unsafe { (*pte)[pte_index] } as u64);

        if !pte_entry.is_valid() {
            return false;
        } else {
            return true;
        }
    }
}
