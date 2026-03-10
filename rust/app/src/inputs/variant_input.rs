use std::{cell::RefCell, ops::Index, rc::Rc};

use app_macros::{builder_into_comp, wasm_getters, watchable_setters};
use bon::Builder;
use itertools::Itertools;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::{
        composite_component::ComponentVecWatchable, Align, AlignMain, AlignMainWatchable,
        AlignWatchable,
    },
    impl_setter, impl_watchable,
    inputs::{
        variant_input::variant_input_comp_builder::{SetOptions, SetWrapper},
        wrapper::{CompWrapper, IdentityWrapper, InputWrapper},
        InheritLabel, Inheritable, InheritedInput,
    },
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        signaller::Signaller, BoolWatchable, ClonableWatchableUtils, Constant, DataState, Derived,
        DynSignaller, DynWatchable, DynWatchableSetter, F32Watchable, Field, IntoWatchable,
        IntoWatchableSetter, Listener, Mutator, Observer, OptionBoolWatchable, Setter,
        U32Watchable, Watchable, WatchableState, Watching,
    },
};

/// A variant input
pub struct VariantInput<V: Sized + Clone + Eq + 'static> {
    value: DynWatchableSetter<V>,
    options: DynWatchable<Vec<V>>,
    filtered: Derived<V>,
}
impl<V: Sized + Clone + Eq> VariantInput<V> {
    pub fn new_options(value: V, options: impl IntoWatchable<Vec<V>> + 'static) -> VariantInput<V> {
        Self::from_options(Field::new(value), options)
    }
    fn from_options<F: Into<DynWatchableSetter<V>> + Clone>(
        value: F,
        options: impl IntoWatchable<Vec<V>> + 'static,
    ) -> VariantInput<V> {
        let value = value.into();
        let value_clone = value.clone();
        let options = DynWatchable::new(options.into_watchable());
        let options_clone = options.clone();
        VariantInput {
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
    pub fn comp_builder<I: Into<Component>, M: Fn(&V) -> I + 'static>(
        self,
        map: M,
    ) -> VariantInputCompBuilder<SetOptions> {
        VariantComponentMapper::new(self, map).comp_builder()
    }
}
pub trait VariantOptions: Sized + Clone + Eq {
    /// The first value is considered the default, and used as a fallback
    fn get_variants() -> Vec<Self>;
}
impl<V: Sized + Clone + Eq + VariantOptions> VariantInput<V> {
    pub fn new(value: V) -> VariantInput<V> {
        Self::from_options(Field::new(value), V::get_variants())
    }
}
impl<V: Sized + Clone + Eq> Clone for VariantInput<V> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            options: self.options.clone(),
            filtered: self.filtered.clone(),
        }
    }
}
impl<V: Sized + Clone + Eq + 'static> WatchableState for VariantInput<V> {
    fn state(&self) -> DataState {
        self.filtered.state()
    }
    fn observe(&self, listener: Box<dyn Listener>) -> Observer {
        self.filtered.observe(listener)
    }
}
impl<V: Sized + Clone + Eq + 'static> Watchable for VariantInput<V> {
    type Output = V;
    fn get(&self) -> Rc<Self::Output> {
        self.filtered.get()
    }
}
impl<V: Sized + Clone + Eq + 'static> Setter for VariantInput<V> {
    type Input = V;
    fn set(&mut self, val: V) -> DynSignaller {
        self.value.set(val)
    }
}

impl<V: Sized + Clone + Eq + 'static> Into<DynWatchable<V>> for VariantInput<V> {
    fn into(self) -> DynWatchable<V> {
        DynWatchable::new(self)
    }
}
impl<V: Sized + Clone + Eq + 'static> Into<DynWatchableSetter<V>> for VariantInput<V> {
    fn into(self) -> DynWatchableSetter<V> {
        DynWatchableSetter::new(self)
    }
}

impl<V: Sized + Clone + Eq + 'static> Inheritable for InheritedInput<VariantInput<V>> {
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + 'static) -> Self {
        let input = self.input();
        InheritedInput::new(
            VariantInput::from_options(Field::new((*input.get()).clone()), input.options.clone()),
            DynWatchable::new(self.clone()),
            self_name,
        )
    }
}
impl<V: Sized + Clone + Eq + VariantOptions + 'static> From<V> for VariantInput<V> {
    fn from(value: V) -> Self {
        VariantInput::from_options(Field::new(value.into()), V::get_variants())
    }
}
impl<V: Sized + Clone + Eq + VariantOptions + 'static> From<V> for InheritedInput<VariantInput<V>> {
    fn from(value: V) -> Self {
        Self::from(VariantInput::from(value))
    }
}
impl<V: Sized + Clone + Eq + 'static> From<(V, Vec<V>)> for VariantInput<V> {
    fn from((value, options): (V, Vec<V>)) -> Self {
        VariantInput::from_options(Field::new(value), options)
    }
}
impl<V: Sized + Clone + Eq + VariantOptions + 'static> From<(V, Vec<V>)>
    for InheritedInput<VariantInput<V>>
{
    fn from(value: (V, Vec<V>)) -> Self {
        Self::from(VariantInput::from(value))
    }
}

/// Maps variants to the components representing them
pub struct VariantComponentMapper<V: Sized + Clone + Eq + 'static> {
    data: VariantInput<V>,
    option_components: ComponentVecWatchable,
    selected_index: U32Watchable,
}
impl<V: Sized + Clone + Eq + 'static> VariantComponentMapper<V> {
    pub fn new<I: Into<Component>, M: Fn(&V) -> I + 'static>(
        data: impl Into<VariantInput<V>>,
        map: M,
    ) -> Self {
        let data = data.into();
        let options = data.options.clone();
        let option_components = ComponentVecWatchable::new(Derived::new(move |t| {
            let options = options.watch(t);
            (*options).iter().map(|v| map(v).into()).collect::<Vec<_>>()
        }));
        let options = data.options.clone();
        let selected = data.value.clone();
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
    pub fn from(data: impl Into<VariantInput<V>>, map: impl Fn(&V) -> Component + 'static) -> Self {
        Self::new(data.into(), map)
    }
    pub fn comp_builder(self) -> VariantInputCompBuilder<SetOptions> {
        VariantInputComp::builder(self.clone()).options(self.option_components)
    }
}
impl<V: Sized + Clone + Eq> Clone for VariantComponentMapper<V> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            option_components: self.option_components.clone(),
            selected_index: self.selected_index.clone(),
        }
    }
}
impl<V: Sized + Clone + Eq + 'static> WatchableState for VariantComponentMapper<V> {
    fn state(&self) -> DataState {
        self.selected_index.state()
    }
    fn observe(&self, listener: Box<dyn Listener>) -> Observer {
        self.selected_index.observe(listener)
    }
}
impl<V: Sized + Clone + Eq + 'static> Watchable for VariantComponentMapper<V> {
    type Output = u32;
    fn get(&self) -> Rc<Self::Output> {
        Watchable::get(&self.selected_index)
    }
}
impl<V: Sized + Clone + Eq + 'static> Setter for VariantComponentMapper<V> {
    type Input = u32;
    fn set(&mut self, val: u32) -> DynSignaller {
        let options = self.data.options.get();
        let option = (*options).get(val as usize);
        Box::new(option.map(|option| self.data.set(option.clone())))
    }
}
impl<V: Sized + Clone + Eq + 'static> Into<DynWatchable<u32>> for VariantComponentMapper<V> {
    fn into(self) -> DynWatchable<u32> {
        DynWatchable::new(self)
    }
}
impl<V: Sized + Clone + Eq + 'static> Into<DynWatchableSetter<u32>> for VariantComponentMapper<V> {
    fn into(self) -> DynWatchableSetter<u32> {
        DynWatchableSetter::new(self)
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

/// Variant component.
#[wasm_getters]
#[wasm_bindgen]
#[builder_into_comp]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct VariantInputComp {
    /// The data of the component.
    #[builder(start_fn, into)]
    data: DynWatchableSetter<u32>,
    /// The options of the dropdown
    #[getter]
    #[setter(Vec<Component>)]
    options: ComponentVecWatchable,
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
    /// Wraps the output component
    #[builder(default=IdentityWrapper::new())]
    wrapper: Rc<dyn CompWrapper>,
}
impl VariantInputComp {
    pub fn wrap_builder_map<
        V: Sized + Clone + Eq + 'static,
        I: Into<VariantInput<V>>,
        C: Into<Component>,
        M: Fn(&V) -> C + 'static,
    >(
        wrapper: impl CompWrapper + InputWrapper<I> + Clone + 'static,
        map: M,
    ) -> VariantInputCompBuilder<SetWrapper<SetOptions>> {
        let input = wrapper.get_input().into();
        VariantComponentMapper::new(input, map)
            .comp_builder()
            .wrapper(Rc::new(wrapper))
    }
    pub fn wrap_builder<V: Sized + Clone + Eq + 'static, I: Into<VariantComponentMapper<V>>>(
        wrapper: impl CompWrapper + InputWrapper<I> + Clone + 'static,
    ) -> VariantInputCompBuilder<SetWrapper<SetOptions>> {
        let input = wrapper.get_input().into();
        Self::builder(input.clone())
            .options(input.option_components.clone())
            .wrapper(Rc::new(wrapper))
    }

    fn watchable(&self) -> &DynWatchableSetter<u32> {
        &self.data
    }
    fn setter(&mut self) -> &mut DynWatchableSetter<u32> {
        &mut self.data
    }
}
impl_watchable!(VariantInputComp, u32);
impl_setter!(VariantInputComp, u32);

impl<V: Sized + Clone + Eq + VariantComponentMapping + VariantOptions> Into<VariantInputComp>
    for VariantInput<V>
{
    fn into(self) -> VariantInputComp {
        Into::<VariantComponentMapper<V>>::into(self).into()
    }
}
impl<V: Sized + Clone + Eq + 'static> Into<VariantInputComp> for VariantComponentMapper<V> {
    fn into(self) -> VariantInputComp {
        VariantInputComp::builder(DynWatchableSetter::new(self.clone()))
            .options(self.option_components)
            .build()
    }
}

impl<V: Sized + Clone + Eq + VariantComponentMapping + VariantOptions> Into<Component>
    for VariantInput<V>
{
    fn into(self) -> Component {
        Into::<VariantInputComp>::into(self).into()
    }
}
impl<V: Sized + Clone + Eq> Into<Component> for VariantComponentMapper<V> {
    fn into(self) -> Component {
        Into::<VariantInputComp>::into(self).into()
    }
}
impl Into<Component> for VariantInputComp {
    fn into(self) -> Component {
        let wrapper = self.wrapper.clone();
        let comp = Component::new(ComponentOption::VariantInput(self));
        wrapper.wrap(comp)
    }
}
