//! Like `Cell<Option<T>>`, but doesn’t require `T: Copy`.
//! Specialization of https://github.com/SimonSapin/rust-movecell

use std::cell::UnsafeCell;
use std::mem;
use std::rc::{Rc, Weak};

pub struct CellOption<T>(UnsafeCell<Option<T>>);

impl<T> CellOption<T> {
    #[inline]
    pub fn new(x: Option<T>) -> Self {
        CellOption(UnsafeCell::new(x))
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        unsafe {
            (*self.0.get()).is_none()
        }
    }

    #[inline]
    pub fn replace(&self, x: Option<T>) -> Option<T> {
        unsafe {
            mem::replace(&mut *self.0.get(), x)
        }
    }

    #[inline]
    pub fn take(&self) -> Option<T> {
        unsafe {
            (*self.0.get()).take()
        }
    }
}

impl<T> CellOption<Weak<T>> {
    #[inline]
    pub fn upgrade(&self) -> Option<Rc<T>> {
        unsafe {
            match *self.0.get() {
                Some(ref weak) => weak.upgrade(),
                None => None,
            }
        }
    }
}

impl<T> CellOption<T> where T: WellBehavedClone {
    #[inline]
    pub fn clone_inner(&self) -> Option<T> {
        unsafe {
            (*self.0.get()).clone()
        }
    }
}

/**
    A Clone impl that will not access the cell again through reference cycles,
    which would introduce mutable aliasing.

    Incorrect example:

    ```rust
    struct Evil(Box<u32>, Rc<CellOption<Evil>>);
    impl Clone for Evil {
        fn clone(&self) -> Self {
            mem::drop(self.1.take());  // Mess with the "other" node, which might be `self`.
            Evil(
                self.0.clone(),  // use after free!
                Rc::new(CellOption::new(None))
            )
        }
    }
    unsafe impl WellBehavedClone for Evil {}  // Wrong.

    let a = Rc::new(CellOption::new(None));
    a.set(Some(Evil(Box::new(5), a.clone())));  // Make a reference cycle.
    a.clone_inner();
    ```

*/
unsafe trait WellBehavedClone: Clone {}
unsafe impl<T> WellBehavedClone for Rc<T> {}
unsafe impl<T> WellBehavedClone for Weak<T> {}
