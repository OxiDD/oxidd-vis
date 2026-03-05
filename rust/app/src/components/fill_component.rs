use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::dyn_component::DynComp,
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::BoolWatchable,
};

/// Fill component which wraps a child component and controls whether it fills its container.
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct FillComp {
    /// The content component rendered inside the fill.
    #[getter]
    #[builder(finish_fn, into)]
    content: DynComp,
    /// Whether to fill the full width of the parent.
    #[getter]
    #[setter(bool, true)]
    full_width: BoolWatchable,
    /// Whether to fill the full height of the parent.
    #[getter]
    #[setter(bool, true)]
    full_height: BoolWatchable,
}

impl FillComp {
    /// Creates a fill component with the specified flags and content.
    pub fn new(content: impl Into<DynComp> + 'static) -> Self {
        FillComp::builder().build(content)
    }
}

impl Into<Component> for FillComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::Fill(self))
    }
}
