#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(naked_functions)]
#![allow(internal_features)]
#![feature(lang_items)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[cfg(not(test))]
use allocator::ll_alloc::LinkedListAllocator;
#[cfg(not(test))]
use sync::SpinMutex;

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: SpinMutex<LinkedListAllocator> = SpinMutex::new(LinkedListAllocator::new());

#[cfg(not(test))]
mod aarch64;
mod allocator;
mod canvas;
#[cfg(not(test))]
mod panic;
mod platform;
mod sync;
mod utils;

#[cfg(not(test))]
#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}
