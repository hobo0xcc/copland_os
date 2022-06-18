use core::cell::Cell;
use core::lazy::OnceCell;
use core::ops::{Deref, DerefMut};

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
