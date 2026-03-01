use std::ops::{Deref, DerefMut};

use wasm_bindgen::prelude::*;

pub struct Mutator<R, F> {
    perform: Box<dyn FnOnce() -> Return<R, F>>,
    signal: Box<dyn FnOnce(F) -> ()>,
}

pub struct Return<R, P> {
    result: R,
    pass: P,
}
impl<R> Return<R, ()> {
    pub fn new(result: R) -> Return<R, ()> {
        Return { result, pass: () }
    }
}
impl<R, P> Return<R, P> {
    pub fn pass(result: R, pass: P) -> Return<R, P> {
        Return { result, pass }
    }
}

impl Mutator<(), ()> {
    pub fn dummy() -> Mutator<(), ()> {
        Mutator {
            perform: Box::new(|| Return::new(())),
            signal: Box::new(|_| ()),
        }
    }
}
impl<R: 'static, F: 'static> Mutator<R, F> {
    pub fn new<P: FnOnce() -> Return<R, F> + 'static, S: FnOnce(F) -> () + 'static>(
        perform: P,
        signal: S,
    ) -> Mutator<R, F> {
        Mutator {
            perform: Box::new(perform),
            signal: Box::new(signal),
        }
    }

    pub fn commit(self) -> R {
        let result = (self.perform)();
        (self.signal)(result.pass);
        result.result
    }

    pub fn chain<R2: 'static, F2: 'static>(self, next: Mutator<R2, F2>) -> Mutator<R2, (F, F2)> {
        let self_perform = self.perform;
        let next_perform = next.perform;
        let self_signal = self.signal;
        let next_signal = next.signal;
        Mutator::new(
            || {
                let result1 = (self_perform)();
                let result2 = (next_perform)();
                Return::pass(result2.result, (result1.pass, result2.pass))
            },
            |(pass1, pass2)| {
                (self_signal)(pass1);
                (next_signal)(pass2);
            },
        )
    }

    pub fn dyn_chain<R2: 'static, F2: 'static, C: FnOnce(R) -> Mutator<R2, F2> + 'static>(
        self,
        dyn_next: C,
    ) -> Mutator<R2, (F, F2, Box<dyn FnOnce(F2) -> ()>)> {
        let self_perform = self.perform;
        let self_signal = self.signal;
        Mutator::new(
            || {
                let result1 = (self_perform)();
                let next = (dyn_next)(result1.result);
                let next_signal = next.signal;
                let result2 = (next.perform)();
                Return::pass(result2.result, (result1.pass, result2.pass, next_signal))
            },
            |(pass1, pass2, next_signal)| {
                (self_signal)(pass1);
                (next_signal)(pass2);
            },
        )
    }

    pub fn map<V: 'static, M: FnOnce(R) -> V + 'static>(self, map: M) -> Mutator<V, F> {
        let self_perform = self.perform;
        let self_signal = self.signal;
        Mutator::new(
            || {
                let result = self_perform();
                let value = map(result.result);
                Return::pass(value, result.pass)
            },
            self_signal,
        )
    }
}

/// Performs inline modification of a given value
pub fn modify<V, A: 'static, B: 'static, M: Fn(&V) -> Mutator<A, B>>(value: V, modify: M) -> V {
    modify(&value).commit();
    value
}

/// Mutator callbacks for communicating with JS
#[wasm_bindgen]
pub struct MutatorCallbacks {
    perform: Option<Box<dyn FnOnce() -> Return<(), ()>>>,
    signal: Option<Box<dyn FnOnce(()) -> ()>>,
}

impl MutatorCallbacks {
    pub fn new(mutator: Mutator<(), ()>) -> MutatorCallbacks {
        MutatorCallbacks {
            perform: Some(mutator.perform),
            signal: Some(mutator.signal),
        }
    }
}
#[wasm_bindgen]
impl MutatorCallbacks {
    pub fn perform(&mut self) -> () {
        let Some(perform) = self.perform.take() else {
            return;
        };
        (perform)();
    }
    pub fn signal(&mut self) -> () {
        let Some(signal) = self.signal.take() else {
            return;
        };
        (signal)(());
    }
}
