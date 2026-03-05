use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::{dyn_component::DynComp, PanelComp},
    new_wasm_interface::{Component, ComponentOption},
};

/// A panel handle that can be dragged to open the panel in the location where dropped
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct PanelHandleComp {
    /// The main component that forms the base layer.
    #[getter]
    #[builder(finish_fn, into)]
    main: DynComp,
    /// The panel to be opened
    #[getter]
    #[builder(into)]
    panel: PanelComp,
}

impl PanelHandleComp {
    pub fn new(main: impl Into<DynComp> + 'static, panel: impl Into<PanelComp> + 'static) -> Self {
        Self::builder().panel(panel).build(main)
    }
}

impl Into<Component> for PanelHandleComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::PanelHandle(self))
    }
}
