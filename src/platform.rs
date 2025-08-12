//! module that abstracts platform specific implementations

#[cfg(feature = "raspi3")]
pub mod raspi3;

#[cfg(feature = "raspi3")]
pub use raspi3::*;

#[cfg(feature = "qemu")]
mod qemu;

#[cfg(feature = "qemu")]
pub use qemu::*;
