use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::JsValue;

use crate::configuration::{
    configuration_object::{
        AbstractConfigurationObject, Abstractable, ConfigObjectGetter, Configurable,
        ConfigurationObject, ValueMapping,
    },
    configuration_object_types::ConfigurationObjectType,
    util::js_object::JsObject,
};

/// A configuration that allows the user from choosing between some predefined options
#[derive(Clone)]
pub struct ButtonConfig {
    data: ConfigurationObject<ButtonConfig, ButtonValue>,
}

#[derive(Clone)]
struct ButtonValue {
    press_count: usize,
    style: ButtonStyle,
}

#[derive(Clone)]
pub enum ButtonStyle {
    Text(String),
    Plain(),
    Icon {
        /// The name of the icon, available in https://uifabricicons.azurewebsites.net/
        name: String,
        /// A custom hover description based on the use case of the button
        description: String,
    },
}

impl ButtonConfig {
    pub fn new_labeled(text: &str) -> ButtonConfig {
        ButtonConfig {
            data: ConfigurationObject::new(ButtonValue {
                press_count: 0,
                style: ButtonStyle::Text(text.to_string()),
            }),
        }
    }
    pub fn new_plain() -> ButtonConfig {
        ButtonConfig {
            data: ConfigurationObject::new(ButtonValue {
                press_count: 0,
                style: ButtonStyle::Plain(),
            }),
        }
    }
    pub fn new_icon(icon: &str, description: &str) -> ButtonConfig {
        ButtonConfig {
            data: ConfigurationObject::new(ButtonValue {
                press_count: 0,
                style: ButtonStyle::Icon {
                    name: icon.to_string(),
                    description: description.to_string(),
                },
            }),
        }
    }

    pub fn add_press_listener<F: FnMut() -> () + 'static>(&mut self, mut listener: F) -> usize {
        let mut prev_press_count = self.data.with_value(|d| d.press_count);
        let data = self.data.clone();
        self.add_value_change_listener(move || {
            let new_press_count = data.with_value(|d| d.press_count);
            if new_press_count == prev_press_count {
                return;
            }
            prev_press_count = new_press_count;
            listener();
        })
    }

    pub fn remove_press_listener(&mut self, listener: usize) -> bool {
        self.data.remove_dirty_listener(listener)
    }
}

impl Abstractable for ButtonConfig {
    fn get_abstract(&self) -> AbstractConfigurationObject {
        AbstractConfigurationObject::new(ConfigurationObjectType::Button, self.data.clone())
    }
}
impl ConfigObjectGetter<ButtonConfig, ButtonValue> for ButtonConfig {
    fn with_config_object<
        O,
        U: FnOnce(&mut ConfigurationObject<ButtonConfig, ButtonValue>) -> O,
    >(
        &mut self,
        e: U,
    ) -> O {
        e(&mut self.data)
    }
}

impl ValueMapping<ButtonValue> for ButtonConfig {
    fn to_js_value(val: &ButtonValue) -> JsValue {
        let obj = JsObject::new().set("pressCount", val.press_count);
        let obj = match &val.style {
            ButtonStyle::Text(text) => obj.set("text", text),
            ButtonStyle::Icon { name, description } => {
                obj.set("icon", name).set("text", description)
            }
            _ => obj,
        };
        obj.into()
    }
    fn from_js_value(js_val: JsValue, cur: &ButtonValue) -> Option<ButtonValue> {
        let obj = JsObject::load(js_val);
        let press_count = obj
            .get("pressCount")
            .and_then(|v| v.as_f64().map(|f| f as usize))
            .unwrap_or_default();
        // let text = obj.get("text").and_then(|v| v.as_string());
        // let icon = obj.get("icon").and_then(|v| v.as_string());
        Some(ButtonValue {
            press_count,
            // text,
            ..cur.clone()
        })
    }

    fn get_children(_val: &ButtonValue) -> Option<Vec<AbstractConfigurationObject>> {
        None
    }
}
