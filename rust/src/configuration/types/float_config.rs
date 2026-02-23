use std::{cell::RefCell, rc::Rc};

use js_sys::{Object, Reflect};
use wasm_bindgen::JsValue;

use crate::{
    configuration::{
        configuration_object::{
            AbstractConfigurationObject, Abstractable, ConfigObjectGetter, Configurable,
            ConfigurationObject, ValueMapping,
        },
        configuration_object_types::ConfigurationObjectType,
        mutator::Mutator,
        util::js_object::JsObject,
    },
    util::{logging::console, rc_refcell::MutRcRefCell},
};

/// An integer config with min and max value constraints
#[derive(Clone)]
pub struct FloatConfig {
    data: ConfigurationObject<FloatConfig, FloatValue>,
}

#[derive(Clone)]
struct FloatValue {
    value: f32,
    min: Option<f32>,
    max: Option<f32>,
    // Value should be a multiple of this
    multiple: Option<f32>,
}

impl FloatConfig {
    pub fn new(val: f32) -> FloatConfig {
        FloatConfig {
            data: ConfigurationObject::new(FloatValue {
                value: val,
                min: None,
                max: None,
                multiple: None,
            }),
        }
    }

    pub fn get(&self) -> f32 {
        self.data.with_value(|v| v.value)
    }
    pub fn set(&mut self, value: f32) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            Some(FloatValue {
                value: cur.bound(value),
                ..cur.clone()
            })
        })
    }

    pub fn get_min(&self) -> Option<f32> {
        self.data.with_value(|v| v.min)
    }
    pub fn set_min(&mut self, min: Option<f32>) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            let mut new = FloatValue { min, ..cur.clone() };
            new.value = new.bound(new.value);
            Some(new)
        })
    }

    pub fn get_max(&self) -> Option<f32> {
        self.data.with_value(|v| v.max)
    }
    pub fn set_max(&mut self, max: Option<f32>) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            let mut new = FloatValue { max, ..cur.clone() };
            new.value = new.bound(new.value);
            Some(new)
        })
    }

    pub fn get_multiple(&self) -> Option<f32> {
        self.data.with_value(|v| v.max)
    }
    /// Sets what the value should be a multiple of
    pub fn set_multiple(&mut self, multiple: Option<f32>) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            let mut new = FloatValue {
                multiple,
                ..cur.clone()
            };
            new.value = new.bound(new.value);
            Some(new)
        })
    }
}
impl Abstractable for FloatConfig {
    fn get_abstract(&self) -> AbstractConfigurationObject {
        AbstractConfigurationObject::new(ConfigurationObjectType::Float, self.data.clone())
    }
}
impl ConfigObjectGetter<FloatConfig, FloatValue> for FloatConfig {
    fn with_config_object<O, U: FnOnce(&mut ConfigurationObject<FloatConfig, FloatValue>) -> O>(
        &mut self,
        e: U,
    ) -> O {
        e(&mut self.data)
    }
}

impl ValueMapping<FloatValue> for FloatConfig {
    fn to_js_value(val: &FloatValue) -> JsValue {
        JsObject::new()
            .set("value", val.value)
            .set("min", val.min)
            .set("max", val.max)
            .set("multiple", val.multiple)
            .into()
    }
    fn from_js_value(js_val: JsValue, cur: &FloatValue) -> Option<FloatValue> {
        let value = JsObject::load(js_val)
            .get("value")
            .and_then(|v| v.as_f64().map(|val| val as f32))
            .unwrap_or_default();
        Some(FloatValue {
            value,
            ..cur.clone()
        })
    }

    fn get_children(_val: &FloatValue) -> Option<Vec<AbstractConfigurationObject>> {
        None
    }
}

impl FloatValue {
    fn bound(&self, mut val: f32) -> f32 {
        if let Some(max) = self.max {
            val = val.min(max)
        }
        if let Some(min) = self.min {
            val = val.max(min)
        }
        if let Some(multiple) = self.multiple {
            val = (val / multiple).round() * multiple;
        }
        val
    }
}
