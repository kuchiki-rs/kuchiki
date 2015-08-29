use gc::{Gc, Trace};
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

impl<T> MoveCell<T> where T: WellBehavedClone {
    #[inline]
    pub fn clone_inner(&self) -> T {
        unsafe {
            (*self.0.get()).clone()
        }
    }
}

/**
    A `Clone` impl that will not access the cell again through reference cycles,
    which would introduce mutable aliasing.

    Incorrect example:

    ```rust
    struct Evil(Box<u32>, Rc<MoveCell<Option<Evil>>>);
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
unsafe impl<T> WellBehavedClone for Gc<T> where T: Trace {}
unsafe impl<T> WellBehavedClone for Option<T> where T: WellBehavedClone {}

impl<T> Trace for MoveCell<T> where T: WellBehavedTrace {
    custom_trace!(this, {
        // XXX is this safe?
        mark(&*this.0.get());
    });
}

/**
    A `Trace` impl that will not access the cell again through reference cycles,
    which would introduce mutable aliasing.

    Incorrect example:

    ```
    struct Evil(Box<u32>, Gc<MoveCell<Option<Evil>>>);
    impl Trace for Evil {
        custom_trace!(this, {
            mem::drop(this.1.take());  // Mess with the "other" node, which might be `this`.
            let _x: u32 = *this.0;  // use after free!
            mark(&this.0);
            panic!()
        });
    }
    unsafe impl WellBehavedTrace for Evil {}  // Wrong.

    let a = Gc::new(MoveCell::new(None));
    a.set(Some(Evil(Box::new(5), a.clone())));  // Make a reference cycle.
    ::gc::force_collect();
    ```
*/
unsafe trait WellBehavedTrace: Trace {}
unsafe impl WellBehavedTrace for ::tree::Node {}
unsafe impl<T> WellBehavedTrace for Gc<T> where T: WellBehavedTrace {}
unsafe impl<T> WellBehavedTrace for Option<T> where T: WellBehavedTrace {}
