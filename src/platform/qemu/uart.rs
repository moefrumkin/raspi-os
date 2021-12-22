pub struct UARTController {
    ptr: *mut u8
}

impl UARTController {
    pub fn new(ptr: usize) -> Self {
        UARTController {
            ptr: ptr as *mut u8
        }
    }

    pub fn writeln(&self, s: &str) {
        self.write(s);
        self.putc(0x0a as char);
    }

    pub fn write(&self, s: &str) {
        for b in s.chars() {
            self.putc(b);
        }
    }

    pub fn putc(&self, b: char) {
        unsafe {
            self.ptr.write_volatile(b as u8);
        }
    }
}