use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        impl_watchable, signaller::Signaller, BoolWatchable, Field, Mutator, OptionStringField,
        OptionStringWatchable, StringWatchable,
    },
};

/// Binary input data
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct BinaryInput {
    #[builder(start_fn, into)]
    data: Field<Vec<u8>>,
    #[getter]
    #[builder(into)]
    filename: OptionStringField,
}
impl BinaryInput {
    pub fn new() -> Self {
        BinaryInput {
            data: Field::new(vec![]),
            filename: OptionStringField::new(None),
        }
    }
    fn watchable(&self) -> &Field<Vec<u8>> {
        &self.data
    }
    pub fn set(&mut self, val: Vec<u8>) -> Signaller {
        self.data.set(val.into())
    }
}
impl_watchable!(BinaryInput, Vec<u8>);
#[wasm_bindgen]
impl BinaryInput {
    #[wasm_bindgen(js_name = "set")]
    pub fn set_js(&mut self, val: Vec<u8>) -> Mutator {
        let mut field = self.data.clone();
        Mutator::exec(move || Box::new(field.set(val)))
    }
}

/// BinaryInput component
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct BinaryInputComp {
    /// The data of the component
    #[getter]
    #[builder(start_fn, into)]
    data: BinaryInput,
    /// A comma separated list of extensions
    #[getter]
    #[setter(Option<String>)]
    formats: OptionStringWatchable,
    /// Whether this input is disabled
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable,
}
impl BinaryInputComp {
    fn watchable(&self) -> &BinaryInput {
        &self.data
    }
}
impl_watchable!(BinaryInputComp, Vec<u8>);

impl Into<BinaryInputComp> for BinaryInput {
    fn into(self) -> BinaryInputComp {
        BinaryInputComp::builder(self).build()
    }
}

impl Into<Component> for BinaryInput {
    fn into(self) -> Component {
        Into::<BinaryInputComp>::into(self).into()
    }
}
impl Into<Component> for BinaryInputComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::BinaryInput(self))
    }
}
