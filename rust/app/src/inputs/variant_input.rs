use std::{cell::RefCell, ops::Index, rc::Rc};

use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use itertools::Itertools;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::{
        composite_component::ComponentVecWatchable, Align, AlignMain, AlignMainWatchable,
        AlignWatchable,
    },
    inputs::variant_input::variant_input_comp_builder::{Empty, SetData},
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        make_typed_dyn_watchable, signaller::Signaller, BoolWatchable, Constant, Derived,
        DynWatchable, F32Watchable, Field, IntoWatchable, Mutator, OptionBoolWatchable,
        U32Watchable, Watchable, Watching,
    },
};

/// A select field
pub struct VariantInput<V: VariantOptions>(Field<V>);
pub trait VariantOptions: Sized + Clone + Eq {
    /// The first value is considered the default, and used as a fallback
    fn get_variants() -> Vec<Self>;
}
impl<V: VariantOptions> Clone for VariantInput<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<V: VariantOptions + 'static> VariantInput<V> {
    pub fn new(val: V) -> Self {
        Self(Field::new(val))
    }
    pub fn from(from: impl Into<V>) -> Self {
        Self::new(from.into())
    }
    fn watchable(&self) -> &Field<V> {
        &self.0
    }
    pub fn set(&mut self, val: V) -> Signaller {
        self.0.set(val)
    }
    pub fn filtered(
        &self,
        options: impl IntoWatchable<Vec<V>> + 'static,
    ) -> VariantInputFiltered<V> {
        VariantInputFiltered::raw(self.0.clone(), options)
    }
}
impl<V: VariantOptions + 'static> Into<VariantInputFiltered<V>> for VariantInput<V> {
    fn into(self) -> VariantInputFiltered<V> {
        self.filtered(Constant::new(V::get_variants()))
    }
}

/// A select field with filtered options
pub struct VariantInputFiltered<V: Sized + Clone + Eq + 'static> {
    value: Field<V>,
    options: DynWatchable<Vec<V>>,
    filtered: Derived<V>,
}
impl<V: Sized + Clone + Eq> VariantInputFiltered<V> {
    fn raw(
        value: Field<V>,
        options: impl IntoWatchable<Vec<V>> + 'static,
    ) -> VariantInputFiltered<V> {
        let value_clone = value.clone();
        let options = DynWatchable::new(options.into_watchable());
        let options_clone = options.clone();
        VariantInputFiltered {
            value,
            options,
            filtered: Derived::new(move |t| {
                let selected = &*value_clone.watch(t);
                let options = options_clone.watch(t);
                let valid = options.contains(&selected);
                if valid {
                    selected.clone()
                } else {
                    if let Some(def) = options.first() {
                        def.clone()
                    } else {
                        selected.clone()
                    }
                }
            }),
        }
    }
    pub fn new(value: V, options: impl IntoWatchable<Vec<V>> + 'static) -> VariantInputFiltered<V> {
        Self::raw(Field::new(value), options)
    }
    pub fn from(
        from: impl Into<V>,
        options: impl IntoWatchable<Vec<V>> + 'static,
    ) -> VariantInputFiltered<V> {
        Self::new(from.into(), options)
    }
    pub fn set(&mut self, val: V) -> Signaller {
        self.value.set(val)
    }
    pub fn comp(&self, map: impl Fn(&V) -> Component + 'static) -> VariantComponentMapper<V> {
        VariantComponentMapper::new(self.clone(), map)
    }
}
impl<V: Sized + Clone + Eq> Clone for VariantInputFiltered<V> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            options: self.options.clone(),
            filtered: self.filtered.clone(),
        }
    }
}

/// Maps variants to the components representing them
pub struct VariantComponentMapper<V: Sized + Clone + Eq + 'static> {
    data: VariantInputFiltered<V>,
    option_components: ComponentVecWatchable,
    selected_index: U32Watchable,
}
impl<V: Sized + Clone + Eq + 'static> VariantComponentMapper<V> {
    pub fn new(
        data: impl Into<VariantInputFiltered<V>>,
        map: impl Fn(&V) -> Component + 'static,
    ) -> Self {
        let data = data.into();
        let options = data.options.clone();
        let option_components = ComponentVecWatchable::new(Derived::new(move |t| {
            let options = options.watch(t);
            (*options).iter().map(&map).collect::<Vec<_>>()
        }));
        let options = data.options.clone();
        let selected = data.value.read();
        let selected_index = U32Watchable::new(Derived::new(move |t| {
            let options = options.watch(t);
            let val = selected.watch(t);
            let index = (*options)
                .iter()
                .position(|v| v == &*val)
                .unwrap_or_default();
            index as u32
        }));
        Self {
            data,
            option_components,
            selected_index,
        }
    }
    pub fn from(
        data: impl Into<VariantInputFiltered<V>>,
        map: impl Fn(&V) -> Component + 'static,
    ) -> Self {
        Self::new(data.into(), map)
    }
}
pub trait VariantComponentMapping {
    fn map(&self) -> Component;
}
impl<V: Sized + Clone + Eq + VariantComponentMapping + VariantOptions>
    Into<VariantComponentMapper<V>> for VariantInput<V>
{
    fn into(self) -> VariantComponentMapper<V> {
        VariantComponentMapper::new(self, |v| V::map(v))
    }
}

impl<V: Sized + Clone + Eq + VariantComponentMapping> Into<VariantComponentMapper<V>>
    for VariantInputFiltered<V>
{
    fn into(self) -> VariantComponentMapper<V> {
        VariantComponentMapper::new(self, |v| V::map(v))
    }
}

impl<V: Sized + Clone + Eq + 'static> VariantOptionComponents for VariantComponentMapper<V> {
    fn get_options(&self) -> &ComponentVecWatchable {
        &self.option_components
    }

    fn select_option(&mut self, option: u32) -> Option<Signaller> {
        let options = self.data.options.get();
        let option = (*options).get(option as usize);
        option.map(|option| self.data.set(option.clone()))
    }

    fn get_option(&self) -> &U32Watchable {
        &self.selected_index
    }
}
trait VariantOptionComponents {
    fn get_options(&self) -> &ComponentVecWatchable;
    fn select_option(&mut self, option: u32) -> Option<Signaller>;
    fn get_option(&self) -> &U32Watchable;
}

/// Variant component.
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct VariantInputComp {
    /// The data of the component.
    #[builder(setters(name=set_data, vis=""))]
    data: Rc<RefCell<dyn VariantOptionComponents>>,
    /// Whether the variants should display horizontally (if advanced).
    #[getter]
    #[setter(Option<bool>)]
    horizontal: OptionBoolWatchable,
    /// The main axis alignment (if advanced).
    #[getter]
    #[setter(AlignMain, AlignMain::Start)]
    main_align: AlignMainWatchable,
    /// The off axis alignment (if advanced).
    #[getter]
    #[setter(Align, Align::Start)]
    perpendicular_align: AlignWatchable,
    /// The gap between child elements (if advanced).
    #[getter]
    #[setter(f32, 1.0)]
    gap: F32Watchable,
    /// Whether this input is disabled
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable,
    /// The current option components
    #[getter]
    #[builder(skip=VariantInputComp::get_options(data.clone()))]
    options: ComponentVecWatchable,
    /// The currently selected option index
    #[getter]
    #[builder(skip=VariantInputComp::get_selected(data.clone()))]
    selected: U32Watchable,
}
impl VariantInputCompBuilder {
    pub fn data<V: Sized + Clone + Eq + 'static>(
        self,
        val: impl Into<VariantComponentMapper<V>>,
    ) -> VariantInputCompBuilder<SetData<Empty>> {
        self.set_data(Rc::new(RefCell::new(val.into())))
    }
}
#[wasm_bindgen]
impl VariantInputComp {
    pub fn select(&mut self, option: u32) -> Mutator {
        let value = self.data.clone();
        Mutator::exec(move || Box::new(value.borrow_mut().select_option(option)))
    }
    fn get_selected(data: Rc<RefCell<dyn VariantOptionComponents>>) -> U32Watchable {
        (*data).borrow().get_option().clone()
    }
    fn get_options(data: Rc<RefCell<dyn VariantOptionComponents>>) -> ComponentVecWatchable {
        (*data).borrow().get_options().clone()
    }
}

impl<V: Sized + Clone + Eq + 'static> Into<VariantInputComp> for VariantComponentMapper<V> {
    fn into(self) -> VariantInputComp {
        VariantInputComp::builder().data(self).build()
    }
}
impl<V: Sized + Clone + Eq + VariantComponentMapping + VariantOptions + 'static>
    Into<VariantInputComp> for VariantInput<V>
{
    fn into(self) -> VariantInputComp {
        Into::<VariantComponentMapper<V>>::into(self).into()
    }
}
impl<V: Sized + Clone + Eq + VariantComponentMapping> Into<VariantInputComp>
    for VariantInputFiltered<V>
{
    fn into(self) -> VariantInputComp {
        Into::<VariantComponentMapper<V>>::into(self).into()
    }
}

impl<V: Sized + Clone + Eq + VariantComponentMapping + VariantOptions + 'static> Into<Component>
    for VariantInput<V>
{
    fn into(self) -> Component {
        Into::<VariantInputComp>::into(self).into()
    }
}
impl<V: Sized + Clone + Eq + VariantComponentMapping> Into<Component> for VariantInputFiltered<V> {
    fn into(self) -> Component {
        Into::<VariantInputComp>::into(self).into()
    }
}
impl<V: Sized + Clone + Eq + VariantComponentMapping> Into<Component>
    for VariantComponentMapper<V>
{
    fn into(self) -> Component {
        Into::<VariantInputComp>::into(self).into()
    }
}
impl Into<Component> for VariantInputComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::VariantInput(self))
    }
}
