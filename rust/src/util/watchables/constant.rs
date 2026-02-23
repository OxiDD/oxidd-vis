use std::{process::Output, rc::Rc};

use crate::util::watchables::watchable::{
    DataState, IntoWatchable, Listener, Observer, Watchable, WatchableState,
};

pub struct Constant<X>(Rc<X>);
impl<X> Clone for Constant<X> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<X> Constant<X> {
    pub fn new(val: X) -> Constant<X> {
        Constant(Rc::new(val))
    }
}

impl<X> WatchableState for Constant<X> {
    fn state(&self) -> DataState {
        DataState::UpToDate
    }

    fn observe(&self, _listener: Box<dyn Listener>) -> Observer {
        Observer::new(|| ())
    }
}
impl<X> Watchable for Constant<X> {
    type Output = X;
    fn get(&self) -> Rc<Self::Output> {
        self.0.clone()
    }
}

impl<X> IntoWatchable<X> for Constant<X> {
    type Output = Constant<X>;
    fn into(self) -> Self::Output {
        self
    }
}
impl<X> IntoWatchable<X> for X {
    type Output = Constant<X>;

    fn into(self) -> Self::Output {
        Constant::new(self)
    }
}
