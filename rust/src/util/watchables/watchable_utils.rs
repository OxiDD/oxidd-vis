use std::rc::Rc;

use super::{derived::Derived, watchable::Watchable};

// Watch calls in both directions
pub trait Tracker<W: Watchable + ?Sized> {
    fn observe(&self, w: &W);
}
pub trait Watching<W: Watchable> {
    /// Obtains and observes the watchable value
    fn watch<T: Tracker<W>>(&self, tracker: &T) -> Rc<W::Output>;
}
impl<W: Watchable> Watching<W> for W {
    fn watch<T: Tracker<Self>>(&self, tracker: &T) -> Rc<W::Output> {
        tracker.observe(&self);
        self.get()
    }
}
pub trait Watcher<W: Watchable> {
    fn watch(&self, watchable: &W) -> Rc<W::Output>;
}
impl<W: Watchable, T: Tracker<W>> Watcher<W> for T {
    fn watch(&self, watchable: &W) -> Rc<W::Output> {
        watchable.watch(self)
    }
}

// Watchable modifiers
pub trait WatchableUtils<X> {
    fn map<Y: 'static, F: Fn(Rc<X>) -> Y + 'static>(self, map: F) -> impl Watchable<Output = Y>;
}

impl<X, W: Watchable<Output = X> + 'static> WatchableUtils<X> for W {
    fn map<Y: 'static, F: Fn(Rc<X>) -> Y + 'static>(self, map: F) -> impl Watchable<Output = Y> {
        Derived::new(move |t| {
            t.observe(&self);
            map(self.get())
        })
    }
}
