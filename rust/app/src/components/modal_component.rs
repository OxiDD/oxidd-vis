use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::dyn_component::DynComp,
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{BoolWatchable, F32Watchable, OptionF32Watchable, U32Field},
};

/// Modal component that displays a child component in a floating overlay.
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct ModalComp {
    /// The content component rendered inside the modal.
    #[getter]
    #[builder(finish_fn, into)]
    content: DynComp,
    /// Whether the modal is currently visible.
    #[getter]
    #[setter(bool)]
    shown: BoolWatchable,
    /// Optional width of the modal in logical units.
    #[getter]
    #[setter(Option<f32>)]
    width: OptionF32Watchable,
    /// Optional height of the modal in logical units.
    #[getter]
    #[setter(Option<f32>)]
    height: OptionF32Watchable,
    /// The number of times the background is clicked on
    #[getter]
    #[builder(default=U32Field::new(0))]
    click_outside: U32Field,
}

impl ModalComp {
    /// Creates a modal for the given content with default visibility and size.
    pub fn new(shown: BoolWatchable, content: impl Into<DynComp> + 'static) -> Self {
        ModalComp::builder().shown(shown).build(content)
    }
}

impl Into<Component> for ModalComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::Modal(self))
    }
}
