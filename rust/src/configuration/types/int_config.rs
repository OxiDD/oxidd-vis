use std::rc::Rc;

use js_sys::{Object, Reflect};
use wasm_bindgen::JsValue;

use crate::configuration::{
    configuration_object::{
        AbstractConfigurationObject, Configurable, ConfigurationObject, ValueMapping,
    },
    configuration_object_types::ConfigurationObjectType,
    mutator::Mutator,
};

/**
 * An integer config with min and max value constraints
 */
#[derive(Clone)]
pub struct IntConfig {
    data: ConfigurationObject<IntConfig, IntValue>,
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
        self.data.set_value(move |cur| IntValue {
            value: cur.bound(value),
            ..cur.clone()
        })
    }

    pub fn get_min(&self) -> Option<isize> {
        self.data.with_value(|v| v.min)
    }
    pub fn set_min(&mut self, min: Option<isize>) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            let mut new = IntValue { min, ..cur.clone() };
            new.value = new.bound(new.value);
            new
        })
    }

    pub fn get_max(&self) -> Option<isize> {
        self.data.with_value(|v| v.max)
    }
    pub fn set_max(&mut self, max: Option<isize>) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            let mut new = IntValue { max, ..cur.clone() };
            new.value = new.bound(new.value);
            new
        })
    }

    pub fn get_abstract(&self) -> AbstractConfigurationObject {
        AbstractConfigurationObject::new(ConfigurationObjectType::Int, self.data.clone())
    }
}
impl IntConfig {
    pub fn add_value_dirty_listener<F: Fn() -> () + 'static>(&mut self, listener: F) -> usize {
        self.data.add_value_dirty_listener(Rc::new(listener))
    }

    pub fn remove_value_dirty_listener(&mut self, listener: usize) -> bool {
        self.data.remove_value_dirty_listener(listener)
    }

    pub fn add_value_change_listener<F: Fn() -> () + 'static>(&mut self, listener: F) -> usize {
        self.data.add_value_change_listener(Rc::new(listener))
    }

    pub fn remove_value_change_listener(&mut self, listener: usize) -> bool {
        self.data.remove_value_change_listener(listener)
    }
}

impl ValueMapping<IntValue> for IntConfig {
    fn to_js_value(val: &IntValue) -> JsValue {
        let obj = Object::new();
        let _ = Reflect::set(&obj, &"val".into(), &val.value.into());
        let _ = Reflect::set(&obj, &"min".into(), &val.min.into());
        let _ = Reflect::set(&obj, &"max".into(), &val.max.into());
        obj.into()
    }
    fn from_js_value(js_val: JsValue, _cur: &IntValue) -> IntValue {
        let get = |text: &str| {
            Reflect::get(&js_val, &text.into())
                .map(|v| v.as_f64().map(|val| val as isize))
                .unwrap_or_default()
        };
        let value = get("val").unwrap_or_default();
        let min = get("min");
        let max = get("max");
        IntValue { value, min, max }
    }

    fn get_children(_val: &IntValue) -> Option<Vec<AbstractConfigurationObject>> {
        None
    }
}

#[derive(Clone)]
struct IntValue {
    value: isize,
    min: Option<isize>,
    max: Option<isize>,
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
