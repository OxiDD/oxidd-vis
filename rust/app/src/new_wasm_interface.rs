use std::rc::Rc;

use crate::{
    components::{
        button_component::ButtonComp,
        composite_component::{CompositeComp, CompositeItemComp},
        dyn_component::DynComp,
    },
    configuration::configuration_object::AbstractConfigurationObject,
    inputs::{
        bool_input::BoolInputComp, f32_input::F32InputComp, i32_input::I32InputComp,
        string_input::StringInputComp, u32_input::U32InputComp, variant_input::VariantInputComp,
    },
    types::util::graph_structure::graph_manipulators::node_presence_adjuster::PresenceRemainder,
    util::rectangle::Rectangle,
};

use super::traits::{Diagram, DiagramSection, DiagramSectionDrawer};
use itertools::Itertools;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

macro_rules! impl_as_variant {
    ($fn_name:ident, $variant:ident, $ty:ty) => {
        #[wasm_bindgen]
        impl Component {
            pub fn $fn_name(&self) -> Option<$ty> {
                if let ComponentOption::$variant(x) = &self.component {
                    Some(x.clone())
                } else {
                    None
                }
            }
        }
    };
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Component {
    component: ComponentOption,
}
impl Component {
    pub fn new(option: ComponentOption) -> Self {
        Self { component: option }
    }
}

#[derive(Clone)]
pub enum ComponentOption {
    BoolInput(BoolInputComp),
    F32Input(F32InputComp),
    I32Input(I32InputComp),
    U32Input(U32InputComp),
    StringInput(StringInputComp),
    VariantInput(VariantInputComp),
    Button(ButtonComp),
    Composite(CompositeComp),
    CompositeItem(CompositeItemComp),
    Dyn(DynComp),
}

impl_as_variant!(as_bool_input, BoolInput, BoolInputComp);
impl_as_variant!(as_f32_input, F32Input, F32InputComp);
impl_as_variant!(as_i32_input, I32Input, I32InputComp);
impl_as_variant!(as_u32_input, U32Input, U32InputComp);
impl_as_variant!(as_string_input, StringInput, StringInputComp);
impl_as_variant!(as_variant_input, VariantInput, VariantInputComp);
impl_as_variant!(as_button, Button, ButtonComp);
impl_as_variant!(as_composite, Composite, CompositeComp);
impl_as_variant!(as_composite_item, CompositeItem, CompositeItemComp);
impl_as_variant!(as_dyn, Dyn, DynComp);
