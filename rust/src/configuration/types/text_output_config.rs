use wasm_bindgen::JsValue;

use crate::{
    configuration::{
        configuration_object::{
            AbstractConfigurationObject, Abstractable, ConfigurationObject, ValueMapping,
        },
        configuration_object_types::ConfigurationObjectType,
        mutator::Mutator,
        util::js_object::JsObject,
    },
    util::logging::console,
};

#[derive(Clone)]
pub struct TextOutputConfig {
    data: ConfigurationObject<TextOutputConfig, TextOutputValue>,
}

#[derive(Clone)]
struct TextOutputValue {
    output: Option<String>,
    output_version: usize,
    auto_copy: bool,
}

impl TextOutputConfig {
    pub fn new(auto_copy: bool) -> TextOutputConfig {
        TextOutputConfig {
            data: ConfigurationObject::new(TextOutputValue {
                output: None,
                output_version: 0,
                auto_copy,
            }),
        }
    }

    pub fn set(&mut self, output: String) -> Mutator<(), ()> {
        console::log!("Set value");
        self.data.set_value(|cur| {
            Some(TextOutputValue {
                output: Some(output),
                output_version: cur.output_version + 1,
                ..cur.clone()
            })
        })
    }
}
impl Abstractable for TextOutputConfig {
    fn get_abstract(&self) -> AbstractConfigurationObject {
        AbstractConfigurationObject::new(ConfigurationObjectType::TextOutput, self.data.clone())
    }
}

impl ValueMapping<TextOutputValue> for TextOutputConfig {
    fn to_js_value(val: &TextOutputValue) -> JsValue {
        JsObject::new()
            .set("output", val.output.clone())
            .set("outputVersion", val.output_version)
            .set("autoCopy", val.auto_copy)
            .into()
    }

    fn from_js_value(js_val: JsValue, cur_val: &TextOutputValue) -> Option<TextOutputValue> {
        let obj = JsObject::load(js_val);
        let output = obj.get("output").and_then(|v| v.as_string());
        let output_version = obj
            .get("outputVersion")
            .and_then(|v| v.as_f64().map(|val| val as usize))
            .unwrap_or_default();
        let auto_copy = obj
            .get("autoCopy")
            .and_then(|v| v.as_bool())
            .unwrap_or_default();
        Some(TextOutputValue {
            // output,
            // output_version,
            // auto_copy,
            ..cur_val.clone()
        })
    }

    fn get_children(val: &TextOutputValue) -> Option<Vec<AbstractConfigurationObject>> {
        None
    }
}
