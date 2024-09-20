use std::{cell::RefCell, rc::Rc};

use itertools::Itertools;
use js_sys::Array;
use wasm_bindgen::JsValue;

use crate::configuration::{
    configuration_object::{
        AbstractConfigurationObject, Abstractable, Configurable, ConfigurationObject, ValueMapping,
    },
    configuration_object_types::ConfigurationObjectType,
    mutator::Mutator,
    util::js_object::JsObject,
};

/// A configuration that allows the user from choosing between some predefined options
#[derive(Clone)]
pub struct ChoiceConfig<C: Clone> {
    data: ConfigurationObject<ChoiceConfig<C>, ChoiceValue<C>>,
}

#[derive(Clone)]
struct ChoiceValue<C: Clone> {
    options: Vec<Choice<C>>,
    selected: usize,
}

#[derive(Clone)]
pub struct Choice<C: Clone> {
    pub value: C,
    pub name: String,
}

impl<C: Clone> Choice<C> {
    pub fn new(value: C, name: &str) -> Choice<C> {
        Choice {
            value,
            name: name.to_string(),
        }
    }
}

impl<C: Clone + 'static> ChoiceConfig<C> {
    pub fn new<const L: usize>(choices: [Choice<C>; L]) -> ChoiceConfig<C> {
        if (L < 1) {
            panic!("Must provide at least 1 option");
        }
        ChoiceConfig {
            data: ConfigurationObject::new(ChoiceValue {
                options: IntoIterator::into_iter(choices).collect(),
                selected: 0,
            }),
        }
    }

    pub fn set_index(&mut self, index: usize) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            Some(ChoiceValue {
                options: cur.options.clone(),
                selected: index,
            })
        })
    }

    pub fn get(&self) -> C {
        self.data.with_value(|v| {
            v.options
                .get(v.selected)
                .unwrap_or_else(|| v.options.get(0).unwrap())
                .value
                .clone()
        })
    }

    pub fn set_options<const L: usize>(&mut self, choices: [Choice<C>; L]) -> Mutator<(), ()> {
        self.data.set_value(|cur| {
            Some(ChoiceValue {
                options: IntoIterator::into_iter(choices).collect(),
                selected: if cur.selected >= L { 0 } else { cur.selected },
            })
        })
    }

    pub fn get_options(&self) -> Vec<Choice<C>> {
        self.data.with_value(|v| v.options.clone())
    }
}

impl<C: Clone + Eq + 'static> ChoiceConfig<C> {
    pub fn set(&mut self, val: C) -> Mutator<(), ()> {
        self.data.set_value(move |cur| {
            Some(ChoiceValue {
                selected: cur
                    .options
                    .iter()
                    .find_position(|&v| v.value == val)
                    .map(|(pos, _)| pos)
                    .unwrap_or(0),
                options: cur.options.clone(),
            })
        })
    }
}

impl<C: Clone + 'static> Abstractable for ChoiceConfig<C> {
    fn get_abstract(&self) -> AbstractConfigurationObject {
        AbstractConfigurationObject::new(ConfigurationObjectType::Choice, self.data.clone())
    }
}
impl<C: Clone + 'static> ChoiceConfig<C> {
    pub fn add_value_dirty_listener<F: FnMut() -> () + 'static>(&mut self, listener: F) -> usize {
        self.data
            .add_value_dirty_listener(Rc::new(RefCell::new(listener)))
    }

    pub fn remove_value_dirty_listener(&mut self, listener: usize) -> bool {
        self.data.remove_value_dirty_listener(listener)
    }

    pub fn add_value_change_listener<F: FnMut() -> () + 'static>(&mut self, listener: F) -> usize {
        self.data
            .add_value_change_listener(Rc::new(RefCell::new(listener)))
    }

    pub fn remove_value_change_listener(&mut self, listener: usize) -> bool {
        self.data.remove_value_change_listener(listener)
    }
}

impl<C: Clone> ValueMapping<ChoiceValue<C>> for ChoiceConfig<C> {
    fn to_js_value(val: &ChoiceValue<C>) -> JsValue {
        JsObject::new()
            .set(
                "options",
                JsValue::from(
                    val.options
                        .iter()
                        .map(|v| JsValue::from(&v.name))
                        .collect::<Array>(),
                ),
            )
            .set("selected", val.selected)
            .into()
    }
    fn from_js_value(js_val: JsValue, cur: &ChoiceValue<C>) -> Option<ChoiceValue<C>> {
        let selected = JsObject::load(js_val)
            .get("selected")
            .and_then(|v| v.as_f64().map(|val| val as usize))
            .unwrap_or_default();
        Some(ChoiceValue {
            selected,
            ..cur.clone()
        })
    }

    fn get_children(_val: &ChoiceValue<C>) -> Option<Vec<AbstractConfigurationObject>> {
        None
    }
}
