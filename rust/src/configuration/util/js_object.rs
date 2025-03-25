use js_sys::{Object, Reflect};
use wasm_bindgen::JsValue;

pub struct JsObject {
    object: Object,
}

impl JsObject {
    pub fn new() -> JsObject {
        JsObject {
            object: Object::new(),
        }
    }
    pub fn load(val: JsValue) -> JsObject {
        JsObject { object: val.into() }
    }

    pub fn set<K: Into<JsValue>>(self, field: &str, value: K) -> Self {
        Reflect::set(&self.object, &field.into(), &value.into());
        self
    }

    pub fn get(&self, field: &str) -> Option<JsValue> {
        Reflect::get(&self.object, &field.into()).ok()
    }
}

impl Into<JsValue> for JsObject {
    fn into(self) -> JsValue {
        self.object.into()
    }
}
