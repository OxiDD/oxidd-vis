use std::{
    cell::{Ref, RefCell, RefMut},
    hash::Hash,
    ops::Deref,
    rc::Rc,
};

pub struct RcRefCell<T: ?Sized>(Rc<RefCell<T>>);
impl<T: ?Sized> RcRefCell<T> {
    pub fn read(&self) -> Ref<T> {
        self.0.borrow()
    }
    pub fn clone(&self) -> Self {
        RcRefCell(self.0.clone())
    }
}
impl<T> RcRefCell<T> {
    pub fn new(data: T) -> Self {
        RcRefCell(Rc::new(RefCell::new(data)))
    }
}

// impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<RcRefCell<U>> for RcRefCell<T> {}

#[derive(PartialEq, Eq, Clone)]
pub struct MutRcRefCell<T: ?Sized>(Rc<RefCell<T>>);
impl<T: ?Sized> MutRcRefCell<T> {
    pub fn read<'a>(&'a self) -> Ref<'a, T> {
        self.0.borrow()
    }
    pub fn get(&self) -> RefMut<T> {
        self.0.borrow_mut()
    }
    pub fn clone(&self) -> Self {
        MutRcRefCell(self.0.clone())
    }
    pub fn clone_readonly(&self) -> RcRefCell<T> {
        RcRefCell(self.0.clone())
    }
}
impl<T> MutRcRefCell<T> {
    pub fn new(data: T) -> Self {
        MutRcRefCell(Rc::new(RefCell::new(data)))
    }
}

impl<T: Hash> Hash for MutRcRefCell<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.read().hash(state);
    }
}
