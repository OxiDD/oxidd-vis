use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
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
    open_side: OpenSide,
    open_relative_visualization: bool,
    open_ratio: f32,
    name: String,
    auto_open: AutoOpen,
    // A panel ID, to allow for persistent layout state storage
    id: String,
    // A name for the category of panel, such that other panels of the same category appear in the same location
    panel_category: String,
}

#[derive(Clone, Copy)]
pub enum AutoOpen {
    Never,
    Always,
    IfExistingPanel, // Automatically open if the target panel is already present, and this only creates a new tab
}

pub struct PanelConfigBuilder<C: Abstractable + Clone + 'static> {
    button_style: ButtonStyle,
    name: Option<String>,
    // A name for the category of panel, such that other panels of the same category appear in the same location
    panel_category: Option<String>,
    open_side: OpenSide,
    open_ratio: f32,
    open_relative_visualization: bool,
    auto_open: AutoOpen,
    data: PhantomData<C>,
}
impl<C: Abstractable + Clone + 'static> PanelConfigBuilder<C> {
    pub fn new() -> Self {
        PanelConfigBuilder {
            button_style: ButtonStyle::Plain(),
            name: None,
            panel_category: None,
            open_side: OpenSide::Right,
            open_relative_visualization: false,
            open_ratio: 1.0,
            auto_open: AutoOpen::IfExistingPanel,
            data: PhantomData,
        }
    }

    /// Sets the text to show of the button (overriding any configured icon/icon-description)
    pub fn set_button_text(mut self, name: &str) -> Self {
        self.button_style = ButtonStyle::Text(name.into());
        self
    }
    /// Sets the icon to show of the button (overriding any configured button text)
    pub fn set_button_icon(mut self, icon: &str) -> Self {
        self.button_style = ButtonStyle::Icon {
            name: icon.into(),
            description: match self.button_style {
                ButtonStyle::Icon {
                    name: _,
                    description,
                } => description,
                _ => "".into(),
            },
        };
        self
    }
    /// Sets the icon-description to show of the button (overriding any configured button text)
    pub fn set_button_icon_description(mut self, icon_description: &str) -> Self {
        self.button_style = ButtonStyle::Icon {
            name: match self.button_style {
                ButtonStyle::Icon {
                    name,
                    description: _,
                } => name,
                _ => "".into(),
            },
            description: icon_description.into(),
        };
        self
    }
    /// Sets the name to show in the panel handle
    pub fn set_name(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Sets the category of the panel, to open it in a location with the same category
    pub fn set_category(mut self, category: &str) -> Self {
        self.panel_category = Some(category.into());
        self
    }
    /// Sets the side to which this panel should be opened the first time
    pub fn set_open_side(mut self, side: OpenSide) -> Self {
        self.open_side = side;
        self
    }
    /// Sets whether the panel should be positioned relative to the visualization, or relative to the view containing this button open
    pub fn set_open_relative_visualization(mut self, relative_visualization: bool) -> Self {
        self.open_relative_visualization = relative_visualization;
        self
    }
    /// Sets the relative size to open this panel at, in relation to the average size of other panels in this layer (defualts to 1)
    pub fn set_open_size(mut self, size: f32) -> Self {
        self.open_ratio = size;
        self
    }
    /// Sets whether or not this panel should automatically open
    pub fn set_auto_open(mut self, auto_open: AutoOpen) -> Self {
        self.auto_open = auto_open;
        self
    }
    // Create the panel-config using the given child config
    pub fn build(self, child: C) -> PanelConfig<C> {
        let buton_style = &self.button_style;
        let name = self.name.or_else(|| match buton_style {
            ButtonStyle::Icon {
                name,
                description: _,
            } => Some(name.clone()),
            ButtonStyle::Text(text) => Some(text.clone()),
            _ => None,
        });
        PanelConfig {
            data: ConfigurationObject::new(PanelValue {
                child: child.clone(),
                name: name.clone().unwrap_or_default(),
                panel_category: self
                    .panel_category
                    .or(name)
                    .unwrap_or_else(|| "visualization-settings".into()),
                auto_open: self.auto_open,
                button_style: self.button_style,
                open_relative_visualization: self.open_relative_visualization,
                open_side: self.open_side,
                open_ratio: self.open_ratio,
                id: Uuid::new_v4().into(),
            }),
            child,
        }
    }
}

#[derive(Clone, Copy)]
pub enum OpenSide {
    In,
    Above,
    Right,
    Below,
    Left,
}
impl<'a> Into<&'a str> for OpenSide {
    fn into(self) -> &'a str {
        match self {
            OpenSide::In => "in",
            OpenSide::Above => "north",
            OpenSide::Right => "east",
            OpenSide::Below => "south",
            OpenSide::Left => "west",
        }
    }
}

impl<C: Abstractable + Clone + 'static> PanelConfig<C> {
    pub fn builder() -> PanelConfigBuilder<C> {
        PanelConfigBuilder::new()
    }
    pub fn new(button_name: &str, name: &str, data: C) -> Self {
        Self::builder()
            .set_button_text(button_name)
            .set_name(name)
            .build(data)
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
        let obj = JsObject::new()
            .set("name", &val.name)
            .set("id", &val.id)
            .set("category", &val.panel_category)
            .set::<&str>("openSide", val.open_side.into())
            .set(
                "autoOpen",
                match val.auto_open {
                    AutoOpen::Never => 0,
                    AutoOpen::Always => 1,
                    AutoOpen::IfExistingPanel => 2,
                },
            )
            .set("openRelativeVis", val.open_relative_visualization)
            .set("openRatio", val.open_ratio);
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
