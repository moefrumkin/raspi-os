

pub struct FrameBuffer {
   start: *mut u32,
   width: u32,
   height: u32
}

impl FrameBuffer {
    pub fn new(start: *mut u32, width: u32, height: u32) -> Self {
        Self {
            start,
            width,
            height
        }
    }

    pub fn write_idx(&mut self, idx: u32, color: u32) {
        unsafe {
            self.start.offset(idx as isize).write_volatile(color);
        }
    }

    pub fn write_pixel(&mut self, x: u32, y: u32, color: u32) {
        unsafe {
            self.start.offset((y * self.width + x) as isize).write_volatile(color);
        }
    }
}

pub struct FrameBufferController {

}
