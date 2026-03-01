use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::configuration::{
    configuration_object::{
        AbstractConfigurationObject, Abstractable, Configurable, ConfigurationObject, ValueMapping,
    },
    configuration_object_types::ConfigurationObjectType,
    mutator::Mutator,
    util::js_object::JsObject,
};

///
/// A container configuration component that adds a styled contained
#[derive(Clone)]
pub struct ContainerConfig<C: Abstractable + Clone> {
    data: ConfigurationObject<ContainerConfig<C>, ContainerValue<C>>,
    child: C,
}

#[derive(Clone)]
struct ContainerValue<C: Abstractable> {
    child: C,
    style: ContainerStyle,
}

#[derive(Copy, Clone, Default)]
pub struct ContainerStyle {
    margin_top: f32,
    margin_bottom: f32,
    margin_left: f32,
    margin_right: f32,
    hidden: bool,
}
impl ContainerStyle {
    pub fn new() -> ContainerStyle {
        ContainerStyle::default()
    }
    pub fn margin_left(mut self, margin: f32) -> Self {
        self.margin_left = margin;
        self
    }
    pub fn margin_right(mut self, margin: f32) -> Self {
        self.margin_right = margin;
        self
    }
    pub fn margin_top(mut self, margin: f32) -> Self {
        self.margin_top = margin;
        self
    }
    pub fn margin_bottom(mut self, margin: f32) -> Self {
        self.margin_bottom = margin;
        self
    }

    pub fn margin_x(mut self, margin: f32) -> Self {
        self.margin_left = margin;
        self.margin_right = margin;
        self
    }
    pub fn margin_y(mut self, margin: f32) -> Self {
        self.margin_top = margin;
        self.margin_bottom = margin;
        self
    }
    pub fn margin(mut self, margin: f32) -> Self {
        self.margin_left = margin;
        self.margin_right = margin;
        self.margin_top = margin;
        self.margin_bottom = margin;
        self
    }
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }
}

impl<C: Abstractable + Clone + 'static> ContainerConfig<C> {
    pub fn new(style: ContainerStyle, data: C) -> ContainerConfig<C> {
        ContainerConfig {
            data: ConfigurationObject::new(ContainerValue {
                child: data.clone(),
                style: style,
            }),
            child: data,
        }
    }
    pub fn get_style(&self) -> ContainerStyle {
        self.data.with_value(|v| v.style.clone())
    }
    pub fn set_style(&mut self, style: ContainerStyle) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            Some(ContainerValue {
                style,
                ..cur.clone()
            })
        })
    }
}
impl<C: Abstractable + Clone + 'static> Abstractable for ContainerConfig<C> {
    fn get_abstract(&self) -> AbstractConfigurationObject {
        AbstractConfigurationObject::new(ConfigurationObjectType::Container, self.data.clone())
    }
}
impl<C: Abstractable + Clone + 'static> Deref for ContainerConfig<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.child
    }
}
impl<C: Abstractable + Clone + 'static> DerefMut for ContainerConfig<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}

impl<C: Abstractable + Clone> ValueMapping<ContainerValue<C>> for ContainerConfig<C> {
    fn to_js_value(val: &ContainerValue<C>) -> wasm_bindgen::JsValue {
        JsObject::new()
            .set("margin_top", val.style.margin_top)
            .set("margin_bottom", val.style.margin_bottom)
            .set("margin_left", val.style.margin_left)
            .set("margin_right", val.style.margin_right)
            .set("hidden", val.style.hidden)
            .into()
    }

    fn get_children(val: &ContainerValue<C>) -> Option<Vec<AbstractConfigurationObject>> {
        Some(vec![val.child.get_abstract()])
    }

    fn from_js_value(
        _js_val: wasm_bindgen::JsValue,
        _cur_val: &ContainerValue<C>,
    ) -> Option<ContainerValue<C>> {
        None
    }
}
