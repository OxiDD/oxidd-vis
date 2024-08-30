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

use oxidd::{bdd::BDDFunction, util::AllocResult, BooleanFunction};
use types::{bdd_drawer::BDDDiagram, qdd_drawer::QDDDiagram};

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
pub fn create_diagram_from_dddmp(dddmp: String) -> Option<DiagramBox> // And some DD type param
{
    util::panic_hook::set_panic_hook();

    let build = || -> AllocResult<DiagramBox> {
        let manager_ref = DummyManagerRef::from(&DummyManager::new());

        Ok(DiagramBox::new(Box::new(QDDDiagram::<
            DummyManagerRef,
            DummyFunction,
        >::new(
            manager_ref,
            |manager_ref| {
                // let res =
                //     DummyFunction::from_dddmp(manager_ref, include_str!(r"..\..\data\qdd.dddmp"));
                let res = DummyFunction::from_dddmp(manager_ref, &dddmp[..]);
                res
            },
        ))))
    };
    match build() {
        Ok(res) => Some(res),
        _ => None,
    }
}
