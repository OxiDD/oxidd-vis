use std::collections::HashMap;
use std::marker::PhantomData;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::util::free_id_manager::FreeIdManager;
use crate::util::rc_refcell::MutRcRefCell;

/// An object that can be used to synchronize configuration data between Rust and TS
pub struct ConfigurationObject<T: ValueMapping<V>, V> {
    inner: MutRcRefCell<ConfigurationObjectData<V>>,
    type_data: PhantomData<T>,
}

pub struct ConfigurationObjectData<V> {
    value: V,
    value_listeners: HashMap<usize, Box<dyn Fn() -> ()>>,
    listener_ids: FreeIdManager<usize>,
}

pub trait ValueMapping<V> {
    fn to_js_value(val: &V) -> JsValue;
    fn get_children(val: &V) -> Option<Vec<AbstractConfigurationObject>>;
    fn from_js_value(js_val: JsValue) -> V;
}

pub trait Configurable {
    fn get_value(&self) -> JsValue;
    fn get_children(&self) -> Vec<AbstractConfigurationObject>;
    fn set_value(&mut self, value: JsValue) -> ();
    fn add_value_listener(&mut self, listener: Box<dyn Fn() -> ()>) -> usize;
    fn remove_value_listener(&mut self, listener: usize) -> bool;
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

impl<T: ValueMapping<V>, V> Configurable for ConfigurationObject<T, V> {
    fn get_value(&self) -> JsValue {
        let value = &self.inner.read().value;
        T::to_js_value(value)
    }

    fn set_value(&mut self, value: JsValue) -> () {
        let value = T::from_js_value(value);
        self.inner.get().value = value;

        let inner = self.inner.read();
        if let Some(l) = inner.value_listeners.get(&0) {
            let p = l.clone();
            drop(inner);
            p();
        }
        // let inner =self.inner.get();
        // inner.value = value;
        // self.inner.
    }

    fn add_value_listener(&mut self, listener: Box<dyn Fn() -> ()>) -> usize {
        let mut inner = self.inner.get();
        let id = inner.listener_ids.get_next();
        inner.value_listeners.insert(id, listener);
        id
    }

    fn remove_value_listener(&mut self, listener: usize) -> bool {
        let mut inner = self.inner.get();
        let val = inner.value_listeners.remove(&listener);
        val.is_some()
    }

    fn get_children(&self) -> Vec<AbstractConfigurationObject> {
        todo!()
    }
}

#[wasm_bindgen]
impl AbstractConfigurationObject {
    pub fn get_value(&self) -> JsValue {
        self.data.get_value()
    }

    pub fn set_value(&mut self, value: JsValue) -> () {
        self.data.set_value(value);
    }

    pub fn add_value_listener(&mut self, listener: js_sys::Function) -> usize {
        let listener = Box::new(move || {
            let this = JsValue::null();
            listener.call0(&this);
        });
        self.data.add_value_listener(listener)
    }

    pub fn remove_value_listener(&mut self, listener: usize) -> bool {
        self.data.remove_value_listener(listener)
    }

    pub fn get_children(&self) -> Vec<AbstractConfigurationObject> {
        self.data.get_children()
    }
}
