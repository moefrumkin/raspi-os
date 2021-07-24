#[cfg(not(test))]
global_asm!(include_str!("start.s"));

use super::gpu;

#[no_mangle]
pub extern "C" fn start() {
    /*let fb = gpu::init();
    
    if let Ok(fbi) = fb {
        fbi.draw();
    }*/
}
