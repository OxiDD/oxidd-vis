use wasm_bindgen::JsValue;

use crate::configuration::configuration_object::{
    AbstractConfigurationObject, ConfigurationObject, ValueMapping,
};

#[derive(Clone)]
pub struct IntConfig {
    data: ConfigurationObject<IntConfig, usize>,
}

impl ValueMapping<usize> for IntConfig {
    fn to_js_value(val: usize) -> JsValue {
        JsValue::from_f64(val as f64)
    }
    fn from_js_value(js_val: JsValue) -> usize {
        js_val.as_f64().map(|val| val as usize).unwrap_or_default()
    }
    fn get_children(val: usize) -> Option<Vec<AbstractConfigurationObject>> {
        None
    }
}
