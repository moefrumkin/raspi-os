#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(global_asm)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(lang_items)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[cfg(not(test))]
use sync::SpinMutex;
#[cfg(not(test))]
use allocator::ll_alloc::LinkedListAllocator;

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: SpinMutex<LinkedListAllocator> =  SpinMutex::new(LinkedListAllocator::new());

#[cfg(not(test))]
mod aarch64;
#[cfg(not(test))]
mod panic;
mod platform;
mod allocator;
mod sync;
mod canvas;
mod utils;

#[cfg(not(test))]
#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}
