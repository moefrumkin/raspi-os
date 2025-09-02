#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![allow(internal_features)]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
//TODO: temporary, just to make development easier
#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(ptr_as_ref_unchecked)]

extern crate alloc;

#[cfg(not(test))]
use allocator::ll_alloc::LinkedListAllocator;
#[cfg(not(test))]
use sync::SpinMutex;

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: SpinMutex<LinkedListAllocator> = SpinMutex::new(LinkedListAllocator::new());

mod aarch64;
mod allocator;
mod canvas;
#[cfg(not(test))]
mod panic;
mod platform;
mod sync;
mod utils;

mod volatile;

mod device;
mod filesystem;

#[cfg(not(test))]
#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}
