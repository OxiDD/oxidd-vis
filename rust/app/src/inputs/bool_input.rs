use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    new_wasm_interface::Component,
    util::watchables::{impl_watchable, signaller::Signaller, BoolField, BoolWatchable, Mutator},
};

/// Boolean input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct BoolInput(BoolField);
impl BoolInput {
    pub fn new(val: bool) -> Self {
        BoolInput(BoolField::new(val))
    }
    pub fn from(val: impl Into<bool>) -> Self {
        Self::new(val.into())
    }
    fn watchable(&self) -> &BoolField {
        &self.0
    }
    pub fn set(&mut self, val: bool) -> Signaller {
        self.0.set(val.into())
    }
    pub fn toggle(&mut self) -> Signaller {
        self.0.set(!self.0.get())
    }
}
impl_watchable!(BoolInput, bool);
#[wasm_bindgen]
impl BoolInput {
    #[wasm_bindgen(js_name = "set")]
    pub fn set_js(&mut self, val: bool) -> Mutator {
        self.0.set_js(val.into())
    }
    #[wasm_bindgen(js_name = "toggle")]
    pub fn toggle_js(&mut self, val: bool) -> Mutator {
        self.0.set_js(!self.0.get())
    }
}

/// BoolInput component
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct BoolInputComp {
    /// The data of the component
    #[getter]
    #[builder(into)]
    data: BoolInput,
    /// Whether this input is disabled
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable,
}
impl BoolInputComp {
    fn watchable(&self) -> &BoolInput {
        &self.data
    }
}
impl_watchable!(BoolInputComp, bool);

impl Into<BoolInputComp> for BoolInput {
    fn into(self) -> BoolInputComp {
        BoolInputComp::builder().data(self).build()
    }
}

impl Into<Component> for BoolInput {
    fn into(self) -> Component {
        Into::<BoolInputComp>::into(self).into()
    }
}
impl Into<Component> for BoolInputComp {
    fn into(self) -> Component {
        todo!()
    }
}
