use wasm_bindgen::JsValue;

use crate::configuration::configuration_object::{
    AbstractConfigurationObject, ConfigurationObject, GetConfigurable, TypeMapping,
};

pub struct IntConfig {
    data: ConfigurationObject<IntConfig, usize, ()>,
}

impl TypeMapping<usize, ()> for IntConfig {
    fn to_js_value(val: usize) -> JsValue {
        JsValue::from_f64(val as f64)
    }
    fn form_js_value(js_val: JsValue) -> usize {
        js_val.as_f64().map(|val| val as usize).unwrap_or_default()
    }
}
impl GetConfigurable for IntConfig {
    fn get_configurable(&self) -> AbstractConfigurationObject {
        self.data.get_configurable()
    }
}
