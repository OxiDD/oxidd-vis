mod components;
mod configuration;
mod inputs;
mod new_wasm_interface;
mod traits;
mod types;
mod util;
mod wasm_interface;

use std::collections::BTreeMap;

use itertools::Itertools;
// use js_sys::Uint32Array;
use oxidd::{bdd::BDDManagerRef, ManagerRef};
use util::{logging::console, panic_hook::set_panic_hook};
// use utils::*;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, Window};

use configuration::configuration_object::ConfigurationObject;
use oxidd::{bdd::BDDFunction, util::AllocResult, BooleanFunction};
use types::{mtbdd::mtbdd_drawer::MTBDDDiagram, qdd::qdd_drawer::QDDDiagram};

use swash::{
    proxy::{CharmapProxy, MetricsProxy},
    scale::{ScaleContext, ScalerBuilder, StrikeWith},
    shape::{ShapeContext, ShaperBuilder},
    Action, CacheKey, Charmap, FontRef,
};

use crate::{
    components::button_component::ButtonComp,
    util::{
        dummy_bdd::{DummyBDDFunction, DummyBDDManager, DummyBDDManagerRef},
        watchables::{ClonableWatchableUtils, Field, WatchableUtils},
    },
    wasm_interface::DiagramBox,
};

#[wasm_bindgen]
pub fn create_qdd_diagram() -> Option<DiagramBox> // And some DD type param
{
    set_panic_hook();
    Some(DiagramBox::new(Box::new(QDDDiagram::new())))
}

#[wasm_bindgen]
pub fn create_mtbdd_diagram() -> Option<DiagramBox> // And some DD type param
{
    set_panic_hook();
    Some(DiagramBox::new(Box::new(MTBDDDiagram::new())))
}

#[wasm_bindgen]
pub fn test() -> ButtonComp {
    let text = Field::from("test");
    let description = Field::new("hoi".to_string());
    let comp = ButtonComp::builder()
        .text(text.option())
        .icon("alarm")
        .build();
    comp
}
