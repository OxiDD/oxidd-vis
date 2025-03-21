use std::rc::Rc;

use super::{derived::Derived, watchable::Watchable};

pub trait WatchableUtils<X> {
    fn map<Y: 'static, F: Fn(Rc<X>) -> Y + 'static>(self, map: F) -> impl Watchable<Output = Y>;
}

impl<X, W: Watchable<Output = X> + 'static> WatchableUtils<X> for W {
    fn map<Y: 'static, F: Fn(Rc<X>) -> Y + 'static>(self, map: F) -> impl Watchable<Output = Y> {
        Derived::new(move |t| map(self.watch(t)))
    }
}
