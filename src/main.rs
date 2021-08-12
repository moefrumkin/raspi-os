#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(global_asm)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(lang_items)]
#![feature(panic_info_message)]

mod aarch64;
#[cfg(not(test))]
mod panic;
mod platform;

#[cfg(not(test))]
#[lang = "eh_personality"]
pub extern fn eh_personality() {}