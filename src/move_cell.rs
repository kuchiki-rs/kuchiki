use rc::{Rc, Weak};
use std::cell::UnsafeCell;
use std::mem;

/// Like `Cell<T>`, but doesn’t require `T: Copy`.
/// Specialization of https://github.com/SimonSapin/rust-movecell
pub struct MoveCell<T>(UnsafeCell<T>);

impl<T> MoveCell<T> {
    #[inline]
    pub fn new(x: T) -> Self {
        MoveCell(UnsafeCell::new(x))
    }

    #[inline]
    pub fn set(&self, x: T) {
        unsafe {
            *self.0.get() = x;
        }
    }

    #[inline]
    pub fn replace(&self, x: T) -> T {
        unsafe {
            mem::replace(&mut *self.0.get(), x)
        }
    }
}

impl<T> MoveCell<Option<T>> {
    #[inline]
    pub fn is_none(&self) -> bool {
        unsafe {
            (*self.0.get()).is_none()
        }
    }

    #[inline]
    pub fn take(&self) -> Option<T> {
        unsafe {
            (*self.0.get()).take()
        }
    }
}

impl<T> MoveCell<Option<Weak<T>>> {
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

impl<T> MoveCell<Option<Rc<T>>> {
    /// Return `Some` if this `Rc` is the only strong reference count,
    /// even if there are weak references.
    #[inline]
    pub fn take_if_unique_strong(&self) -> Option<Rc<T>> {
        unsafe {
            match *self.0.get() {
                None => None,
                Some(ref rc) if Rc::strong_count(rc) > 1 => None,
                // Not borrowing the `Rc<T>` here
                // as we would be invalidating that borrow while it is outstanding:
                Some(_) => self.take(),
            }
        }
    }
}

impl<T> MoveCell<T> where T: WellBehavedClone {
    #[inline]
    pub fn clone_inner(&self) -> T {
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
    struct Evil(Box<u32>, Rc<MoveCell<Option<<Evil>>>);
    impl Clone for Evil {
        fn clone(&self) -> Self {
            mem::drop(self.1.take());  // Mess with the "other" node, which might be `self`.
            Evil(
                self.0.clone(),  // use after free!
                Rc::new(MoveCell::new(None))
            )
        }
    }
    unsafe impl WellBehavedClone for Evil {}  // Wrong.

    let a = Rc::new(MoveCell::new(None));
    a.set(Some(Evil(Box::new(5), a.clone())));  // Make a reference cycle.
    a.clone_inner();
    ```

*/
unsafe trait WellBehavedClone: Clone {}
unsafe impl<T> WellBehavedClone for Rc<T> {}
unsafe impl<T> WellBehavedClone for Weak<T> {}
unsafe impl<T> WellBehavedClone for Option<T> where T: WellBehavedClone {}
