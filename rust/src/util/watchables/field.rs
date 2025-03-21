use std::{cell::RefCell, rc::Rc};

use super::{
    mutator::Mutator,
    trackers::Trackers,
    watchable::{DataState, Listener, Observer, Watchable, WatchableState},
};

pub struct Field<X> {
    inner: Rc<RefCell<FieldInner<X>>>,
}
struct FieldInner<X> {
    val: Rc<X>,
    dependents: Trackers,
}

const LOOPMESSAGE: &str = "Dependency loop detected!";
impl<X> Watchable for Field<X> {
    type Output = X;

    fn get(&self) -> Rc<Self::Output> {
        self.inner.try_borrow().expect(LOOPMESSAGE).val.clone()
    }
}
impl<X> WatchableState for Field<X> {
    fn state(&self) -> DataState {
        self.inner
            .try_borrow()
            .expect(LOOPMESSAGE)
            .dependents
            .get_state()
    }

    fn observe<T: Listener + 'static>(&self, tracker: T) -> Observer {
        self.inner
            .try_borrow()
            .expect(LOOPMESSAGE)
            .dependents
            .observe(tracker)
    }
}

impl<X: 'static> Field<X> {
    pub fn new(init: X) -> Self {
        Field {
            inner: Rc::new(RefCell::new(FieldInner {
                val: Rc::new(init),
                dependents: Trackers::new(DataState::UpToDate),
            })),
        }
    }

    #[must_use = "Only once the mutator is committed, will the field be changed"]
    /// Creates a mutator that when committed changes the value, after committing the mutation, the state of this field is DataState::UpToDate again
    pub fn set(&mut self, val: X) -> Mutator {
        let inner = self.inner.clone();
        Mutator::new_pass(
            move || {
                inner
                    .try_borrow()
                    .expect(LOOPMESSAGE)
                    .dependents
                    .change_state(DataState::Outdated);
                inner.try_borrow_mut().expect(LOOPMESSAGE).val = Rc::new(val);
                (inner, ())
            },
            |inner| {
                inner
                    .try_borrow()
                    .expect(LOOPMESSAGE)
                    .dependents
                    .change_state(DataState::UpToDate);
            },
        )
    }

    /// Creates a readonly reference to this field data
    pub fn read(&self) -> ReadonlyField<X> {
        ReadonlyField {
            inner: self.inner.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ReadonlyField<X> {
    inner: Rc<RefCell<FieldInner<X>>>,
}

impl<X> Watchable for ReadonlyField<X> {
    type Output = X;

    fn get(&self) -> Rc<Self::Output> {
        self.inner.try_borrow().expect(LOOPMESSAGE).val.clone()
    }
}
impl<X> WatchableState for ReadonlyField<X> {
    fn state(&self) -> DataState {
        self.inner
            .try_borrow()
            .expect(LOOPMESSAGE)
            .dependents
            .get_state()
    }

    fn observe<T: Listener + 'static>(&self, tracker: T) -> Observer {
        self.inner
            .try_borrow()
            .expect(LOOPMESSAGE)
            .dependents
            .observe(tracker)
    }
}
