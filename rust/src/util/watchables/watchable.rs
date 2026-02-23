use std::{fmt, rc::Rc};

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
/// - when executing in sequence (with anything not containing `o` being dropped at the ...) without assertion violations:
///     ```
///     let s = w.state();
///     let o = w.observe(c);
///     ...
///     let u = w.state();
///     assert!(s != u);
///     ```
///     then there was a call `c.state_changed(w.state())` before `u` was assigned
/// - when executing in sequence (with anything not containing `observe(c)` at the ...):
///     ```
///     ...
///     let o = w.observe(c);
///     ```
///     then no calls `c.state_changed(w.state())` will be made before the observe call
/// - when executing in sequence (with anything not containing `observe(o)` at the ...):
///     ```
///     let o = w.observe(c);
///     ...
///     drop(o);
///     ...
///     ```
///     then no calls `c.state_changed(w.state())` will be made after dropping the observer
/// - `w.watch(t)` behaves equivalently to `{t.observe(&w); w.get()}`
pub trait Watchable: WatchableState {
    type Output;
    /// Retrieves the data of the watchable
    fn get(&self) -> Rc<Self::Output>;
}

pub trait WatchableState {
    /// Retrieves the current state of the data
    fn state(&self) -> DataState;
    /// Observes data state changes until the next change.
    /// Returns the observer that performs the observation. Once the observer is dropped, no more observations will happen
    ///
    /// Note that observing without performing `.get()` may result in no state changes occurring (see the spec)
    #[must_use = "When the observer is dropped, observation automatically stops"]
    fn observe(&self, listener: Box<dyn Listener>) -> Observer;
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

pub trait IntoWatchable<X> {
    type Output: Watchable<Output = X>;
    fn into(self) -> Self::Output;
}
