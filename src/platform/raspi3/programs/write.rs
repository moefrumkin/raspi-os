use crate::aarch64::cpu::{self, close_object, exit_thread, write_object};

pub extern "C" fn write(_: usize) {
    let stdio = cpu::open_object("stdio");

    let message = "Hello, World\n";
    write_object(stdio, message.as_bytes());

    close_object(stdio);

    exit_thread(0);
}
