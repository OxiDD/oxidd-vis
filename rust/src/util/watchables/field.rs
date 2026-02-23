use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::prelude::wasm_bindgen;

use crate::util::watchables::watchable::IntoWatchable;

use super::{
    mutator::Mutator,
    trackers::Trackers,
    watchable::{DataState, Listener, Observer, Watchable, WatchableState},
};

pub struct Field<X> {
    inner: Rc<RefCell<FieldInner<X>>>,
}
impl<X> Clone for Field<X> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
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
impl<X> IntoWatchable<X> for Field<X> {
    type Output = Field<X>;
    fn into(self) -> Self::Output {
        self
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

    fn observe(&self, tracker: Box<dyn Listener>) -> Observer {
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
        ReadonlyField(self.clone())
    }
}

/// Disallow writing to the field
pub struct ReadonlyField<X>(Field<X>);
impl<X> Clone for ReadonlyField<X> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<X> Watchable for ReadonlyField<X> {
    type Output = X;

    fn get(&self) -> Rc<Self::Output> {
        self.0.get()
    }
}
impl<X> WatchableState for ReadonlyField<X> {
    fn state(&self) -> DataState {
        self.0.state()
    }

    fn observe(&self, tracker: Box<dyn Listener>) -> Observer {
        self.0.observe(tracker)
    }
}
impl<X> IntoWatchable<X> for ReadonlyField<X> {
    type Output = ReadonlyField<X>;
    fn into(self) -> Self::Output {
        self
    }
}

/// Disallows cloning of the field
pub struct ControlledField<X>(Field<X>);

impl<X> Watchable for ControlledField<X> {
    type Output = X;

    fn get(&self) -> Rc<Self::Output> {
        self.0.get()
    }
}
impl<X> WatchableState for ControlledField<X> {
    fn state(&self) -> DataState {
        self.0.state()
    }

    fn observe(&self, tracker: Box<dyn Listener>) -> Observer {
        self.0.observe(tracker)
    }
}
impl<X: 'static> ControlledField<X> {
    pub fn new(init: X) -> Self {
        Self(Field::new(init))
    }

    #[must_use = "Only once the mutator is committed, will the field be changed"]
    /// Creates a mutator that when committed changes the value, after committing the mutation, the state of this field is DataState::UpToDate again
    pub fn set(&mut self, val: X) -> Mutator {
        self.0.set(val)
    }

    /// Creates a readonly reference to this field data
    pub fn read(&self) -> ReadonlyField<X> {
        self.0.read()
    }
}
impl<X> IntoWatchable<X> for ControlledField<X> {
    type Output = ControlledField<X>;
    fn into(self) -> Self::Output {
        self
    }
}
