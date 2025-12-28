use core::fmt::Arguments;

pub trait Console {
    fn newline(&self);
    fn writef(&self, args: Arguments);
    fn writefln(&self, args: Arguments);
}