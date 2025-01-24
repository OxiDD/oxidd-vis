mod configuration;
mod traits;
mod types;
mod util;
mod wasm_interface;

use std::collections::BTreeMap;

use itertools::Itertools;
// use js_sys::Uint32Array;
use oxidd::{bdd::BDDManagerRef, ManagerRef};
use util::logging::console;
// use utils::*;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, Window};

use configuration::configuration_object::ConfigurationObject;
use oxidd::{bdd::BDDFunction, util::AllocResult, BooleanFunction};
use types::qdd::qdd_drawer::QDDDiagram;

use swash::{
    proxy::{CharmapProxy, MetricsProxy},
    scale::{ScaleContext, ScalerBuilder, StrikeWith},
    shape::{ShapeContext, ShaperBuilder},
    Action, CacheKey, Charmap, FontRef,
};

use crate::{
    util::dummy_bdd::{DummyFunction, DummyManager, DummyManagerRef},
    wasm_interface::DiagramBox,
};

#[wasm_bindgen]
pub fn create_qdd_diagram() -> Option<DiagramBox> // And some DD type param
{
    Some(DiagramBox::new(Box::new(QDDDiagram::new())))
}
