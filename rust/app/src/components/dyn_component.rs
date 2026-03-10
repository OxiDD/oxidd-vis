use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    impl_watchable, make_typed_dyn_watchable,
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        Constant, ControlledField, Derived, Field, IntoWatchable, ReadonlyField, Watchable,
        WatchableUtils,
    },
};

make_typed_dyn_watchable!(ComponentWatchable, Component);

#[derive(Clone)]
#[wasm_bindgen]
pub struct DynComp(ComponentWatchable);
impl DynComp {
    pub fn new(val: impl IntoWatchable<Component> + 'static) -> Self {
        DynComp(ComponentWatchable::new(val))
    }
}
impl DynComp {
    fn watchable(&self) -> &ComponentWatchable {
        &self.0
    }
    pub fn into_component(self) -> Component {
        Component::new(ComponentOption::Dyn(self))
    }
}
impl_watchable!(DynComp, Component);
impl<C: Into<Component>> From<C> for DynComp {
    fn from(value: C) -> Self {
        DynComp::new(value.into())
    }
}

macro_rules! some_into_component {
    ($(< $($Generics:ident),* >)?, $ValueType:tt, $Map:expr) => {
        impl$(<$($Generics: Into<Component> + Clone + 'static),*>)? Into<Component> for $ValueType {
            fn into(self) -> Component {
                DynComp::new(self).into_component()
            }
        }
    }
}

some_into_component!(<>, (Constant<Component>), |me: Constant<Component>| me.map(|value| (*value).clone().into()));
some_into_component!(<>, (Derived<Component>), |me: Derived<Component>| me.map(|value| (*value).clone().into()));
some_into_component!(<>, (Field<Component>), |me: Field<Component>| me.map(|value| (*value).clone().into()));
some_into_component!(<>, (ReadonlyField<Component>), |me: ReadonlyField<Component>| me.map(|value| (*value).clone().into()));
some_into_component!(<>, (ControlledField<Component>), |me: ControlledField<Component>| me.map(|value| (*value).clone().into()));
