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
    data: Rc<C>,
}

struct CompositeValue(Vec<Box<dyn Abstractable>>);

impl<C: 'static> CompositeConfig<C> {
    pub fn new<F: Fn(&C) -> Vec<Box<dyn Abstractable>>>(
        data: C,
        children: F,
    ) -> CompositeConfig<C> {
        CompositeConfig {
            config: ConfigurationObject::new(CompositeValue(children(&data))),
            data: Rc::new(data),
        }
    }
}

impl<C: 'static> Deref for CompositeConfig<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.data
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
        JsValue::null()
    }

    fn get_children(val: &CompositeValue) -> Option<Vec<AbstractConfigurationObject>> {
        Some(val.0.iter().map(|a| a.get_abstract()).collect_vec())
    }

    fn from_js_value(js_val: JsValue, cur_val: &CompositeValue) -> Option<CompositeValue> {
        None
    }
}
