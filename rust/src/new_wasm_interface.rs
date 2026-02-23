use std::rc::Rc;

use crate::{
    components::{button_component::ButtonComponent, composite_component::CompositeComponent},
    configuration::configuration_object::AbstractConfigurationObject,
    types::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceRemainder,
    util::rectangle::Rectangle,
};

use super::traits::{Diagram, DiagramSection, DiagramSectionDrawer};
use itertools::Itertools;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
struct Component {
    component: ComponentOption,
}

enum ComponentOption {
    Button(ButtonComponent),
    Composite(CompositeComponent),
}

use ComponentOption::*;
#[wasm_bindgen]
impl Component {
    pub fn as_button(&self) -> Option<ButtonComponent> {
        match &self.component {
            Button(button) => Some(button.clone()),
            _ => None,
        }
    }
    pub fn as_composite(&self) -> Option<CompositeComponent> {
        match &self.component {
            Composite(composite) => Some(composite.clone()),
            _ => None,
        }
    }
}
