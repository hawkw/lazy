use std::{cell::UnsafeCell, fmt, mem, ops};

/// A non-thread-safe lazy cell.
pub struct UnsyncLazy<T, F> {
    inner: UnsafeCell<LazyInner<T, F>>,
}

enum LazyInner<T, F> {
    Init(T),
    Uninit(F),
    Empty,
}

impl<T, F> ops::Deref for UnsyncLazy<T, F>
where
    F: FnOnce() -> T,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        loop {
            unsafe {
                // It's safe to access the inner data, as it will only be
                // mutated if it does not already exist, and this type is not
                // Sync, guarding against multiple concurrent mutations.
                let ptr = self.inner.get();
                if let LazyInner::Init(ref t) = &*ptr {
                    return t;
                }
                (*ptr).force();
            }
        }
    }
}

impl<T, F> ops::DerefMut for UnsyncLazy<T, F>
where
    F: FnOnce() -> T,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        loop {
            unsafe {
                // This function is safe as we have mut access to the cell regardless.
                let ptr = self.inner.get();
                if let LazyInner::Init(ref mut t) = &mut *ptr {
                    return t;
                }
                (*ptr).force();
            }
        }
    }
}

impl<T, F> From<F> for UnsyncLazy<T, F>
where
    F: FnOnce() -> T,
{
    fn from(f: F) -> Self {
        Self {
            inner: UnsafeCell::new(LazyInner::Uninit(f)),
        }
    }
}

impl<T: fmt::Debug, F> fmt::Debug for UnsyncLazy<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { (*self.inner.get()).fmt(f) }
    }
}

// === impl LazyInner ===

impl<T, F> LazyInner<T, F>
where
    F: FnOnce() -> T,
{
    fn force(&mut self) {
        *self = match mem::replace(self, LazyInner::Empty) {
            LazyInner::Uninit(f) => LazyInner::Init(f()),
            LazyInner::Empty => unreachable!(),
            x => x,
        }
    }
}

impl<T: fmt::Debug, F> fmt::Debug for LazyInner<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LazyInner::Init(ref t) => t.fmt(f),
            LazyInner::Uninit(_) => fmt::Display::fmt("<uninitialized>", f),
            LazyInner::Empty => fmt::Display::fmt("<empty>", f),
        }
    }
}
