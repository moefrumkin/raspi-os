use core::{
    sync::atomic::{AtomicBool, Ordering},
    cell::UnsafeCell,
    ops::{Deref, DerefMut}
};

pub struct SpinMutex<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>
}

impl<'a, T> SpinMutex<T> {
    #[allow(dead_code)]
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data)
        }
    }

    pub fn lock(&'a self) -> SpinMutexGuard<'a, T> {

        //TODO: implement lock on rpi
        //while self.lock.compare_exchange_weak(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {}

        SpinMutexGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() }
        }
    }

    pub fn execute(&self, f: impl FnOnce(&T)) {
        f(self.lock().data);
    }

    pub fn execute_mut(&mut self, f: impl FnOnce(&mut T)) {
        f(self.lock().data);
    }
}

unsafe impl<T> Sync for SpinMutex<T> {}

pub struct SpinMutexGuard<'a, T> {
    lock: &'a AtomicBool,
    data: &'a mut T
}

impl<'a, T> Deref for SpinMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T> DerefMut for SpinMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T> Drop for SpinMutexGuard<'a, T> {
    /// The dropping of the MutexGuard will release the lock it was created from.
    fn drop(&mut self) {
        //self.lock.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::{SpinMutex};

    #[test]
    fn test_spin_mutex() {
        let state = SpinMutex::new(0);

        assert_eq!(*state.lock().data, 0);

        *state.lock().data = 9;

        assert_eq!(*state.lock().data, 9);
    }
}
