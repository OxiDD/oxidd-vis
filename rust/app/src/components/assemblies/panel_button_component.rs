use std::rc::Rc;

use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::{ButtonComp, DynComp, PanelComp, PanelHandleComp, PartialPanelCompBuilder},
    new_wasm_interface::Component,
};

/// Assembly component representing a button that opens a panel
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct PanelButtonComp {
    /// The child component that is shown inside the panel.
    #[getter]
    #[builder(finish_fn, into)]
    content: DynComp,
    /// The button to open the panel
    #[getter]
    #[builder(into)]
    button: ButtonComp,
    /// The panel that is finished except for setting the panel content
    #[builder(into)]
    panel: Rc<dyn PartialPanelCompBuilder>,
    /// The panel with the content provided
    #[getter]
    #[builder(skip=(*panel).build(button.clicks(), content.clone().into_component()))]
    panel_out: PanelComp,
    /// The final component
    #[getter]
    #[builder(skip=PanelButtonComp::make_output(button.clone(), panel_out.clone()))]
    output: PanelHandleComp,
}

impl PanelButtonComp {
    pub fn make_output(button: ButtonComp, panel: PanelComp) -> PanelHandleComp {
        PanelHandleComp::builder().panel(panel).build(button)
    }
}

impl Into<Component> for PanelButtonComp {
    fn into(self) -> Component {
        self.output.into()
    }
}
