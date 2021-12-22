//! module that abstracts platform specific implementations

#[cfg(feature = "raspi3")]
#[cfg(not(test))]
mod raspi3;

#[cfg(not(test))]
#[cfg(feature = "raspi3")]
pub use raspi3::*;

#[cfg(feature = "qemu")]
mod qemu;

#[cfg(feature = "qemu")]
pub use qemu::*;
