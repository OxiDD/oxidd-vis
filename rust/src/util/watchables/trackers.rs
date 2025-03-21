use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};

use itertools::Itertools;

use super::watchable::{DataState, Listener, Observer, Watchable};

pub struct Trackers {
    state: Rc<RefCell<DataState>>,
    next_id: Rc<RefCell<u64>>,
    trackers: Rc<RefCell<HashMap<u64, Rc<Box<dyn Listener>>>>>,
}

impl Trackers {
    pub fn new(state: DataState) -> Self {
        Trackers {
            next_id: Rc::new(RefCell::new(0)),
            state: Rc::new(RefCell::new(state)),
            trackers: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn observe<T: Listener + 'static>(&self, tracker: T) -> Observer {
        let id = *self.next_id.borrow();
        *self.next_id.borrow_mut() += 1;
        self.trackers
            .borrow_mut()
            .insert(id, Rc::new(Box::new(tracker)));

        let tracker = self.trackers.clone();
        Observer::new(move || {
            tracker.borrow_mut().remove(&id);
        })
    }

    pub fn get_state(&self) -> DataState {
        *self.state.borrow()
    }

    pub fn change_state(&self, state: DataState) {
        *self.state.borrow_mut() = state;
        let trackers = self.trackers.borrow_mut();
        let vals = trackers.values().cloned().collect::<Vec<_>>();
        drop(trackers);
        for val in vals {
            val.state_changed(state);
        }
    }
}
