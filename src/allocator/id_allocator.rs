pub struct IDAllocator {
    current_id: u64,
}

impl IDAllocator {
    pub fn new() -> Self {
        Self { current_id: 1 }
    }

    pub fn allocate_id(&mut self) -> u64 {
        let id = self.current_id;

        self.current_id += 1;

        id
    }
}
