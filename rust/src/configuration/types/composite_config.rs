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

pub trait GetConfigChildren {
    fn get_children(&self) -> Vec<Box<dyn Abstractable>>;
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

impl<C: GetConfigChildren + 'static> CompositeConfig<C> {
    pub fn new(data: C) -> CompositeConfig<C> {
        CompositeConfig {
            config: ConfigurationObject::new(CompositeValue {
                children: data.get_children(),
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

// Some default "GetConfigChildren" implementations for ease of use
impl<A: Abstractable + Clone + 'static> GetConfigChildren for A {
    fn get_children(&self) -> Vec<Box<dyn Abstractable>> {
        vec![Box::new(self.clone())]
    }
}

impl<A: GetConfigChildren, B: GetConfigChildren> GetConfigChildren for (A, B) {
    fn get_children(&self) -> Vec<Box<dyn Abstractable>> {
        let mut out = self.0.get_children();
        out.extend(self.1.get_children());
        out
    }
}

impl<A: GetConfigChildren, B: GetConfigChildren, C: GetConfigChildren> GetConfigChildren
    for (A, B, C)
{
    fn get_children(&self) -> Vec<Box<dyn Abstractable>> {
        let mut out = self.0.get_children();
        out.extend(self.1.get_children());
        out.extend(self.2.get_children());
        out
    }
}
impl<A: GetConfigChildren, B: GetConfigChildren, C: GetConfigChildren, D: GetConfigChildren>
    GetConfigChildren for (A, B, C, D)
{
    fn get_children(&self) -> Vec<Box<dyn Abstractable>> {
        let mut out = self.0.get_children();
        out.extend(self.1.get_children());
        out.extend(self.2.get_children());
        out.extend(self.3.get_children());
        out
    }
}
impl<
        A: GetConfigChildren,
        B: GetConfigChildren,
        C: GetConfigChildren,
        D: GetConfigChildren,
        E: GetConfigChildren,
    > GetConfigChildren for (A, B, C, D, E)
{
    fn get_children(&self) -> Vec<Box<dyn Abstractable>> {
        let mut out = self.0.get_children();
        out.extend(self.1.get_children());
        out.extend(self.2.get_children());
        out.extend(self.3.get_children());
        out.extend(self.4.get_children());
        out
    }
}
impl<
        A: GetConfigChildren,
        B: GetConfigChildren,
        C: GetConfigChildren,
        D: GetConfigChildren,
        E: GetConfigChildren,
        F: GetConfigChildren,
    > GetConfigChildren for (A, B, C, D, E, F)
{
    fn get_children(&self) -> Vec<Box<dyn Abstractable>> {
        let mut out = self.0.get_children();
        out.extend(self.1.get_children());
        out.extend(self.2.get_children());
        out.extend(self.3.get_children());
        out.extend(self.4.get_children());
        out.extend(self.5.get_children());
        out
    }
}
impl<
        A: GetConfigChildren,
        B: GetConfigChildren,
        C: GetConfigChildren,
        D: GetConfigChildren,
        E: GetConfigChildren,
        F: GetConfigChildren,
        G: GetConfigChildren,
    > GetConfigChildren for (A, B, C, D, E, F, G)
{
    fn get_children(&self) -> Vec<Box<dyn Abstractable>> {
        let mut out = self.0.get_children();
        out.extend(self.1.get_children());
        out.extend(self.2.get_children());
        out.extend(self.3.get_children());
        out.extend(self.4.get_children());
        out.extend(self.5.get_children());
        out.extend(self.6.get_children());
        out
    }
}
impl<
        A: GetConfigChildren,
        B: GetConfigChildren,
        C: GetConfigChildren,
        D: GetConfigChildren,
        E: GetConfigChildren,
        F: GetConfigChildren,
        G: GetConfigChildren,
        H: GetConfigChildren,
    > GetConfigChildren for (A, B, C, D, E, F, G, H)
{
    fn get_children(&self) -> Vec<Box<dyn Abstractable>> {
        let mut out = self.0.get_children();
        out.extend(self.1.get_children());
        out.extend(self.2.get_children());
        out.extend(self.3.get_children());
        out.extend(self.4.get_children());
        out.extend(self.5.get_children());
        out.extend(self.6.get_children());
        out.extend(self.7.get_children());
        out
    }
}
impl<
        A: GetConfigChildren,
        B: GetConfigChildren,
        C: GetConfigChildren,
        D: GetConfigChildren,
        E: GetConfigChildren,
        F: GetConfigChildren,
        G: GetConfigChildren,
        H: GetConfigChildren,
        I: GetConfigChildren,
    > GetConfigChildren for (A, B, C, D, E, F, G, H, I)
{
    fn get_children(&self) -> Vec<Box<dyn Abstractable>> {
        let mut out = self.0.get_children();
        out.extend(self.1.get_children());
        out.extend(self.2.get_children());
        out.extend(self.3.get_children());
        out.extend(self.4.get_children());
        out.extend(self.5.get_children());
        out.extend(self.6.get_children());
        out.extend(self.7.get_children());
        out.extend(self.8.get_children());
        out
    }
}
impl<
        A: GetConfigChildren,
        B: GetConfigChildren,
        C: GetConfigChildren,
        D: GetConfigChildren,
        E: GetConfigChildren,
        F: GetConfigChildren,
        G: GetConfigChildren,
        H: GetConfigChildren,
        I: GetConfigChildren,
        J: GetConfigChildren,
    > GetConfigChildren for (A, B, C, D, E, F, G, H, I, J)
{
    fn get_children(&self) -> Vec<Box<dyn Abstractable>> {
        let mut out = self.0.get_children();
        out.extend(self.1.get_children());
        out.extend(self.2.get_children());
        out.extend(self.3.get_children());
        out.extend(self.4.get_children());
        out.extend(self.5.get_children());
        out.extend(self.6.get_children());
        out.extend(self.7.get_children());
        out.extend(self.8.get_children());
        out.extend(self.9.get_children());
        out
    }
}
