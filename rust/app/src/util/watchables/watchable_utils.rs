use std::rc::Rc;

use crate::util::watchables::Constant;

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
    fn map<Y: 'static, F: Fn(Rc<X>) -> Y + 'static>(self, map: F) -> Derived<Y>;
}
impl<X, W: Watchable<Output = X> + 'static> WatchableUtils<X> for W {
    fn map<Y: 'static, F: Fn(Rc<X>) -> Y + 'static>(self, map: F) -> Derived<Y> {
        Derived::new(move |t| {
            t.observe(&self);
            map(self.get())
        })
    }
}

pub trait ClonableWatchableUtils<X: Clone> {
    fn option(self) -> Derived<Option<X>>;
}
impl<X: Clone, W: Watchable<Output = X> + 'static> ClonableWatchableUtils<X> for W {
    fn option(self) -> Derived<Option<X>> {
        self.map(|v| Some((*v).clone()))
    }
}

// Into watchables

pub trait IntoWatchable<X> {
    type Output: Watchable<Output = X>;
    fn into_watchable(self) -> Self::Output;
}

impl IntoWatchable<String> for &str {
    type Output = Constant<String>;

    fn into_watchable(self) -> Self::Output {
        Constant::new(self.to_string())
    }
}
impl IntoWatchable<Option<String>> for &str {
    type Output = Constant<Option<String>>;

    fn into_watchable(self) -> Self::Output {
        Constant::new(Some(self.to_string()))
    }
}
