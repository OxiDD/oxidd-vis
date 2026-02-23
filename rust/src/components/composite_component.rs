use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct CompositeComponent {}

impl Clone for CompositeComponent {
    fn clone(&self) -> Self {
        Self {}
    }
}
