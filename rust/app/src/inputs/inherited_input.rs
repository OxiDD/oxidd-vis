use std::rc::Rc;

use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::DynComp,
    inputs::wrapper::{CompWrapper, InputWrapper},
    make_typed_dyn_watchable,
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        BoolWatchable, Constant, DataState, Derived, DynSignaller, DynWatchable,
        DynWatchableSetter, Field, IntoWatchable, Listener, MutateSetter, Mutator, Observer,
        Setter, StringWatchable, Watchable, WatchableSetter, WatchableState, Watcher, Watching,
    },
};

#[wasm_getters]
#[wasm_bindgen]
#[derive(Clone)]
pub struct InheritLabel {
    #[getter]
    inherited_label: String,
    #[getter]
    local_label: String,
}
make_typed_dyn_watchable!(InheritLabelWatchable, InheritLabel);
impl<W: IntoWatchable<String> + 'static> IntoWatchable<InheritLabel> for W {
    type Output = Derived<InheritLabel>;
    fn into_watchable(self) -> Self::Output {
        let watchable = self.into_watchable();
        Derived::new(move |t| {
            let text = t.watch(&watchable);
            InheritLabel {
                inherited_label: format!("Inheriting from {text}"),
                local_label: format!("Inherit from {text}"),
            }
        })
    }
}
pub trait Inheritable {
    /// Creates a new input that inherits from this input, and label this
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + Clone + 'static) -> Self;
}
impl<F: WatchableSetter + Clone> From<F> for InheritedInput<F>
where
    F::Input: Clone,
{
    fn from(value: F) -> Self {
        InheritedInput::default(value)
    }
}

/// Optionally inherited input data
#[watchable_setters]
#[derive(Builder)]
pub struct InheritedInput<F: WatchableSetter + Clone + 'static> {
    /// The data of the component
    #[builder(start_fn, into)]
    local: F,
    /// The inherited value
    #[builder(into)]
    inherited: DynWatchable<F::Output>,
    /// Whether or not we are inheriting now
    #[builder(default=DynWatchableSetter::new(Field::new(true)))]
    inheriting: DynWatchableSetter<bool>,
    /// The name of the source of the inherited value
    #[setter(InheritLabel)]
    inherited_from: InheritLabelWatchable,
    /// The final output
    #[builder(skip=InheritedInput::<F>::inherited(local.clone(), inherited.clone(), inheriting.clone()))]
    output: Derived<F::Output>,
}
impl<F: WatchableSetter + Clone + 'static> CompWrapper for InheritedInput<F> {
    fn wrap(&self, comp: Component) -> Component {
        InheritedInputComp::new(self.clone(), |_| comp).into()
    }
}
impl<F: WatchableSetter + Clone + 'static> InputWrapper<InheritedInput<F>> for InheritedInput<F> {
    fn get_input(&self) -> InheritedInput<F> {
        self.clone()
    }
}
impl<F: WatchableSetter + Clone + 'static> Clone for InheritedInput<F> {
    fn clone(&self) -> Self {
        Self {
            inherited: self.inherited.clone(),
            local: self.local.clone(),
            inheriting: self.inheriting.clone(),
            inherited_from: self.inherited_from.clone(),
            output: self.output.clone(),
        }
    }
}
impl<F: WatchableSetter + Clone + 'static> InheritedInput<F>
where
    F::Input: Clone,
{
    pub fn default(local: F) -> Self {
        let default = (*local.get()).clone();
        Self::builder(local)
            .inherited(DynWatchable::new(Constant::new(default)))
            .inherited_from(InheritLabel {
                inherited_label: "Set to default".to_string(),
                local_label: "Reset to default".to_string(),
            })
            .build()
    }
}
impl<F: WatchableSetter + Clone + 'static> InheritedInput<F> {
    pub fn new(
        local: F,
        inherited: impl Into<DynWatchable<F::Output>>,
        inherited_from: impl IntoWatchable<InheritLabel> + 'static,
    ) -> Self {
        Self::builder(local)
            .inherited(inherited)
            .inherited_from(inherited_from)
            .build()
    }
    fn inherited(
        local: F,
        inherited: DynWatchable<F::Output>,
        inheriting: DynWatchableSetter<bool>,
    ) -> Derived<F::Output> {
        Derived::new_rc(move |t| {
            if *inheriting.watch(t) {
                inherited.watch(t)
            } else {
                local.watch(t)
            }
        })
    }

    pub fn input(&self) -> &F {
        &self.local
    }
    pub fn set_inherit(&mut self, inherit: bool) -> DynSignaller {
        self.inheriting.set(inherit)
    }

    /// Creates the component for this inherited input
    pub fn comp<I: Into<Component>, M: FnOnce(Self, &F) -> I>(self, map: M) -> InheritedInputComp {
        InheritedInputComp::new(self.clone(), |val| map(self, val))
    }
}
impl<F: WatchableSetter + Clone + 'static> WatchableState for InheritedInput<F> {
    fn state(&self) -> DataState {
        self.output.state()
    }
    fn observe(&self, listener: Box<dyn Listener>) -> Observer {
        self.output.observe(listener)
    }
}
impl<F: WatchableSetter + Clone + 'static> Watchable for InheritedInput<F> {
    type Output = F::Output;
    fn get(&self) -> Rc<Self::Output> {
        self.output.get()
    }
}
impl<F: WatchableSetter + Clone + 'static> Setter for InheritedInput<F> {
    type Input = F::Output;
    fn set(&mut self, val: F::Output) -> DynSignaller {
        Box::new((self.local.set(val), self.inheriting.set(false)))
    }
}

impl<F: WatchableSetter + Clone + 'static> Into<DynWatchableSetter<F::Output>>
    for InheritedInput<F>
{
    fn into(self) -> DynWatchableSetter<F::Output> {
        DynWatchableSetter::new(self)
    }
}

/// Inherited input component.
#[wasm_getters]
#[wasm_bindgen]
#[derive(Clone)]
pub struct InheritedInputComp {
    /// The input component.
    #[getter]
    input: DynComp,
    /// Whether the value is currently being inherited.
    #[getter]
    inheriting: BoolWatchable,
    /// The text indicating where the inherited value comes from.
    #[getter]
    inherited_from: InheritLabelWatchable,
    /// The field that stores whether inheriting
    inheriting_field: DynWatchableSetter<bool>,
}
impl InheritedInputComp {
    pub fn from<F: Into<Component> + WatchableSetter + Clone + 'static>(
        data: InheritedInput<F>,
    ) -> Self {
        Self {
            input: DynComp::new(data.local.clone().into()),
            inheriting_field: data.inheriting.clone(),
            inheriting: BoolWatchable::new(data.inheriting.clone()),
            inherited_from: data.inherited_from.clone(),
        }
    }
    pub fn new<F: WatchableSetter + Clone + 'static, I: Into<Component>, M: FnOnce(&F) -> I>(
        data: InheritedInput<F>,
        map: M,
    ) -> Self {
        Self {
            input: DynComp::new(map(&data.local).into()),
            inheriting_field: data.inheriting.clone(),
            inheriting: BoolWatchable::new(data.inheriting.clone()),
            inherited_from: data.inherited_from.clone(),
        }
    }
}

#[wasm_bindgen]
impl InheritedInputComp {
    /// Starts inheriting the value
    pub fn set_inherit(&mut self, inheriting: bool) -> Mutator {
        self.inheriting_field.mutate_set(inheriting)
    }
}

impl Into<Component> for InheritedInputComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::InheritedInput(self))
    }
}
