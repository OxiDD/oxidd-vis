use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use itertools::Itertools;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::util::free_id_manager::FreeIdManager;
use crate::util::rc_refcell::MutRcRefCell;
use crate::util::rc_refcell::RcRefCell;

use super::configuration_object_types::ConfigurationObjectType;
use super::mutator::{Mutator, MutatorCallbacks, Return};

/// An object that can be used to synchronize configuration data between Rust and TS
pub struct ConfigurationObject<T: ValueMapping<V>, V> {
    inner: MutRcRefCell<ConfigurationObjectData<V>>,
    type_data: PhantomData<T>,
}

pub struct ConfigurationObjectData<V> {
    id: Uuid,
    value: V,
    dirty_listener_order: Vec<usize>,
    dirty_listeners: HashMap<usize, Rc<RefCell<dyn FnMut() -> ()>>>,
    dirty_listener_ids: FreeIdManager<usize>,
    change_listener_order: Vec<usize>,
    change_listeners: HashMap<usize, Rc<RefCell<dyn FnMut() -> ()>>>,
    change_listener_ids: FreeIdManager<usize>,
}

pub trait ValueMapping<V> {
    fn to_js_value(val: &V) -> JsValue;
    fn get_children(val: &V) -> Option<Vec<AbstractConfigurationObject>>;
    fn from_js_value(js_val: JsValue, cur_val: &V) -> Option<V>;
}

pub trait Configurable {
    fn get_id(&self) -> Uuid;
    fn get_value(&self) -> JsValue;
    fn get_children(&self) -> Vec<AbstractConfigurationObject>;
    fn set_value(&mut self, value: JsValue) -> Mutator<(), ()>;
    /// Adds a listener that gets called when a value becomes outdated
    fn add_dirty_listener(&mut self, listener: Rc<RefCell<dyn FnMut() -> ()>>) -> usize;
    fn remove_dirty_listener(&mut self, listener: usize) -> bool;
    /// Adds a listener that gets called when a value's change happens (after potentially multiple values are already marked dirty)
    fn add_change_listener(&mut self, listener: Rc<RefCell<dyn FnMut() -> ()>>) -> usize;
    fn remove_change_listener(&mut self, listener: usize) -> bool;
    /// Make sure the object can be disposed, by removing listeners and therefor any likely reference loops
    fn dispose(&mut self) -> ();
}

#[wasm_bindgen]
pub struct AbstractConfigurationObject {
    object_type: ConfigurationObjectType,
    data: Rc<RefCell<dyn Configurable>>,
}
impl Clone for AbstractConfigurationObject {
    fn clone(&self) -> Self {
        Self {
            object_type: self.object_type.clone(),
            data: self.data.clone(),
        }
    }
}
impl AbstractConfigurationObject {
    pub fn new<C: Configurable + 'static>(
        object_type: ConfigurationObjectType,
        configurable: C,
    ) -> AbstractConfigurationObject {
        AbstractConfigurationObject {
            object_type,
            data: Rc::new(RefCell::new(configurable)),
        }
    }
}
pub trait Abstractable {
    fn get_abstract(&self) -> AbstractConfigurationObject;
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
                id: Uuid::new_v4(),
                value: init,
                change_listener_order: Vec::new(),
                change_listener_ids: FreeIdManager::new(0),
                change_listeners: HashMap::new(),
                dirty_listener_order: Vec::new(),
                dirty_listener_ids: FreeIdManager::new(0),
                dirty_listeners: HashMap::new(),
            }),
        }
    }
}
impl<T: ValueMapping<V> + 'static, V: 'static> ConfigurationObject<T, V> {
    pub fn with_value<O, F: Fn(&V) -> O>(&self, func: F) -> O {
        func(&self.inner.read().value)
    }
    pub fn set_value<F: FnOnce(&V) -> Option<V> + 'static>(
        &mut self,
        get_value: F,
    ) -> Mutator<(), ()> {
        let inner = self.inner.clone();
        let inner2 = self.inner.clone();
        let changed = MutRcRefCell::new(false);
        let changed2 = changed.clone();
        Mutator::new(
            move || {
                let mut inner = inner.get();
                let Some(val) = get_value(&inner.value) else {
                    return Return::new(());
                };
                inner.value = val;

                let listeners = inner
                    .dirty_listener_order
                    .iter()
                    .filter_map(|id| inner.dirty_listeners.get(id))
                    .cloned()
                    .collect_vec();
                drop(inner);
                for listener in listeners {
                    (listener.borrow_mut())();
                }
                *changed.get() = true;
                Return::new(())
            },
            move |_| {
                if !*changed2.read() {
                    return;
                }

                let inner = inner2.read();
                let listeners = inner
                    .change_listener_order
                    .iter()
                    .filter_map(|id| inner.change_listeners.get(id))
                    .cloned()
                    .collect_vec();
                drop(inner);
                for listener in listeners {
                    (listener.borrow_mut())();
                }
            },
        )
    }
}
impl<T: ValueMapping<V> + 'static, V: 'static> Configurable for ConfigurationObject<T, V> {
    fn get_id(&self) -> Uuid {
        self.inner.read().id
    }

    fn get_value(&self) -> JsValue {
        self.with_value(T::to_js_value)
    }

    fn set_value(&mut self, value: JsValue) -> Mutator<(), ()> {
        self.set_value(|cur| T::from_js_value(value, cur))
    }

    fn add_dirty_listener(&mut self, listener: Rc<RefCell<dyn FnMut() -> ()>>) -> usize {
        let mut inner = self.inner.get();
        let id = inner.dirty_listener_ids.get_next();
        inner.dirty_listeners.insert(id, listener);
        inner.dirty_listener_order.push(id);
        id
    }

    fn remove_dirty_listener(&mut self, listener: usize) -> bool {
        let mut inner = self.inner.get();
        let val = inner.dirty_listeners.remove(&listener);
        if val.is_some() {
            inner.dirty_listener_ids.make_available(listener);
            inner.dirty_listener_order.retain(|c_id| c_id != &listener);
            true
        } else {
            false
        }
    }

    fn add_change_listener(&mut self, listener: Rc<RefCell<dyn FnMut() -> ()>>) -> usize {
        let mut inner = self.inner.get();
        let id = inner.change_listener_ids.get_next();
        inner.change_listeners.insert(id, listener);
        inner.change_listener_order.push(id);
        id
    }

    fn remove_change_listener(&mut self, listener: usize) -> bool {
        let mut inner = self.inner.get();
        let val = inner.change_listeners.remove(&listener);
        if val.is_some() {
            inner.change_listener_ids.make_available(listener);
            inner.change_listener_order.retain(|c_id| c_id != &listener);
            true
        } else {
            false
        }
    }

    fn get_children(&self) -> Vec<AbstractConfigurationObject> {
        let val = &self.inner.read().value;
        T::get_children(val).unwrap_or_else(|| Vec::new())
    }

    fn dispose(&mut self) -> () {
        let mut inner = self.inner.get();
        inner.dirty_listeners.clear();
        inner.dirty_listener_ids = FreeIdManager::new(0);
        inner.change_listeners.clear();
        inner.change_listener_ids = FreeIdManager::new(0);
        let Some(children) = T::get_children(&inner.value) else {
            return;
        };
        for mut child in children {
            child.dispose()
        }
    }
}

#[wasm_bindgen]
impl AbstractConfigurationObject {
    pub fn get_id(&self) -> String {
        self.data.borrow().get_id().to_string()
    }
    pub fn get_type(&self) -> ConfigurationObjectType {
        self.object_type.clone()
    }
    pub fn get_value(&self) -> JsValue {
        self.data.borrow_mut().get_value()
    }

    pub fn set_value(&mut self, value: JsValue) -> MutatorCallbacks {
        MutatorCallbacks::new(self.data.borrow_mut().set_value(value))
    }

    pub fn add_js_dirty_listener(&mut self, listener: js_sys::Function) -> usize {
        let listener = move || {
            let this = JsValue::null();
            listener.call0(&this);
        };
        self.data
            .borrow_mut()
            .add_dirty_listener(Rc::new(RefCell::new(listener)))
    }

    pub fn remove_dirty_listener(&mut self, listener: usize) -> bool {
        self.data.borrow_mut().remove_dirty_listener(listener)
    }

    pub fn add_js_change_listener(&mut self, listener: js_sys::Function) -> usize {
        let listener = move || {
            let this = JsValue::null();
            listener.call0(&this);
        };
        self.data
            .borrow_mut()
            .add_change_listener(Rc::new(RefCell::new(listener)))
    }

    pub fn remove_change_listener(&mut self, listener: usize) -> bool {
        self.data.borrow_mut().remove_change_listener(listener)
    }

    pub fn get_children(&self) -> Vec<AbstractConfigurationObject> {
        self.data.borrow().get_children()
    }
    pub fn dispose(&mut self) -> () {
        self.data.borrow_mut().dispose()
    }
}
impl AbstractConfigurationObject {
    pub fn add_dirty_listener<F: FnMut() -> () + 'static>(&mut self, listener: F) -> usize {
        self.data
            .borrow_mut()
            .add_dirty_listener(Rc::new(RefCell::new(listener)))
    }
    pub fn add_change_listener<F: FnMut() -> () + 'static>(&mut self, listener: F) -> usize {
        self.data
            .borrow_mut()
            .add_change_listener(Rc::new(RefCell::new(listener)))
    }
}
