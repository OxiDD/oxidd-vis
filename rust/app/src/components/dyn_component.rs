use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        impl_watchable, make_typed_dyn_watchable, Constant, ControlledField, Derived, Field,
        IntoWatchable, ReadonlyField, Watchable, WatchableUtils,
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
}
impl_watchable!(DynComp, Component);

impl Into<Component> for DynComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::Dyn(self))
    }
}

macro_rules! some_into_dyn_component {
    ($(< $($Generics:ident),* >)?, $ValueType:tt, $Map:expr) => {
        impl$(<$($Generics: Into<Component> + Clone + 'static),*>)? Into<DynComp> for $ValueType {
            fn into(self) -> DynComp {
                DynComp::new(($Map)(self))
            }
        }
    };
}

macro_rules! some_into_component {
    ($(< $($Generics:ident),* >)?, $ValueType:tt, $Map:expr) => {
        impl$(<$($Generics: Into<Component> + Clone + 'static),*>)? Into<Component> for $ValueType {
            fn into(self) -> Component {
                let watchable = Into::<DynComp>::into(self);
                Into::<Component>::into(DynComp::new(watchable))
            }
        }
    }
}

some_into_dyn_component!(<T>, (Constant<T>), |me: Constant<T>| me.map(|value| (*value).clone().into()));
some_into_component!(<>, (Constant<Component>), |me: Constant<Component>| me.map(|value| (*value).clone().into()));
some_into_dyn_component!(<T>, (Derived<T>), |me: Derived<T>| me.map(|value| (*value).clone().into()));
some_into_component!(<>, (Derived<Component>), |me: Derived<Component>| me.map(|value| (*value).clone().into()));
some_into_dyn_component!(<T>, (Field<T>), |me: Field<T>| me.map(|value| (*value).clone().into()));
some_into_component!(<>, (Field<Component>), |me: Field<Component>| me.map(|value| (*value).clone().into()));
some_into_dyn_component!(<T>, (ReadonlyField<T>), |me: ReadonlyField<T>| me.map(|value| (*value).clone().into()));
some_into_component!(<>, (ReadonlyField<Component>), |me: ReadonlyField<Component>| me.map(|value| (*value).clone().into()));
some_into_dyn_component!(<T>, (ControlledField<T>), |me: ControlledField<T>| me.map(|value| (*value).clone().into()));
some_into_component!(<>, (ControlledField<Component>), |me: ControlledField<Component>| me.map(|value| (*value).clone().into()));
