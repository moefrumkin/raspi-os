use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::aarch64::syscall;

#[derive(Debug)]
struct Semaphore {
    value: AtomicU64,
}

impl Semaphore {
    const ORDERING: Ordering = Ordering::Relaxed;

    pub const fn new(value: u64) -> Self {
        Self {
            value: AtomicU64::new(value),
        }
    }

    pub fn wait(&self) {
        loop {
            let value = self.value.load(Self::ORDERING);

            if value == 0 {
                syscall::yield_thread(); // TODO: have this thread sleep until increment
            } else {
                let result =
                    self.value
                        .compare_exchange(value, value - 1, Self::ORDERING, Self::ORDERING);

                if !result.is_err() {
                    return;
                }
            }
        }
    }

    pub fn signal(&self) {
        loop {
            let value = self.value.load(Self::ORDERING);

            if self
                .value
                .compare_exchange(value, value + 1, Self::ORDERING, Self::ORDERING)
                .is_ok()
            {
                return;
            }
        }
    }
}

#[derive(Debug)]
pub struct SemMutex<T> {
    semaphore: Semaphore,
    data: UnsafeCell<T>,
}

pub struct SemMutexGuard<'a, T> {
    semaphore: &'a Semaphore,
    data: &'a mut T,
}

impl<'a, T> SemMutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            semaphore: Semaphore::new(1),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&'a self) -> SemMutexGuard<'a, T> {
        self.semaphore.wait();

        SemMutexGuard {
            semaphore: &self.semaphore,
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

impl<'a, T> Deref for SemMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T> DerefMut for SemMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T> Drop for SemMutexGuard<'a, T> {
    fn drop(&mut self) {
        self.semaphore.signal();
    }
}
