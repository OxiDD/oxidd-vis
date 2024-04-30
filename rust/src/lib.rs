mod traits;
mod types;
mod util;
mod wasm_interface;

use wasm_interface::VisualizationManager;

// use js_sys::Uint32Array;
use oxidd::ManagerRef;
// use utils::*;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, Window};

#[wasm_bindgen]
pub fn initialize() -> VisualizationManager {
    util::panic_hook::set_panic_hook();
    panic!("Not implemented");
}
