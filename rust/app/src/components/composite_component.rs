use app_macros::{gen_tuple_into_component_vec_watchables, wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::dyn_component::DynComp,
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        make_typed_dyn_watchable, BoolWatchable, Constant, ControlledField, Derived, F32Watchable,
        Field, ReadonlyField, WatchableUtils,
    },
};

make_typed_dyn_watchable!(ComponentVecWatchable, Vec<Component>);

#[wasm_bindgen]
#[derive(Clone)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}
make_typed_dyn_watchable!(AlignWatchable, Align);
#[wasm_bindgen]
#[derive(Clone)]
pub enum AlignMain {
    Start,
    Center,
    End,
    Stretch,
    SpaceAround,
    SpaceBetween,
}
make_typed_dyn_watchable!(AlignMainWatchable, AlignMain);

/// Composite component
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct CompositeComp {
    /// The children of this component
    #[getter]
    #[builder(finish_fn, into)]
    children: ComponentVecWatchable,
    /// Whether the variants should display horizontally (if advanced)
    #[getter]
    #[setter(bool, false)]
    horizontal: BoolWatchable,
    /// Whether to fill the parent container
    #[getter]
    #[setter(bool, false)]
    fill: BoolWatchable,
    /// The main axis alignment
    #[getter]
    #[setter(AlignMain, AlignMain::Start)]
    main_align: AlignMainWatchable,
    /// The off axis alignment
    #[getter]
    #[setter(Align, Align::Start)]
    perpendicular_align: AlignWatchable,
}
impl CompositeComp {
    pub fn new(children: impl Into<ComponentVecWatchable>) -> Self {
        Self::builder().build(children)
    }
}
impl Into<Component> for CompositeComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::Composite(self))
    }
}

#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct CompositeItemComp {
    /// The children of this component
    #[getter]
    #[builder(finish_fn, into)]
    child: DynComp,
    /// The off axis alignment
    #[getter]
    #[setter(Align, Align::Start)]
    perpendicular_align: AlignWatchable,
    /// How much to shrink compared to siblings, if stretching on main-axis
    #[getter]
    #[setter(f32, 0.0)]
    shrink_ratio: F32Watchable,
    /// How much to grow compared to siblings, if stretching on main-axis
    #[getter]
    #[setter(f32, 0.0)]
    grow_ratio: F32Watchable,
}
impl Into<Component> for CompositeItemComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::CompositeItem(self))
    }
}

/*
   Traits to automatically derive ComponentVecWatchables implicitly without boilerplate
   Any Vec or tuple of Into<Component> can be used, as well as derived watchables or watchable fields of this type.
*/
trait IntoComponentVec {
    fn into_vec(self) -> Vec<Component>;
}
impl<T: Into<Component>> IntoComponentVec for Vec<T> {
    fn into_vec(self) -> Vec<Component> {
        self.into_iter().map(|v| v.into()).collect()
    }
}
impl IntoComponentVec for Component {
    fn into_vec(self) -> Vec<Component> {
        vec![self]
    }
}

macro_rules! some_into_component_vec_watchable {
    ($(< $($Generics:ident),* >)?, $ValueType:tt, $Map:expr) => {
        impl$(<$($Generics: Into<Component> + Clone + 'static),*>)? Into<ComponentVecWatchable> for $ValueType {
            fn into(self) -> ComponentVecWatchable {
                ComponentVecWatchable::new(($Map)(self))
            }
        }
        impl$(<$($Generics: Into<Component> + Clone + 'static),*>)? Into<Component> for $ValueType {
            fn into(self) -> Component {
                let watchable = Into::<ComponentVecWatchable>::into(self);
                Into::<Component>::into(CompositeComp::new(watchable))
            }
        }
    };
}

macro_rules! into_component_vec_watchable {
    ($(<$($Generics:ident),* >)?, $ValueType:tt) => {
        some_into_component_vec_watchable!($(<$($Generics),*>)?, $ValueType, |me: $ValueType| Constant::new(
            me.into_vec()
        ));

        some_into_component_vec_watchable!($(<$($Generics),*>)?, (Constant<$ValueType>), |me: Constant<$ValueType>| me
            .map(|values| (*values).clone().into_vec()));
        some_into_component_vec_watchable!($(<$($Generics),*>)?, (Derived<$ValueType>), |me: Derived<$ValueType>| me
            .map(|values| (*values).clone().into_vec()));
        some_into_component_vec_watchable!($(<$($Generics),*>)?, (Field<$ValueType>), |me: Field<$ValueType>| me
            .map(|values| (*values).clone().into_vec()));
        some_into_component_vec_watchable!($(<$($Generics),*>)?, (ReadonlyField<$ValueType>), |me: ReadonlyField<
            $ValueType,
        >| me
            .map(|values| (*values).clone().into_vec()));
        some_into_component_vec_watchable!(
            $(<$($Generics),*>)?,
            (ControlledField<$ValueType>),
            |me: ControlledField<$ValueType>| me.map(|values| (*values).clone().into_vec())
        );
    };
}
into_component_vec_watchable!(<T>, (Vec<T>));
gen_tuple_into_component_vec_watchables!();

#[doc(hidden)]
fn test() {
    comp(
        CompositeComp::builder()
            .horizontal(true)
            .build(((), (), ((), ()))),
    );
}
#[doc(hidden)]
fn comp(v: impl Into<Component>) {}
