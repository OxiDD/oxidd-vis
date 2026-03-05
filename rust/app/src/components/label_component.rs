use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::dyn_component::DynComp,
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{make_typed_dyn_watchable, StringWatchable},
};

#[wasm_bindgen]
#[derive(Clone)]
pub enum LabelKind {
    /// Put the label above a text field
    Above,
    /// Put the label on the left of the field
    Inline,
    /// A category label for a bigger subsection
    Category,
}
make_typed_dyn_watchable!(LabelKindWatchable, LabelKind);

/// Component that couples a text label with a specific input component.
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct LabelComp {
    /// The input component that this label describes.
    #[getter]
    #[builder(finish_fn, into)]
    input: DynComp,
    /// The text content of the label.
    #[getter]
    #[setter(String)]
    label: StringWatchable,
    /// Whether the label is rendered above or inline with the input.
    #[getter]
    #[setter(LabelKind, LabelKind::Above)]
    kind: LabelKindWatchable,
}

impl Into<Component> for LabelComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::Label(self))
    }
}
