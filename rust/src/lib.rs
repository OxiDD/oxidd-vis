mod traits;
mod types;
mod util;
mod wasm_interface;

// use js_sys::Uint32Array;
use oxidd::ManagerRef;
// use utils::*;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, Window};

use oxidd::{bdd::BDDFunction, util::AllocResult, BooleanFunction};
use types::bdd_drawer::BDDDiagram;

use crate::wasm_interface::DiagramBox;

#[wasm_bindgen]
pub fn create_diagram() -> Option<DiagramBox> // And some DD type param
{
    util::panic_hook::set_panic_hook();

    fn build() -> AllocResult<DiagramBox> {
        // Create a manager for up to 2048 nodes, up to 1024 apply cache entries, and
        // use 8 threads for the apply algorithms. In practice, you would choose higher
        // capacities depending on the system resources.
        let manager_ref = oxidd::bdd::new_manager(2048, 1024, 1);
        let (x1, x2, x3) = manager_ref.with_manager_exclusive(|manager| {
            (
                BDDFunction::new_var(manager).unwrap(),
                BDDFunction::new_var(manager).unwrap(),
                BDDFunction::new_var(manager).unwrap(),
            )
        });
        // The APIs are designed such that out-of-memory situations can be handled
        // gracefully. This is the reason for the `?` operator.
        let res = x1.and(&x2)?.or(&x3)?;

        Ok(DiagramBox::new(Box::new(BDDDiagram::new(manager_ref, res))))
    }

    match build() {
        Ok(res) => Some(res),
        _ => None,
    }
}
