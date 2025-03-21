use std::{cell::RefCell, rc::Rc};

use super::{
    trackers::Trackers,
    watchable::{DataState, Listener, Observer, Tracker, Watchable, WatchableState},
};

/// A watchable value that is computed from other watchable values and has its value cached to prevent unnecessary recomputes
pub struct Derived<X: 'static> {
    compute: Rc<Box<dyn Fn(&DerivedTracker) -> X>>,
    dependents: Rc<Trackers>,
    val: Rc<RefCell<Option<Rc<X>>>>,
    // listener: Rc<RefCell<DerivedListener>>,
    dependencies: Rc<RefCell<Vec<Observer>>>,
}
impl<X> Clone for Derived<X> {
    /// Cloning shares internal state, to make optimal use of caching
    fn clone(&self) -> Self {
        Self {
            compute: self.compute.clone(),
            dependents: self.dependents.clone(),
            val: self.val.clone(),
            // listener: self.listener.clone(),
            dependencies: self.dependencies.clone(),
        }
    }
}

impl<'a, X: 'static> Derived<X> {
    /// Creates a new value, such that for function `f` and `w = Derived::new(f)`:
    /// - If `w.state() == DataState::UpToDate` then `f(...) = w.get()`
    /// - If every dependency `d` of `f` has `d.state() == DataState::UpToDate`, then `w.state() == DataState::UpToDate`
    ///
    /// For these constaints to be met, we require that:
    /// - `f` only depends on constants and other watchables
    /// - `f` obtains results from watchables only by calling `.watch(t)` with the provied tracker
    pub fn new<F: Fn(&DerivedTracker) -> X + 'static>(f: F) -> Derived<X> {
        Derived {
            compute: Rc::new(Box::new(f)),
            // When there are no more copies of this Derived instance, the dependencies are dropped, stopping observation of the dependencies
            dependencies: Rc::new(RefCell::new(Vec::new())),
            dependents: Rc::new(Trackers::new(DataState::UpToDate)),
            val: Rc::new(RefCell::new(None)),
        }
    }
}

impl<X: 'static> Watchable for Derived<X> {
    type Output = X;

    fn get(&self) -> Rc<Self::Output> {
        let val = self.val.borrow();
        if let Some(val) = &*val {
            return val.clone();
        }
        drop(val);

        // Remove old dependencies
        let mut prev_dependencies = self.dependencies.borrow_mut();
        prev_dependencies.clear(); // Drops the observers, stopping the observation
        drop(prev_dependencies);

        // Define when to invalidate the value
        let val_mut_ref = self.val.clone();
        let dependents_ref = self.dependents.clone();
        let listener = DerivedListener::new(move |dirty| {
            let new_state = if dirty {
                DataState::Outdated
            } else {
                // If all depencies just became clean, force recoompute of the next value
                *val_mut_ref.borrow_mut() = None;
                DataState::UpToDate
            };

            dependents_ref.change_state(new_state);
        });

        // Compute new value
        let dependencies = self.dependencies.clone();
        let tracker = DerivedTracker {
            on_observe: Box::new(move |dependency| {
                let mut listener = listener.clone();
                listener.init(&**dependency);
                let observer = dependency.observe(listener);
                dependencies.borrow_mut().push(observer);
            }),
        };
        let val = Rc::new((self.compute)(&tracker));

        // Stores the value
        *self.val.borrow_mut() = Some(val.clone());
        val
    }
}

impl<X: 'static> WatchableState for Derived<X> {
    fn state(&self) -> super::watchable::DataState {
        self.dependents.get_state()
    }

    fn observe<T: Listener + 'static>(&self, tracker: T) -> Observer {
        self.dependents.observe(tracker)
    }
}

pub struct DerivedTracker {
    on_observe: Box<dyn Fn(Box<&dyn Dependency>) -> ()>,
}
impl<W: Watchable + 'static> Tracker<W> for DerivedTracker {
    fn observe(&self, w: &W) {
        (self.on_observe)(Box::new(w));
    }
}

#[derive(Clone)]
struct DerivedListener {
    data: Rc<RefCell<DerivedListenerInternal>>,
}

struct DerivedListenerInternal {
    dirty_count: usize,
    on_clean_change: Box<dyn Fn(bool) -> ()>,
}
impl Listener for DerivedListener {
    fn state_changed(&self, state: DataState) {
        let delta = match state {
            DataState::Outdated => 1,
            DataState::UpToDate => -1,
        };
        self.change_count(delta);
    }
}
impl DerivedListener {
    fn new<F: Fn(bool) -> () + 'static>(callback: F) -> Self {
        DerivedListener {
            data: Rc::new(RefCell::new(DerivedListenerInternal {
                dirty_count: 0,
                on_clean_change: Box::new(callback),
            })),
        }
    }
    fn init<D: Dependency + ?Sized>(&mut self, dependency: &D) {
        if dependency.state() == DataState::Outdated {
            self.change_count(1);
        }
    }
    fn change_count(&self, delta: isize) {
        let mut data = self.data.borrow_mut();
        let was_dirty = data.dirty_count != 0;
        data.dirty_count = ((data.dirty_count as isize) + delta) as usize;
        let is_dirty = data.dirty_count != 0;
        drop(data);
        if is_dirty != was_dirty {
            let data = self.data.borrow();
            (data.on_clean_change)(is_dirty);
        }
    }
}

/// A trait that reflects the dependency aspect of watchables
trait Dependency {
    /// Retrieves the current state of the data
    fn state(&self) -> DataState;
    /// Observes data state changes until the next change, returns the function that can be invoked to remove the tracker
    fn observe(&self, tracker: DerivedListener) -> Observer;
}

impl<W: Watchable + ?Sized> Dependency for W {
    fn state(&self) -> DataState {
        self.state()
    }

    fn observe(&self, tracker: DerivedListener) -> Observer {
        self.observe(tracker)
    }
}
