use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

// UPSafeCell: 在单核环境下安全使用的内部可变性容器
pub struct UPSafeCell<T> {
    inner: UnsafeCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
        }
    }

    pub fn exclusive_access(&self) -> ExclusiveRef<'_, T> {
        ExclusiveRef {
            inner: unsafe { &mut *self.inner.get() },
        }
    }
}

pub struct ExclusiveRef<'a, T> {
    inner: &'a mut T,
}

impl<'a, T> Deref for ExclusiveRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> DerefMut for ExclusiveRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}
