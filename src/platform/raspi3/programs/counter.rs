use crate::aarch64::cpu;
use crate::println;
use alloc::string::String;
use alloc::vec::Vec;

pub extern "C" fn run_count(n: usize) {
    let mut threads = Vec::with_capacity(n);

    for i in 0..n {
        println!("Starting counter {}", i);

        let id = cpu::create_thread(
            counter_thread,
            String::from(alloc::format!("Counter {}", i)),
            i,
        );

        threads.push(id);
    }

    for i in 0..n {
        let ret = cpu::join_thread(threads[i] as u64);

        println!("Counter thread {} exited with code {}", i, ret);
    }

    cpu::exit_thread(0);
}

pub extern "C" fn counter_thread(number: usize) {
    let mut count = 1;
    let mut oops = alloc::vec![];
    println!("Starting thread: {}", number);
    for i in 0..10 {
        println!("Counter {}: {}", number, count);
        count += 1;
        oops.push(i);

        cpu::sleep(200_000);
    }

    println!("Goodbye from Counter {}", number);

    cpu::exit_thread(0);
}
