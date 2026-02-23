use std::rc::Rc;

use crate::util::watchables::{
    DataState, IntoWatchable, Listener, Observer, Watchable, WatchableState,
};

pub struct DynWatchable<X>(Box<dyn Watchable<Output = X>>);

impl<X> DynWatchable<X> {
    pub fn new<W: Watchable<Output = X> + 'static>(watchable: W) -> Self {
        DynWatchable(Box::new(watchable))
    }
}
impl<X> Watchable for DynWatchable<X> {
    type Output = X;

    fn get(&self) -> Rc<Self::Output> {
        self.0.get()
    }
}
impl<X> WatchableState for DynWatchable<X> {
    fn state(&self) -> DataState {
        self.0.state()
    }

    fn observe(&self, listener: Box<dyn Listener>) -> Observer {
        self.0.observe(listener)
    }
}

impl<X> IntoWatchable<X> for DynWatchable<X> {
    type Output = DynWatchable<X>;
    fn into(self) -> Self::Output {
        self
    }
}
pub trait IntoDynWatchable<X> {
    fn into(self) -> DynWatchable<X>;
}
impl<X, W: Watchable<Output = X> + 'static> IntoDynWatchable<X> for W {
    fn into(self) -> DynWatchable<X> {
        DynWatchable::new(self)
    }
}
