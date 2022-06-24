use core::cell::{Cell, UnsafeCell};
// use core::lazy::OnceCell;
use core::ops::{Deref, DerefMut};

pub struct OnceCell<T> {
    inner: UnsafeCell<Option<T>>,
}

impl<T> Default for OnceCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> OnceCell<T> {
    pub const fn new() -> OnceCell<T> {
        OnceCell {
            inner: UnsafeCell::new(None),
        }
    }

    pub fn get(&self) -> Option<&T> {
        unsafe { &*self.inner.get() }.as_ref()
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.inner.get_mut().as_mut()
    }

    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        match self.get_or_try_init(|| Ok::<T, !>(f())) {
            Ok(val) => val,
            _ => unreachable!(),
        }
    }

    pub fn set(&self, value: T) -> Result<(), T> {
        let slot = unsafe { &*self.inner.get() };
        if slot.is_some() {
            return Err(value);
        }

        let slot = unsafe { &mut *self.inner.get() };
        *slot = Some(value);
        Ok(())
    }

    pub fn get_or_try_init<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        if let Some(val) = self.get() {
            return Ok(val);
        }

        #[cold]
        fn outlined_call<F, T, E>(f: F) -> Result<T, E>
        where
            F: FnOnce() -> Result<T, E>,
        {
            f()
        }

        let val = outlined_call(f)?;
        assert!(self.set(val).is_ok(), "reentrant init");
        Ok(self.get().unwrap())
    }
}

pub struct Lazy<T, F = fn() -> T> {
    cell: OnceCell<T>,
    init: Cell<Option<F>>,
}

impl<T, F> Lazy<T, F> {
    pub const fn new(init: F) -> Lazy<T, F> {
        Lazy {
            cell: OnceCell::new(),
            init: Cell::new(Some(init)),
        }
    }
}

impl<T, F: FnOnce() -> T> Deref for Lazy<T, F> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.cell.get_or_init(|| match self.init.take() {
            Some(f) => f(),
            None => panic!("invalid init function"),
        })
    }
}

impl<T, F: FnOnce() -> T> DerefMut for Lazy<T, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let _ = self.cell.get_or_init(|| match self.init.take() {
            Some(f) => f(),
            None => panic!("invalid init function"),
        });
        match self.cell.get_mut() {
            Some(res) => res,
            None => panic!("uninitialized"),
        }
    }
}
