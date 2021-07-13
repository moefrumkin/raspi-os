#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(global_asm)]
#![feature(asm)]
#![feature(naked_functions)]

mod aarch64;
mod panic;
mod platform;