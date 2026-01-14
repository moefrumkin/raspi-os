//! AArch64 interrupt API
//! This API does not control platform specific interrupt features

use core::{
    arch::asm,
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InterruptState {
    Enabled,
    Disabled,
}

/// Globally enable interrupts
pub fn enable_irq() {
    unsafe {
        asm!("msr daifclr, 0b1111");
    }
}

/// Globally disable interrupts
pub fn disable_irq() {
    unsafe { asm!("msr daifset, 0b1111") }
}

/// Set whether interrupts are enabled or disabled
pub fn set_irq_state(state: InterruptState) {
    match state {
        InterruptState::Enabled => enable_irq(),
        InterruptState::Disabled => disable_irq(),
    }
}

pub fn get_irq_state() -> InterruptState {
    let daif: u64;

    unsafe {
        asm!(
            "mrs {}, daif",
            out(reg) daif
        )
    }

    if daif >> 6 == 0b1111 {
        InterruptState::Disabled
    } else {
        InterruptState::Enabled
    }
}

/// Disables interrupts and returns the interrupt state before being disabled
pub fn pop_irq_state() -> InterruptState {
    let daif: u64;

    unsafe {
        asm!(
            "mrs {}, daif",
            "msr daifset, 0b1111",
            out(reg) daif,
            options(preserves_flags, nostack)
        );
    }

    if daif >> 6 == 0b1111 {
        InterruptState::Disabled
    } else {
        InterruptState::Enabled
    }
}

/// A lock that provides synchronization by disabling interrupts when the contents are accessed.
#[derive(Debug)]
pub struct IRQLock<T> {
    data: UnsafeCell<T>,
}

impl<'a, T> IRQLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&'a self) -> IRQLockGuard<'a, T> {
        // This implementation is valid provided that the interrupt handler is correct
        // The main concern is that an interrupt occurs in the execution of pop_irq_state
        // after the state is read but before irqs are disables. However, the interrupt handler
        // returns the irq state to the same state as before the interrupt so even if an interrupt
        // occured between these instructions, the result is correct
        IRQLockGuard {
            state: pop_irq_state(),
            data: unsafe { &mut *self.data.get() },
        }
    }

    /// Execute a closure on the locked data.
    pub fn execute(&self, f: impl FnOnce(&T)) {
        f(self.lock().data);
    }

    pub fn execute_mut(&mut self, f: impl FnOnce(&mut T)) {
        f(self.lock().data);
    }
}

pub struct IRQLockGuard<'a, T> {
    state: InterruptState,
    data: &'a mut T,
}

impl<'a, T> Deref for IRQLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T> DerefMut for IRQLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T> Drop for IRQLockGuard<'a, T> {
    fn drop(&mut self) {
        set_irq_state(self.state);
    }
}
