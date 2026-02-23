use std::rc::Rc;

use bon::{builder, Builder};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::util::{
    rc_refcell::RcRefCell,
    watchables::{field::Field, watchable::Watchable},
};

#[wasm_bindgen]
pub struct ButtonComponent {
    inner: RcRefCell<ButtonComponentInner>,
}

#[derive(Builder)]
pub struct ButtonConfig {
    text: String,
    icon: Option<String>,
}

// #[wasm_bindgen]
struct ButtonComponentInner {
    pub config: Field<ButtonConfig>,
    pub clicks: Field<usize>,
}

impl Clone for ButtonComponent {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[wasm_bindgen]
struct FieldUSize {
    val: Field<usize>,
}

#[wasm_bindgen]
impl FieldUSize {
    pub fn get(&self) -> usize {
        *self.val.get()
    }
}
