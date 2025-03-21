use std::ops::{Deref, DerefMut};

use wasm_bindgen::prelude::*;

pub struct Mutator<R = ()> {
    inner: Box<dyn MutatorInner<R>>,
}

impl Mutator<()> {
    pub fn dummy() -> Self {
        Mutator {
            inner: Box::new(Mutation::new(|| ((), ()), |_| ())),
        }
    }
}
impl<R: 'static> Mutator<R> {
    /// Creates a new mutator that first calls perform, and then calls signal, upon committing
    pub fn new<P: FnOnce() -> R + 'static, S: FnOnce() -> () + 'static>(
        perform: P,
        signal: S,
    ) -> Self {
        Mutator {
            inner: Box::new(Mutation::new(|| ((), perform()), |_| signal())),
        }
    }
    /// Creates a new mutator that first calls perform, and then calls signal using the result from perform, upon committing
    pub fn new_pass<F: 'static, P: FnOnce() -> (F, R) + 'static, S: FnOnce(F) -> () + 'static>(
        perform: P,
        signal: S,
    ) -> Self {
        Mutator {
            inner: Box::new(Mutation::new(perform, signal)),
        }
    }

    /// Commits the given mutation
    pub fn commit(self) -> R {
        let mut inner = self.inner;
        let result = inner.perform();
        inner.signal();
        result
    }

    /// Cancels the mutation
    pub fn cancel(mut self) {
        self.inner.cancel();
    }

    /// Creates a new mutation that chains the given mutations together, first performing both in sequence, then signalling both in sequence
    pub fn chain<R2: 'static>(mut self, mut next: Mutator<R2>) -> Mutator<R2> {
        Mutator::new_pass(
            move || {
                self.inner.perform();
                let result = next.inner.perform();
                ((self, next), result)
            },
            |(mut first, mut next)| {
                first.inner.signal();
                next.inner.signal();
            },
        )
    }

    /// Creates a new mutation that performs this mutation, then obtains the second mutator and performs it, and finally signals both mutators in order
    pub fn dyn_chain<R2: 'static, C: FnOnce(R) -> Mutator<R2> + 'static>(
        mut self,
        dyn_next: C,
    ) -> Mutator<R2> {
        Mutator::new_pass(
            move || {
                let result = self.inner.perform();
                let mut next = dyn_next(result);
                let result = next.inner.perform();
                ((self, next), result)
            },
            |(mut first, mut next)| {
                first.inner.signal();
                next.inner.signal();
            },
        )
    }

    /// Maps the result of the given mutation to another result
    pub fn map<R2: 'static, M: FnOnce(R) -> R2 + 'static>(mut self, map: M) -> Mutator<R2> {
        Mutator::new_pass(
            || {
                let result = self.inner.perform();
                (self, map(result))
            },
            |mut mutator| mutator.inner.signal(),
        )
    }
}

trait MutatorInner<R> {
    fn perform(&mut self) -> R;
    fn signal(&mut self);
    fn cancel(&mut self);
}
struct Mutation<R = (), F = ()> {
    pass: Option<F>,
    perform: Option<Box<dyn FnOnce() -> (F, R)>>,
    signal: Option<Box<dyn FnOnce(F) -> ()>>,
    canceled: bool,
}
impl<R, F> MutatorInner<R> for Mutation<R, F> {
    fn perform(&mut self) -> R {
        let perform = self.perform.take().unwrap();
        let (pass, result) = perform();
        self.pass = Some(pass);
        result
    }

    fn signal(&mut self) {
        let signal = self.signal.take().unwrap();
        signal(self.pass.take().unwrap());
    }

    fn cancel(&mut self) {
        self.canceled = true;
    }
}
impl<R, F> Mutation<R, F> {
    fn new<P: FnOnce() -> (F, R) + 'static, S: FnOnce(F) -> () + 'static>(
        perform: P,
        signal: S,
    ) -> Self {
        Mutation {
            pass: None,
            perform: Some(Box::new(perform)),
            signal: Some(Box::new(signal)),
            canceled: false,
        }
    }
}

impl<R, F> Drop for Mutation<R, F> {
    fn drop(&mut self) {
        if self.signal.is_some() && !self.canceled {
            eprintln!("Mutation was dropped without being executed or canceled! If you wish to not perform the mutation, please cancel it explicitely using .cancel()");
        }
    }
}
