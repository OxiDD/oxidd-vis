use std::marker::PhantomData;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::util::rc_refcell::MutRcRefCell;

/// An object that can be used to synchronize configuration data between Rust and TS
pub struct ConfigurationObject<T: TypeMapping<V, D>, V, D: GetConfigurable> {
    inner: MutRcRefCell<ConfigurationObjectData<V, D>>,
    type_data: PhantomData<T>,
}

pub struct ConfigurationObjectData<V, D: GetConfigurable> {
    value: V,
    value_listeners: Vec<Box<dyn Fn() -> ()>>,
    children: Vec<D>,
    children_listeners: Vec<Box<dyn Fn() -> ()>>,
}

pub trait TypeMapping<V, D: GetConfigurable>: GetConfigurable {
    fn to_js_value(val: V) -> JsValue;
    fn form_js_value(js_val: JsValue) -> V;
}

pub trait Configurable {
    fn get_value(&self) -> JsValue;
    fn set_value(&mut self, value: JsValue) -> ();
    fn add_value_listener(&mut self, listener: Box<dyn Fn() -> ()>) -> usize;
    fn remove_value_listener(&mut self, listener: usize) -> bool;
    fn get_children(&self) -> Vec<AbstractConfigurationObject>;
    fn insert_child(&mut self, index: usize) -> Option<AbstractConfigurationObject>;
    fn remove_child(&mut self, index: usize) -> bool;
    fn add_children_listener(&mut self, listener: Box<dyn Fn() -> ()>) -> usize;
    fn remove_children_listener(&mut self, listener: usize) -> bool;
}

pub trait GetConfigurable {
    fn get_configurable(&self) -> AbstractConfigurationObject;
}

pub struct AbstractConfigurationObject {
    data: Box<dyn Configurable>,
}

impl<T: TypeMapping<V, D>, V, D: GetConfigurable> Clone for ConfigurationObject<T, V, D> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            type_data: self.type_data.clone(),
        }
    }
}

impl<T: TypeMapping<V, D> + 'static, V: 'static, D: GetConfigurable + 'static> GetConfigurable
    for ConfigurationObject<T, V, D>
{
    fn get_configurable(&self) -> AbstractConfigurationObject {
        AbstractConfigurationObject {
            data: Box::new(self.clone()),
        }
    }
}

impl<T: TypeMapping<V, D>, V, D: GetConfigurable> Configurable for ConfigurationObject<T, V, D> {
    fn get_value(&self) -> JsValue {
        todo!()
    }

    fn set_value(&mut self, value: JsValue) -> () {
        todo!()
    }

    fn add_value_listener(&mut self, listener: Box<dyn Fn() -> ()>) -> usize {
        todo!()
    }

    fn remove_value_listener(&mut self, listener: usize) -> bool {
        todo!()
    }

    fn get_children(&self) -> Vec<AbstractConfigurationObject> {
        todo!()
    }

    fn insert_child(&mut self, index: usize) -> Option<AbstractConfigurationObject> {
        todo!()
    }

    fn remove_child(&mut self, index: usize) -> bool {
        todo!()
    }

    fn add_children_listener(&mut self, listener: Box<dyn Fn() -> ()>) -> usize {
        todo!()
    }

    fn remove_children_listener(&mut self, listener: usize) -> bool {
        todo!()
    }
}

// pub struct Test<D> {
//     val: D,
// }
// #[wasm_bindgen]
// impl<D> Test<D> {
//     fn do_smth(&self) {}
// }

/**
 * Some base case we can use if we want a configuration object without children
 */
impl GetConfigurable for () {
    fn get_configurable(&self) -> AbstractConfigurationObject {
        AbstractConfigurationObject {
            data: Box::new(self.clone()),
        }
    }
}
impl Configurable for () {
    fn get_value(&self) -> JsValue {
        JsValue::null()
    }

    fn set_value(&mut self, value: JsValue) -> () {}

    fn add_value_listener(&mut self, listener: Box<dyn Fn() -> ()>) -> usize {
        0
    }

    fn remove_value_listener(&mut self, listener: usize) -> bool {
        false
    }

    fn get_children(&self) -> Vec<AbstractConfigurationObject> {
        Vec::new()
    }

    fn insert_child(&mut self, index: usize) -> Option<AbstractConfigurationObject> {
        None
    }

    fn remove_child(&mut self, index: usize) -> bool {
        false
    }

    fn add_children_listener(&mut self, listener: Box<dyn Fn() -> ()>) -> usize {
        0
    }

    fn remove_children_listener(&mut self, listener: usize) -> bool {
        false
    }
}
