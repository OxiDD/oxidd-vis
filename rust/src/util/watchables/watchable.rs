use std::{cell::RefCell, fmt, hash::Hash, ops::Deref, rc::Rc};

/// A watchable data value.
///
/// Specification for a watchable `w` and observer `o`:
/// - if `w.state() == DataState::UpToDate` then the obtained value is according to the specific watchable implementation's spec (See Field and Derived)
/// - when executing in sequence (with anything at the ...) without assertion violations:
///     ```
///     assert!(w.state()==DataState::UpToDate);
///     let a = w.get();
///     ...
///     assert!(w.state()==DataState::UpToDate);
///     let b = w.get();
///     assert!(a != b)
///     ```
///     then there has been a moment after `a` was assigned and before `b` was assigned where `w.state()==DataState::Outdated`.
/// - when executing in sequence (with anything not containing `unobserve(o)` at the ...) without assertion violations:
///     ```
///     let s = w.state();
///     w.observe(o);
///     ...
///     let u = w.state();
///     assert!(s != u);
///     ```
///     then there was a call `o.state_changed(&w)` before `u` was assigned
/// - when executing in sequence (with anything not containing `observe(o)` at the ...):
///     ```
///     ...
///     let observer = w.observe(o);
///     ```
///     then no calls `o.state_changed(&w)` will be made before the observe call
/// - when executing in sequence (with anything not containing `observe(o)` at the ...):
///     ```
///     let observer = w.observe(o);
///     ...
///     drop(observer);
///     ...
///     ```
///     then no calls `o.state_changed(&w)` will be made after dropping the observer
/// - `w.watch(t)` behaves equivalently to `{t.observe(&w); w.get()}`
pub trait Watchable: WatchableState {
    type Output;
    /// Retrieves the data of the watchable
    fn get(&self) -> Rc<Self::Output>;
    /// Obtains and observes the watchable value
    fn watch<T: Tracker<Self>>(&self, tracker: &T) -> Rc<Self::Output> {
        tracker.observe(&self);
        self.get()
    }
}

pub trait WatchableState {
    /// Retrieves the current state of the data
    fn state(&self) -> DataState;
    /// Observes data state changes until the next change.
    /// Returns the observer that performs the observation. Once the observer is dropped, no more observations will happen
    ///
    /// Note that observing without performing `.get()` may result in no state changes occuring (see the spec)
    #[must_use = "When the observer is dropped, the observation automatically stops"]
    fn observe<L: Listener + 'static>(&self, listener: L) -> Observer;
}

pub trait Tracker<W: Watchable + ?Sized> {
    fn observe(&self, w: &W);
}

pub trait Listener {
    fn state_changed(&self, state: DataState);
}

pub struct Observer {
    remove: Option<Box<dyn FnOnce() -> ()>>,
}
impl Observer {
    pub fn new<F: FnOnce() -> () + 'static>(f: F) -> Self {
        Observer {
            remove: Some(Box::new(f)),
        }
    }
    /// Stops observing the value, also automatically called when the observer is dropped
    fn remove(mut self) {
        (self.remove.take().unwrap())()
    }
}
impl Drop for Observer {
    fn drop(&mut self) {
        if let Some(remove) = self.remove.take() {
            remove();
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DataState {
    UpToDate, // The data accessed from this watchable accurately reflects its spec
    Outdated, // The data accessed from this watchable reflects an old version of the value (which may or may not accurately reflect the value according to spec)

              // Final, // The data accessed from this watchable accurately reflects its spec, and will never change again
}

impl fmt::Display for DataState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataState::UpToDate => write!(f, "Up to date"),
            DataState::Outdated => write!(f, "Outdated"),
        }
    }
}
