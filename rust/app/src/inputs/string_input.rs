use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    new_wasm_interface::Component,
    util::watchables::{
        impl_watchable, signaller::Signaller, BoolWatchable, Mutator, OptionBoolWatchable,
        OptionU32Watchable, StringField,
    },
};

/// String input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct StringInput(StringField);
impl StringInput {
    pub fn new(val: String) -> Self {
        StringInput(StringField::new(val))
    }
    pub fn from<V: Into<String>>(val: V) -> Self {
        StringInput(StringField::from(val))
    }
    fn watchable(&self) -> &StringField {
        &self.0
    }
    pub fn set(&mut self, val: impl Into<String>) -> Signaller {
        self.0.set(val.into())
    }
}
impl_watchable!(StringInput, String);
#[wasm_bindgen]
impl StringInput {
    #[wasm_bindgen(js_name = "set")]
    pub fn set_js(&mut self, val: String) -> Mutator {
        self.0.set_js(val.into())
    }
}

/// StringInput component
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct StringInputComp {
    /// The data of the component
    #[getter]
    #[builder(into)]
    data: StringInput,
    /// Whether to allow multiline inputs
    #[getter]
    #[setter(bool, false)]
    multiline: BoolWatchable,
    /// The minimum number of lines to show
    #[getter]
    #[setter(Option<u32>)]
    show_lines_min: OptionU32Watchable,
    /// The maximum number of lines to show
    #[getter]
    #[setter(Option<u32>)]
    show_lines_max: OptionU32Watchable,
    /// Whether to dynamically adjust the shown number of lines based on content
    #[getter]
    #[setter(bool, false)]
    show_lines_dynamic: BoolWatchable,
    /// Whether this input is disabled
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable,
}

impl Into<StringInputComp> for StringInput {
    fn into(self) -> StringInputComp {
        StringInputComp::builder().data(self).build()
    }
}

impl Into<Component> for StringInput {
    fn into(self) -> Component {
        Into::<StringInputComp>::into(self).into()
    }
}
impl Into<Component> for StringInputComp {
    fn into(self) -> Component {
        todo!()
    }
}
