use std::{ops::Deref, rc::Rc};

use itertools::Itertools;
use wasm_bindgen::JsValue;

use crate::configuration::{
    configuration_object::{
        AbstractConfigurationObject, Abstractable, ConfigurationObject, ValueMapping,
    },
    configuration_object_types::ConfigurationObjectType,
};

///
/// A composite configuration that consists of multiple child configurations
#[derive(Clone)]
pub struct CompositeConfig<C> {
    config: ConfigurationObject<CompositeConfig<C>, CompositeValue>,
    readonly_data: Rc<C>,
}

struct CompositeValue {
    children: Vec<Box<dyn Abstractable>>,
    style: CompositeDirection,
}

#[derive(PartialEq, Eq)]
pub enum CompositeDirection {
    Vertical = 0,
    Horizontal = 1,
}

impl<C: 'static> CompositeConfig<C> {
    pub fn new<F: Fn(&C) -> Vec<Box<dyn Abstractable>>>(
        data: C,
        children: F,
    ) -> CompositeConfig<C> {
        CompositeConfig {
            config: ConfigurationObject::new(CompositeValue {
                children: children(&data),
                style: CompositeDirection::Vertical,
            }),
            readonly_data: Rc::new(data),
        }
    }
    pub fn new_horizontal<F: Fn(&C) -> Vec<Box<dyn Abstractable>>>(
        data: C,
        children: F,
    ) -> CompositeConfig<C> {
        CompositeConfig {
            config: ConfigurationObject::new(CompositeValue {
                children: children(&data),
                style: CompositeDirection::Horizontal,
            }),
            readonly_data: Rc::new(data),
        }
    }
}

impl<C: 'static> Deref for CompositeConfig<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        // Readonly is fine, since all config objects can be cloned to make them mutable
        &self.readonly_data
    }
}

// TODO:  add functions for adding child listeners

impl<C: 'static> Abstractable for CompositeConfig<C> {
    fn get_abstract(&self) -> AbstractConfigurationObject {
        AbstractConfigurationObject::new(ConfigurationObjectType::Composite, self.config.clone())
    }
}

impl<C> ValueMapping<CompositeValue> for CompositeConfig<C> {
    fn to_js_value(val: &CompositeValue) -> JsValue {
        JsValue::from_bool(val.style == CompositeDirection::Horizontal)
    }

    fn get_children(val: &CompositeValue) -> Option<Vec<AbstractConfigurationObject>> {
        Some(val.children.iter().map(|a| a.get_abstract()).collect_vec())
    }

    fn from_js_value(js_val: JsValue, cur_val: &CompositeValue) -> Option<CompositeValue> {
        None
    }
}
