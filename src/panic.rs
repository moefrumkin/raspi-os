//! The panic handler that is used in the case of a runtime exception
//! 
//! The standard library has the default options of stack unwinding or aborting, however neither of those can be used, as the full standard library is not included

use core::panic::PanicInfo;

///The global panic handler
#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! {
    loop {}
}