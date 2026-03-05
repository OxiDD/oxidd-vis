use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::dyn_component::{ComponentWatchable, DynComp},
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{Constant, Field, IntoWatchable},
};

/// Overlay component that renders one component over another.
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct OverlayComp {
    /// The main component that forms the base layer.
    #[getter]
    #[builder(finish_fn, into)]
    main: DynComp,
    /// The overlay component that is rendered above `main`.
    #[getter]
    #[builder(into)]
    overlay: DynComp,
}

impl OverlayComp {
    pub fn new(overlay: impl Into<DynComp> + 'static, main: impl Into<DynComp> + 'static) -> Self {
        Self::builder().overlay(overlay).build(main)
    }
}

impl Into<Component> for OverlayComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::Overlay(self))
    }
}
