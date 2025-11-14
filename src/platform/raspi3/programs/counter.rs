use crate::aarch64::cpu;
use crate::aarch64::interrupt::IRQLock;
use crate::println;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

pub struct Counter {
    count: u64,
}

impl Counter {
    fn new() -> Self {
        Self { count: 0 }
    }

    fn count(&self) -> u64 {
        self.count
    }

    fn increment(&mut self) {
        self.count += 1;
    }
}

pub extern "C" fn run_count(n: usize) {
    let mut threads = Vec::with_capacity(n);
    let counter = Arc::new(IRQLock::new(Counter::new()));

    for i in 0..n {
        println!("Starting counter {}", i);

        let id = cpu::create_thread(
            counter_thread,
            String::from(alloc::format!("Counter {}", i)),
            &counter as *const Arc<_> as usize,
        );

        threads.push(id);
    }

    for i in 0..n {
        let ret = cpu::join_thread(threads[i] as u64);

        println!("Counter thread {} exited with code {}", i, ret);
    }

    println!("Final count: {}", counter.lock().count());

    cpu::exit_thread(0);
}

pub extern "C" fn counter_thread(counter: &Arc<IRQLock<Counter>>) {
    let counter = counter.clone();
    for _ in 0..100000 {
        counter.lock().increment();

        //cpu::sleep(200_000);
    }

    cpu::exit_thread(0);
}
