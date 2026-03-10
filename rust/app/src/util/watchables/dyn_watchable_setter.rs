use std::{cell::RefCell, rc::Rc};

use crate::util::watchables::{
    signaller::Signaller, DataState, DynSignaller, IntoWatchable, IntoWatchableSetter, Listener,
    Observer, Setter, Watchable, WatchableState,
};

pub trait WatchableSetter: Setter<Input = Self::Output> + Watchable {}
impl<S: Setter + Watchable<Output = S::Input>> WatchableSetter for S {}

pub struct DynWatchableSetter<X>(Rc<RefCell<dyn WatchableSetter<Input = X, Output = X>>>);
impl<X> Clone for DynWatchableSetter<X> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<X> DynWatchableSetter<X> {
    pub fn new<W: WatchableSetter<Input = X, Output = X> + 'static>(watchable: W) -> Self {
        DynWatchableSetter(Rc::new(RefCell::new(watchable)))
    }
}
impl<X> Watchable for DynWatchableSetter<X> {
    type Output = X;

    fn get(&self) -> Rc<Self::Output> {
        self.0.borrow().get()
    }
}
impl<X> WatchableState for DynWatchableSetter<X> {
    fn state(&self) -> DataState {
        self.0.borrow().state()
    }

    fn observe(&self, listener: Box<dyn Listener>) -> Observer {
        self.0.borrow().observe(listener)
    }
}
impl<X> Setter for DynWatchableSetter<X> {
    type Input = X;
    fn set(&mut self, val: X) -> DynSignaller {
        self.0.borrow_mut().set(val)
    }
}

// impl<X: 'static> IntoWatchableSetter<X> for DynWatchableSetter<X> {
//     type Output = DynWatchableSetter<X>;
//     fn into_watchable_setter(self) -> Self::Output {
//         self
//     }
// }
impl<X> IntoWatchable<X> for DynWatchableSetter<X> {
    type Output = DynWatchableSetter<X>;
    fn into_watchable(self) -> Self::Output {
        self
    }
}
