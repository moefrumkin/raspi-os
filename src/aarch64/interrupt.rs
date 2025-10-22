use core::{
    arch::asm,
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use crate::platform::interrupt::IRQSource;

#[derive(Copy, Clone)]
pub enum InterruptState {
    Enabled,
    Disabled,
}

pub fn enable_irq() {
    unsafe {
        asm!("msr daifclr, 0b10");
    }
}

pub fn disable_irq() {
    unsafe { asm!("msr daifset, 0b10") }
}

pub fn set_irq_state(state: InterruptState) {
    match state {
        InterruptState::Enabled => enable_irq(),
        InterruptState::Disabled => disable_irq(),
    }
}

pub fn pop_irq_state() -> InterruptState {
    let daif: u64;

    unsafe {
        asm!("mrs {}, daif", out(reg) daif);
    }

    if (daif >> 7 & (0b1)) == 1 {
        InterruptState::Disabled
    } else {
        InterruptState::Enabled
    }
}

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
        // TODO: do we need to make sure that the irq state is atomically popped?
        IRQLockGuard {
            state: pop_irq_state(),
            data: unsafe { &mut *self.data.get() },
        }
    }

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
