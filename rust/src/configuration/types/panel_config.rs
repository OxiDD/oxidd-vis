use std::ops::{Deref, DerefMut};
use uuid::Uuid;

use wasm_bindgen::JsValue;

use crate::configuration::{
    configuration_object::{
        AbstractConfigurationObject, Abstractable, ConfigObjectGetter, ConfigurationObject,
        ValueMapping,
    },
    configuration_object_types::ConfigurationObjectType,
    util::js_object::JsObject,
};

use super::button_config::ButtonStyle;

///
/// A panel configuration component that allows a sub-configuration to be openable in a separate panel
#[derive(Clone)]
pub struct PanelConfig<C: Abstractable + Clone> {
    data: ConfigurationObject<PanelConfig<C>, PanelValue<C>>,
    child: C,
}

#[derive(Clone)]
struct PanelValue<C: Abstractable + Clone> {
    child: C,
    button_style: ButtonStyle,
    name: String,
    // A panel ID, to allow for persistent layout state storage
    id: String,
}

impl<C: Abstractable + Clone + 'static> PanelConfig<C> {
    pub fn new(button: ButtonStyle, name: &str, data: C) -> Self {
        PanelConfig {
            data: ConfigurationObject::new(PanelValue {
                child: data.clone(),
                button_style: button,
                name: name.into(),
                id: Uuid::new_v4().into(),
            }),
            child: data,
        }
    }
}
impl<C: Abstractable + Clone + 'static> Deref for PanelConfig<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.child
    }
}
impl<C: Abstractable + Clone + 'static> DerefMut for PanelConfig<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}

impl<C: Abstractable + Clone + 'static> Abstractable for PanelConfig<C> {
    fn get_abstract(&self) -> AbstractConfigurationObject {
        AbstractConfigurationObject::new(ConfigurationObjectType::Panel, self.data.clone())
    }
}
impl<C: Abstractable + Clone + 'static> ConfigObjectGetter<PanelConfig<C>, PanelValue<C>>
    for PanelConfig<C>
{
    fn with_config_object<
        O,
        U: FnOnce(&mut ConfigurationObject<PanelConfig<C>, PanelValue<C>>) -> O,
    >(
        &mut self,
        e: U,
    ) -> O {
        e(&mut self.data)
    }
}

impl<C: Abstractable + Clone> ValueMapping<PanelValue<C>> for PanelConfig<C> {
    fn to_js_value(val: &PanelValue<C>) -> JsValue {
        let obj = JsObject::new().set("name", &val.name).set("id", &val.id);
        let obj = match &val.button_style {
            ButtonStyle::Text(text) => obj.set("text", text),
            ButtonStyle::Icon { name, description } => {
                obj.set("icon", name).set("text", description)
            }
            _ => obj,
        };
        obj.into()
    }
    fn from_js_value(js_val: JsValue, cur: &PanelValue<C>) -> Option<PanelValue<C>> {
        let obj = JsObject::load(js_val);
        let id = obj
            .get("id")
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| cur.id.clone());
        Some(PanelValue { id, ..cur.clone() })
    }

    fn get_children(val: &PanelValue<C>) -> Option<Vec<AbstractConfigurationObject>> {
        Some(vec![val.child.get_abstract()])
    }
}
