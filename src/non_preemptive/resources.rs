//! Abstractions to handle static resources

use core::cell::UnsafeCell;
use cortex_m::interrupt::Mutex;

/// UnsafeCell wrapper for resources which are shared between
/// different execution contexts.
pub type Shared<T> = Mutex<T>;

/// UnsafeCell wrapper for resources which are not shared between
/// different execution contexts.
pub struct UnShared<T> {
    inner: UnsafeCell<T>,
}

impl<T> UnShared<T> {
    pub const fn new(value: T) -> Self {
        UnShared {
            inner: UnsafeCell::new(value),
        }
    }
    #[allow(clippy::should_implement_trait)]
    pub fn borrow(&self) -> &T {
        unsafe { &*self.inner.get() }
    }
}

unsafe impl<T> Sync for UnShared<T> where T: Send {}
