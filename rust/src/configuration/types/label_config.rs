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
/// A label configuration component that adds a label to a given configuration item
#[derive(Clone)]
pub struct LabelConfig<C: Abstractable + Clone> {
    data: ConfigurationObject<LabelConfig<C>, LabelValue<C>>,
    child: C,
}

#[derive(Clone)]
struct LabelValue<C: Abstractable> {
    child: C,
    label: String,
    style: LabelStyle,
}

#[derive(Copy, Clone)]
pub enum LabelStyle {
    Inline = 0,
    Above = 1,
    Category = 2,
}

impl<C: Abstractable + Clone + 'static> LabelConfig<C> {
    pub fn new(label: &str, data: C) -> LabelConfig<C> {
        Self::new_styled(label, LabelStyle::Inline, data)
    }
    pub fn new_styled(label: &str, style: LabelStyle, data: C) -> LabelConfig<C> {
        LabelConfig {
            data: ConfigurationObject::new(LabelValue {
                child: data.clone(),
                label: label.to_string(),
                style: style,
            }),
            child: data,
        }
    }
    pub fn get_label(&self) -> String {
        self.data.with_value(|v| v.label.to_string())
    }
    pub fn set_label(&mut self, label: &str) -> Mutator<(), ()> {
        let label = label.to_string();
        self.data.set_value(move |cur| {
            Some(LabelValue {
                label,
                ..cur.clone()
            })
        })
    }
    pub fn get_style(&self) -> LabelStyle {
        self.data.with_value(|v| v.style.clone())
    }
    pub fn set_style(&mut self, style: LabelStyle) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            Some(LabelValue {
                style,
                ..cur.clone()
            })
        })
    }
}
impl<C: Abstractable + Clone + 'static> Abstractable for LabelConfig<C> {
    fn get_abstract(&self) -> AbstractConfigurationObject {
        AbstractConfigurationObject::new(ConfigurationObjectType::Label, self.data.clone())
    }
}
impl<C: Abstractable + Clone + 'static> Deref for LabelConfig<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.child
    }
}
impl<C: Abstractable + Clone + 'static> DerefMut for LabelConfig<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}

impl<C: Abstractable + Clone + 'static> LabelConfig<C> {
    pub fn add_label_dirty_listener<F: FnMut() -> () + 'static>(&mut self, listener: F) -> usize {
        self.data
            .add_dirty_listener(Rc::new(RefCell::new(listener)))
    }

    pub fn remove_label_dirty_listener(&mut self, listener: usize) -> bool {
        self.data.remove_dirty_listener(listener)
    }

    pub fn add_label_change_listener<F: FnMut() -> () + 'static>(&mut self, listener: F) -> usize {
        self.data
            .add_change_listener(Rc::new(RefCell::new(listener)))
    }

    pub fn remove_label_change_listener(&mut self, listener: usize) -> bool {
        self.data.remove_change_listener(listener)
    }
}

impl<C: Abstractable + Clone> ValueMapping<LabelValue<C>> for LabelConfig<C> {
    fn to_js_value(val: &LabelValue<C>) -> wasm_bindgen::JsValue {
        JsObject::new()
            .set("label", val.label.clone())
            .set("style", val.style as u8)
            .into()
    }

    fn get_children(val: &LabelValue<C>) -> Option<Vec<AbstractConfigurationObject>> {
        Some(vec![val.child.get_abstract()])
    }

    fn from_js_value(
        _js_val: wasm_bindgen::JsValue,
        _cur_val: &LabelValue<C>,
    ) -> Option<LabelValue<C>> {
        None
    }
}
