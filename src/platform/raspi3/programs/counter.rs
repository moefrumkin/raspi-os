use crate::aarch64::interrupt::IRQLock;
use crate::aarch64::{cpu, syscall};
use crate::platform::semaphore::SemMutex;
use crate::println;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

type Mutex<T> = SemMutex<T>;

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

pub struct CounterThreadArguments {
    counter: Arc<SemMutex<Counter>>,
    iterations: usize,
    thread_number: usize,
}

pub extern "C" fn run_count(n: usize) {
    let mut threads = Vec::with_capacity(n);
    let counter = Arc::new(Mutex::new(Counter::new()));

    for i in 0..n {
        println!("Starting counter {}", i);

        let args = Box::new(CounterThreadArguments {
            counter: counter.clone(),
            iterations: 200_000,
            thread_number: i,
        });

        let id = syscall::create_thread(
            counter_thread,
            String::from(alloc::format!("Counter {}", i)),
            Box::into_raw(args) as usize,
        );

        threads.push(id);
    }

    for i in 0..n {
        let ret = syscall::join(threads[i] as u64);

        println!("Counter thread {} exited with code {}", i, ret);
    }

    println!("Final count: {}", counter.lock().count());

    syscall::exit(0);
}

pub extern "C" fn counter_thread(counter: Box<CounterThreadArguments>) {
    let args = counter;
    let counter = args.counter;
    for _ in 0..args.iterations {
        counter.lock().increment();

        //cpu::sleep(200_000);
    }

    syscall::exit(0);
}
