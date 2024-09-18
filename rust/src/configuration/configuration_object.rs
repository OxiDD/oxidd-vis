use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use itertools::Itertools;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::util::free_id_manager::FreeIdManager;
use crate::util::rc_refcell::MutRcRefCell;

use super::mutator::{Mutator, MutatorCallbacks, Return};

/// An object that can be used to synchronize configuration data between Rust and TS
pub struct ConfigurationObject<T: ValueMapping<V>, V> {
    inner: MutRcRefCell<ConfigurationObjectData<V>>,
    type_data: PhantomData<T>,
}

pub struct ConfigurationObjectData<V> {
    value: V,
    value_dirty_listeners: HashMap<usize, Rc<dyn Fn() -> ()>>,
    dirty_listener_ids: FreeIdManager<usize>,
    value_change_listeners: HashMap<usize, Rc<dyn Fn() -> ()>>,
    change_listener_ids: FreeIdManager<usize>,
}

pub trait ValueMapping<V> {
    fn to_js_value(val: &V) -> JsValue;
    fn get_children(val: &V) -> Option<Vec<AbstractConfigurationObject>>;
    fn from_js_value(js_val: JsValue) -> V;
}

pub trait Configurable {
    fn get_value(&self) -> JsValue;
    fn get_children(&self) -> Vec<AbstractConfigurationObject>;
    fn set_value(&mut self, value: JsValue) -> Mutator<(), ()>;
    /// Adds a listener that gets called when a value becomes outdated
    fn add_value_dirty_listener(&mut self, listener: Rc<dyn Fn() -> ()>) -> usize;
    fn remove_value_dirty_listener(&mut self, listener: usize) -> bool;
    /// Adds a listener that gets called when a value's change happens (after potentially multiple values are already marked dirty)
    fn add_value_change_listener(&mut self, listener: Rc<dyn Fn() -> ()>) -> usize;
    fn remove_value_change_listener(&mut self, listener: usize) -> bool;
}

#[wasm_bindgen]
pub struct AbstractConfigurationObject {
    data: Box<dyn Configurable>,
}
impl AbstractConfigurationObject {
    pub fn new<C: Configurable + 'static>(configurable: C) -> AbstractConfigurationObject {
        AbstractConfigurationObject {
            data: Box::new(configurable),
        }
    }
}

impl<T: ValueMapping<V>, V> Clone for ConfigurationObject<T, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            type_data: self.type_data.clone(),
        }
    }
}

impl<T: ValueMapping<V> + 'static, V: 'static> ConfigurationObject<T, V> {
    pub fn new(init: V) -> ConfigurationObject<T, V> {
        ConfigurationObject {
            type_data: PhantomData,
            inner: MutRcRefCell::new(ConfigurationObjectData {
                value: init,
                change_listener_ids: FreeIdManager::new(0),
                value_change_listeners: HashMap::new(),
                dirty_listener_ids: FreeIdManager::new(0),
                value_dirty_listeners: HashMap::new(),
            }),
        }
    }
}
impl<T: ValueMapping<V> + 'static, V: 'static> Configurable for ConfigurationObject<T, V> {
    fn get_value(&self) -> JsValue {
        let value = &self.inner.read().value;
        T::to_js_value(value)
    }

    fn set_value(&mut self, value: JsValue) -> Mutator<(), ()> {
        let inner = self.inner.clone();
        let inner2 = self.inner.clone();
        Mutator::new(
            move || {
                let value = T::from_js_value(value);
                let mut inner = inner.get();
                inner.value = value;

                let listeners = inner.value_dirty_listeners.values().cloned().collect_vec();
                drop(inner);
                for listener in listeners {
                    listener();
                }
                Return::new(())
            },
            move |_| {
                let inner = inner2.read();
                let listeners = inner.value_change_listeners.values().cloned().collect_vec();
                drop(inner);
                for listener in listeners {
                    listener();
                }
            },
        )
    }

    fn add_value_dirty_listener(&mut self, listener: Rc<dyn Fn() -> ()>) -> usize {
        let mut inner = self.inner.get();
        let id = inner.dirty_listener_ids.get_next();
        inner.value_dirty_listeners.insert(id, listener);
        id
    }

    fn remove_value_dirty_listener(&mut self, listener: usize) -> bool {
        let mut inner = self.inner.get();
        let val = inner.value_dirty_listeners.remove(&listener);
        if val.is_some() {
            inner.dirty_listener_ids.make_available(listener);
            true
        } else {
            false
        }
    }

    fn add_value_change_listener(&mut self, listener: Rc<dyn Fn() -> ()>) -> usize {
        let mut inner = self.inner.get();
        let id = inner.change_listener_ids.get_next();
        inner.value_change_listeners.insert(id, listener);
        id
    }

    fn remove_value_change_listener(&mut self, listener: usize) -> bool {
        let mut inner = self.inner.get();
        let val = inner.value_change_listeners.remove(&listener);
        if val.is_some() {
            inner.change_listener_ids.make_available(listener);
            true
        } else {
            false
        }
    }

    fn get_children(&self) -> Vec<AbstractConfigurationObject> {
        let val = &self.inner.read().value;
        T::get_children(val).unwrap_or_else(|| Vec::new())
    }
}

#[wasm_bindgen]
impl AbstractConfigurationObject {
    pub fn get_value(&self) -> JsValue {
        self.data.get_value()
    }

    pub fn set_value(&mut self, value: JsValue) -> MutatorCallbacks {
        MutatorCallbacks::new(self.data.set_value(value))
    }

    pub fn add_value_dirty_listener(&mut self, listener: js_sys::Function) -> usize {
        let listener = Rc::new(move || {
            let this = JsValue::null();
            listener.call0(&this);
        });
        self.data.add_value_dirty_listener(listener)
    }

    pub fn remove_value_dirty_listener(&mut self, listener: usize) -> bool {
        self.data.remove_value_dirty_listener(listener)
    }

    pub fn add_value_change_listener(&mut self, listener: js_sys::Function) -> usize {
        let listener = Rc::new(move || {
            let this = JsValue::null();
            listener.call0(&this);
        });
        self.data.add_value_change_listener(listener)
    }

    pub fn remove_value_change_listener(&mut self, listener: usize) -> bool {
        self.data.remove_value_change_listener(listener)
    }

    pub fn get_children(&self) -> Vec<AbstractConfigurationObject> {
        self.data.get_children()
    }
}
