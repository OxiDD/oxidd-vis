mod traits;
mod types;
mod util;
mod wasm_interface;

use std::collections::BTreeMap;

// use js_sys::Uint32Array;
use oxidd::{bdd::BDDManagerRef, ManagerRef};
// use utils::*;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, Window};

use oxidd::{bdd::BDDFunction, util::AllocResult, BooleanFunction};
use types::bdd_drawer::BDDDiagram;

use crate::{
    util::dummy_bdd::{DummyFunction, DummyManager, DummyManagerRef},
    wasm_interface::DiagramBox,
};

#[wasm_bindgen]
pub fn create_diagram() -> Option<DiagramBox> // And some DD type param
{
    util::panic_hook::set_panic_hook();

    fn build() -> AllocResult<DiagramBox> {
        // // Create a manager for up to 2048 nodes, up to 1024 apply cache entries, and
        // // use 8 threads for the apply algorithms. In practice, you would choose higher
        // // capacities depending on the system resources.
        // let manager_ref = oxidd::bdd::new_manager(2048, 1024, 1);
        // let (x1, x2, x3) = manager_ref.with_manager_exclusive(|manager| {
        //     (
        //         BDDFunction::new_var(manager).unwrap(),
        //         BDDFunction::new_var(manager).unwrap(),
        //         BDDFunction::new_var(manager).unwrap(),
        //     )
        // });

        // // The APIs are designed such that out-of-memory situations can be handled
        // // gracefully. This is the reason for the `?` operator.
        // let res = x1.and(&x2)?.or(&x3)?;

        // Ok(DiagramBox::new(Box::new(BDDDiagram::new(manager_ref, res))))

        // let manager_ref = oxidd_manager_index::manager::ManagerRef::from(DummyManager);
        // let res = manager_ref.with_manager_exclusive(|manager|

        // )

        let manager_ref = DummyManagerRef::from(&DummyManager::new());
        Ok(DiagramBox::new(Box::new(BDDDiagram::<
            DummyManagerRef,
            DummyFunction,
        >::new(
            manager_ref,
            |manager_ref| {
                // let res = DummyFunction::from(manager_ref, "0>1, 1>2, 0>2, 0>3, 1>3");
                // let res = DummyFunction::from(
                //     manager_ref,
                //     "0>1, 1>2, 0>2, 0>3, 1>3, 4>5, 1>4, 3>4, 1>5, 0>5",
                // );
                // let res = DummyFunction::from(manager_ref, "0>1, 1>2, 0>2, 0>3, 1>3, 1>4, 3>4");
                // let res = DummyFunction::from(
                //     manager_ref,
                //     "0>1, 1>2, 2>3, 3>4, 0>4, 4>5, 5>6, 6>7, 4>7, 7>8, 8>9, 9>10, 10>11, 8>11",
                // );
                // let res = DummyFunction::from(manager_ref, "0>1, 1>2, 2>3, 0>3");
                // let res =
                //     DummyFunction::from_dddmp(manager_ref, include_str!(r"..\..\data\b02.dddmp"));
                let res =
                    DummyFunction::from_dddmp(manager_ref, include_str!(r"..\..\data\b01.dddmp"));
                // let res = DummyFunction::from_dddmp(
                //     manager_ref,
                //     include_str!(r"..\..\data\toybox.dddmp"),
                // );
                // let res = DummyFunction::from_dddmp(
                //     manager_ref,
                //     include_str!(r"..\..\data\buildroot.dddmp"),
                // );
                // let res =
                //     DummyFunction::from_dddmp(manager_ref, include_str!(r"..\..\data\b03.dddmp"));

                // let res = DummyFunction::from_dddmp(
                //     manager_ref,
                //     include_str!(r"..\..\data\embtoolkit.dddmp"),
                // );
                res
            },
        ))))
    }

    match build() {
        Ok(res) => Some(res),
        _ => None,
    }
}
