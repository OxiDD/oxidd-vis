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
pub struct IntConfig {
    data: ConfigurationObject<IntConfig, IntValue>,
}

#[derive(Clone)]
struct IntValue {
    value: isize,
    min: Option<isize>,
    max: Option<isize>,
}

impl IntConfig {
    pub fn new(val: isize) -> IntConfig {
        IntConfig {
            data: ConfigurationObject::new(IntValue {
                value: val,
                min: None,
                max: None,
            }),
        }
    }

    pub fn get(&self) -> isize {
        self.data.with_value(|v| v.value)
    }
    pub fn set(&mut self, value: isize) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            Some(IntValue {
                value: cur.bound(value),
                ..cur.clone()
            })
        })
    }

    pub fn get_min(&self) -> Option<isize> {
        self.data.with_value(|v| v.min)
    }
    pub fn set_min(&mut self, min: Option<isize>) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            let mut new = IntValue { min, ..cur.clone() };
            new.value = new.bound(new.value);
            Some(new)
        })
    }

    pub fn get_max(&self) -> Option<isize> {
        self.data.with_value(|v| v.max)
    }
    pub fn set_max(&mut self, max: Option<isize>) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            let mut new = IntValue { max, ..cur.clone() };
            new.value = new.bound(new.value);
            Some(new)
        })
    }
}
impl Abstractable for IntConfig {
    fn get_abstract(&self) -> AbstractConfigurationObject {
        AbstractConfigurationObject::new(ConfigurationObjectType::Int, self.data.clone())
    }
}
impl ConfigObjectGetter<IntConfig, IntValue> for IntConfig {
    fn with_config_object<O, U: FnOnce(&mut ConfigurationObject<IntConfig, IntValue>) -> O>(
        &mut self,
        e: U,
    ) -> O {
        e(&mut self.data)
    }
}

impl ValueMapping<IntValue> for IntConfig {
    fn to_js_value(val: &IntValue) -> JsValue {
        JsObject::new()
            .set("value", val.value)
            .set("min", val.min)
            .set("max", val.max)
            .into()
    }
    fn from_js_value(js_val: JsValue, cur: &IntValue) -> Option<IntValue> {
        let value = JsObject::load(js_val)
            .get("value")
            .and_then(|v| v.as_f64().map(|val| val as isize))
            .unwrap_or_default();
        Some(IntValue {
            value,
            ..cur.clone()
        })
    }

    fn get_children(_val: &IntValue) -> Option<Vec<AbstractConfigurationObject>> {
        None
    }
}

impl IntValue {
    fn bound(&self, mut val: isize) -> isize {
        if let Some(max) = self.max {
            val = val.min(max)
        }
        if let Some(min) = self.min {
            val = val.max(min)
        }
        val
    }
}
