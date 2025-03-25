use wasm_bindgen::prelude::*;

#[derive(Clone)]
#[wasm_bindgen]
pub enum ConfigurationObjectType {
    Int,
    Choice,
    Label,
    Composite,
    Button,
    Panel,
    Location,
    TextOutput,
}
