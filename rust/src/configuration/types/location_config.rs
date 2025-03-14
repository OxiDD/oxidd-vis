use std::ops::{Deref, DerefMut};

use wasm_bindgen::JsValue;

use crate::configuration::{
    configuration_object::{
        AbstractConfigurationObject, Abstractable, ConfigObjectGetter, ConfigurationObject,
        ValueMapping,
    },
    configuration_object_types::ConfigurationObjectType,
    util::js_object::JsObject,
};

///
/// A panel configuration component that allows a sub-configuration to be openable in a separate panel
#[derive(Clone)]
pub struct LocationConfig<C: Abstractable + Clone> {
    data: ConfigurationObject<LocationConfig<C>, LocationValue<C>>,
    child: C,
}

#[derive(Clone)]
struct LocationValue<C: Abstractable + Clone> {
    child: C,
    location: Location,
}

#[derive(Clone, Copy)]
pub struct Location {
    /// The x coordinate between 0 and 1
    pub x: f32,
    /// The y coordinate between 0 and 1
    pub y: f32,
    /// Padding to apply, as a fraction of the standard padding amount
    pub padding: f32,
}
impl Location {
    pub const TOP_LEFT: Location = Location {
        x: 0.,
        y: 0.,
        padding: 1.0,
    };
    pub const TOP_MIDDLE: Location = Location {
        x: 0.5,
        y: 0.,
        padding: 1.0,
    };
    pub const TOP_RIGHT: Location = Location {
        x: 1.,
        y: 0.,
        padding: 1.0,
    };
    pub const MIDDLE_LEFT: Location = Location {
        x: 0.,
        y: 0.5,
        padding: 1.0,
    };
    pub const MIDDLE: Location = Location {
        x: 0.5,
        y: 0.5,
        padding: 1.0,
    };
    pub const MIDDLE_RIGHT: Location = Location {
        x: 1.,
        y: 0.5,
        padding: 1.0,
    };
    pub const BOTTOM_LEFT: Location = Location {
        x: 0.,
        y: 1.,
        padding: 1.0,
    };
    pub const BOTTOM_MIDDLE: Location = Location {
        x: 0.5,
        y: 1.,
        padding: 1.0,
    };
    pub const BOTTOM_RIGHT: Location = Location {
        x: 1.,
        y: 1.,
        padding: 1.0,
    };
}

impl<C: Abstractable + Clone + 'static> LocationConfig<C> {
    pub fn new(location: Location, child: C) -> Self {
        LocationConfig {
            data: ConfigurationObject::new(LocationValue {
                child: child.clone(),
                location,
            }),
            child: child,
        }
    }
}

impl<C: Abstractable + Clone + 'static> Deref for LocationConfig<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.child
    }
}
impl<C: Abstractable + Clone + 'static> DerefMut for LocationConfig<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}

impl<C: Abstractable + Clone + 'static> Abstractable for LocationConfig<C> {
    fn get_abstract(&self) -> AbstractConfigurationObject {
        AbstractConfigurationObject::new(ConfigurationObjectType::Location, self.data.clone())
    }
}
impl<C: Abstractable + Clone + 'static> ConfigObjectGetter<LocationConfig<C>, LocationValue<C>>
    for LocationConfig<C>
{
    fn with_config_object<
        O,
        U: FnOnce(&mut ConfigurationObject<LocationConfig<C>, LocationValue<C>>) -> O,
    >(
        &mut self,
        e: U,
    ) -> O {
        e(&mut self.data)
    }
}

impl<C: Abstractable + Clone> ValueMapping<LocationValue<C>> for LocationConfig<C> {
    fn to_js_value(val: &LocationValue<C>) -> JsValue {
        let obj = JsObject::new()
            .set("horizontal", val.location.x)
            .set("vertical", val.location.y)
            .set("padding", val.location.padding);
        obj.into()
    }
    fn from_js_value(js_val: JsValue, cur: &LocationValue<C>) -> Option<LocationValue<C>> {
        None
    }

    fn get_children(val: &LocationValue<C>) -> Option<Vec<AbstractConfigurationObject>> {
        Some(vec![val.child.get_abstract()])
    }
}
