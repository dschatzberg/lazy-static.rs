#[cfg(not(feature="nostd"))]
use std::sync::Once;

#[cfg(not(feature="nightly"))]
use std::mem::transmute;
#[cfg(all(feature="nightly", not(feature="nostd")))]
use std::cell::UnsafeCell;
#[cfg(all(feature="nightly", not(feature="nostd")))]
use std::sync::ONCE_INIT;
#[cfg(feature="nostd")]
use core::cell::UnsafeCell;
#[cfg(feature="nostd")]
use core::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT};
#[cfg(feature="nostd")]
use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};

#[cfg(all(feature="nightly", not(feature="nostd")))]
pub struct Lazy<T: Sync>(UnsafeCell<Option<T>>, Once);

#[cfg(not(feature="nightly"))]
pub struct Lazy<T: Sync>(pub *const T, pub Once);

#[cfg(feature="nostd")]
pub struct Lazy<T: Sync>(UnsafeCell<Option<T>>, AtomicUsize);

#[cfg(all(feature="nightly", not(feature="nostd")))]
impl<T: Sync> Lazy<T> {
    #[inline(always)]
    pub const fn new() -> Self {
        Lazy(UnsafeCell::new(None), ONCE_INIT)
    }

    #[inline(always)]
    pub fn get<F>(&'static self, f: F) -> &T
        where F: FnOnce() -> T
    {
        unsafe {
            self.1.call_once(|| {
                *self.0.get() = Some(f());
            });

            match *self.0.get() {
                Some(ref x) => x,
                None => ::std::intrinsics::unreachable(),
            }
        }
    }
}

#[cfg(not(feature="nightly"))]
impl<T: Sync> Lazy<T> {
    #[inline(always)]
    pub fn get<F>(&'static mut self, f: F) -> &T
        where F: FnOnce() -> T
    {
        unsafe {
            let r = &mut self.0;
            self.1.call_once(|| {
                *r = transmute(Box::new(f()));
            });

            &*self.0
        }
    }
}


#[cfg(feature="nostd")]
impl<T: Sync> Lazy<T> {
    #[inline(always)]
    pub const fn new() -> Self {
        Lazy(UnsafeCell::new(None), ATOMIC_USIZE_INIT)
    }

    #[inline(always)]
    pub fn get<F>(&'static self, f: F) -> &T
        where F: FnOnce() -> T
    {
        const UNINIT: usize = 0;
        const INITIALIZING: usize = 1;
        const DONE: usize = 2;
        unsafe {
            if self.1.load(Acquire) == DONE {
            } else if self.1.compare_and_swap(UNINIT, INITIALIZING, Relaxed) == UNINIT {
                *self.0.get() = Some(f());
                self.1.store(DONE, Release);
            } else {
                while self.1.load(Acquire) != DONE {}
            }

            match *self.0.get() {
                Some(ref x) => x,
                None => ::core::intrinsics::unreachable(),
            }
        }
    }
}
unsafe impl<T: Sync> Sync for Lazy<T> {}
