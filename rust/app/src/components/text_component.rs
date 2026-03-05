use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        BoolWatchable, Constant, ControlledField, Derived, Field, IntoWatchable, ReadonlyField,
        StringWatchable, WatchableUtils,
    },
};

/// Component that renders text, either as paragraph content or as a title.
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct TextComp {
    /// The textual content to display.
    #[getter]
    #[setter(String)]
    text: StringWatchable,
    /// Whether this text is rendered as a title or normal paragraph.
    #[getter]
    #[setter(bool, false)]
    is_title: BoolWatchable,
}

impl TextComp {
    pub fn new(text: impl IntoWatchable<String> + 'static) -> Self {
        TextComp::builder().text(text).build()
    }
}
impl Into<Component> for TextComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::Text(self))
    }
}

impl<W: IntoWatchable<String> + 'static> From<W> for TextComp {
    fn from(watchable: W) -> Self {
        TextComp::builder()
            .text(StringWatchable::new(watchable))
            .build()
    }
}

macro_rules! some_into_component {
    ($(< $($Generics:ident),* >)?, $ValueType:tt, $Map:expr) => {
        impl$(<$($Generics: IntoString + Clone + 'static),*>)? Into<Component> for $ValueType {
            fn into(self) -> Component {
                TextComp::new(($Map)(self)).into()
            }
        }
    }
}

trait IntoString {
    fn into_string(self) -> String;
}
impl IntoString for String {
    fn into_string(self) -> String {
        self
    }
}
impl IntoString for &str {
    fn into_string(self) -> String {
        self.to_string()
    }
}
some_into_component!(<>, (String), |me: String| Constant::new(me.into_string()));
some_into_component!(<>, (&str), |me: &str| Constant::new(me.into_string()));
some_into_component!(<T>, (Constant<T>), |me: Constant<T>| me.map(|value| (*value).clone().into_string()));
some_into_component!(<T>, (Derived<T>), |me: Derived<T>| me.map(|value| (*value).clone().into_string()));
some_into_component!(<T>, (Field<T>), |me: Field<T>| me.map(|value| (*value).clone().into_string()));
some_into_component!(<T>, (ReadonlyField<T>), |me: ReadonlyField<T>| me.map(|value| (*value).clone().into_string()));
some_into_component!(<T>, (ControlledField<T>), |me: ControlledField<T>| me.map(|value| (*value).clone().into_string()));
