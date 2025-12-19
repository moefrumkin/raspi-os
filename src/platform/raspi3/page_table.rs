use crate::platform::platform_devices::PLATFORM;

pub struct PageTable {
    pgd: *mut [usize; Self::TABLE_LENGTH],
}

impl PageTable {
    const TABLE_LENGTH: usize = 512;

    pub fn new() -> Self {
        let page = PLATFORM.allocate_page();
        let page_ptr = page.page as usize as *mut [usize; Self::TABLE_LENGTH];

        for i in 0..Self::TABLE_LENGTH {
            unsafe {
                (*page_ptr)[i] = 0;
            }
        }

        Self { pgd: page_ptr }
    }
}
