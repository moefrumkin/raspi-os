//! The panic handler that is used in the case of a runtime exception
//!
//! The standard library has the default options of stack unwinding or aborting, however neither of those can be used, as the full standard library is not included

#[cfg(feature = "raspi3")]
use super::platform::{
    gpio::{GPIOController, OutputLevel, StatusLight},
    //platform_devices::PLATFORM
};

#[cfg(feature = "raspi3")]
use crate::println;

use core::{panic::PanicInfo, alloc::Layout};
use crate::ALLOCATOR;

///The global panic handler
#[cfg(feature = "raspi3")]
#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    /*let status_light = PLATFORM.get_status_light().unwrap();
    let status_light = status_light.borrow_mut();

    status_light.set_green(OutputLevel::Low);
    status_light.set_blue(OutputLevel::Low);

    status_light.set_red(OutputLevel::High);*/

    println!("");
    println!("A Fatal Kernel Panic Occured");

    // TODO:?
    println!("{}", info);
    
    // TODO: tidy up
    if let Some(args) = info.message().as_str() {
        println!("{}", args);
    } else {
        println!("No message supplied");
    }

    if let Some(location) = info.location() {
        println!("@{}:{}", location.file(), location.line());
    } else {
        println!("No location found");
    }

    loop {}
}

#[cfg(feature = "raspi3")]
#[alloc_error_handler]
fn on_alloc_error(layout: Layout) -> ! {
    /*let status_light = PLATFORM.get_status_light().unwrap();
    let status_light = status_light.borrow_mut();

    status_light.set_green(OutputLevel::Low);
    status_light.set_blue(OutputLevel::Low);
    status_light.set_red(OutputLevel::High);*/

    println!("A Fatal Allocation Error Occured");
    println!("Unable to allocate: {:?} using allocator: {:?}", layout, ALLOCATOR);

    let stats = ALLOCATOR.stats();

    println!("{} allocations, {} frees", stats.allocs, stats.frees);
    println!("{} bytes in {} blocks", stats.free_space, stats.blocks);

    loop {}
}

#[cfg(not(feature = "raspi3"))]
#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[cfg(not(feature = "raspi3"))]
#[alloc_error_handler]
fn on_alloc_error(_layout: Layout) -> ! {
    panic!();
}
