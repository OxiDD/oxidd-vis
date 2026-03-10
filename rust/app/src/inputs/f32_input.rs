use std::rc::Rc;

use app_macros::{builder_into_comp, wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{impl_setter, impl_watchable, inputs::{ InheritLabel, Inheritable, InheritedInput, f32_input::f32_input_comp_builder::SetWrapper, wrapper::{CompWrapper, IdentityWrapper, InputWrapper}}, new_wasm_interface::{Component, ComponentOption}, util::watchables::{
    BoolWatchable, Constant, DataState, Derived, DynSignaller, DynWatchable, DynWatchableSetter, F32Field, F32Watchable, IntoWatchable, IntoWatchableSetter, JsListener, Listener, MutateSetter, Mutator, Observer, OptionBoolWatchable, OptionF32Watchable, Setter, Watchable, WatchableState, Watching, signaller::Signaller
}};

/// Number input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct F32Input(F32Field);
impl F32Input {
    pub fn new(val: f32) -> Self {
        F32Input(F32Field::new(val).into())
    }
    fn watchable(&self) -> &F32Field {
        &self.0
    }
    fn setter(&mut self) -> &mut F32Field {
        &mut self.0
    }
}
impl_watchable!(F32Input, f32);
impl_setter!(F32Input, f32);

impl Inheritable for InheritedInput<F32Input> {
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + 'static) -> Self {
        InheritedInput::new(
            F32Input::new(*self.get()),
            DynWatchable::new(self.clone()),
            self_name,
        )
    }
}
impl<X: Into<f32>> From<X> for F32Input {
    fn from(value: X) -> Self {
        Self::new(value.into())
    }
}
impl<X: Into<f32>> From<X> for InheritedInput<F32Input> {
    fn from(value: X) -> Self {
        Self::from(F32Input::from(value))
    }
}

/// Clamped f32 input
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct F32InputClamped {
    /// The raw input
    #[getter]
    #[builder(start_fn, into)]
    input: F32Input,
    /// The minimum value
    #[getter]
    #[setter(Option<f32>)]
    min: OptionF32Watchable,
    /// The maximum value
    #[getter]
    #[setter(Option<f32>)]
    max: OptionF32Watchable,
    /// The precision to use for the number, e.g. `0.5` would round to the nearest half
    #[getter]
    #[setter(Option<f32>)]
    precision: OptionF32Watchable,
    /// The output clamped value based on the input and constraints
    #[builder(skip=clamp(
        F32Watchable::new(input.clone()), 
        min.clone(), 
        max.clone(), 
        precision.clone()
    ))]
    #[getter]
    clamped: F32Watchable,
}

fn clamp(
    val: F32Watchable,
    min: OptionF32Watchable,
    max: OptionF32Watchable,
    precision: OptionF32Watchable,
) -> F32Watchable {
    let clamped = Derived::new(move |t| {
        let mut val = *val.watch(t);
        if let Some(min) = *min.watch(t) {
            if val < min {
                val = min;
            }
        }
        if let Some(max) = *max.watch(t) {
            if val > max {
                val = max;
            }
        }
        if let Some(precision) = *precision.watch(t) {
            val = (val / precision).round() * precision;
        }
        val
    });
    F32Watchable::new(clamped)
}
impl F32InputClamped {
    pub fn clamp(&self, val: F32Watchable) -> F32Watchable {
        let min = self.min.clone();
        let max = self.max.clone();
        let precision = self.precision.clone();
        clamp(val, min, max, precision)
    }
    fn watchable(&self) -> &F32Watchable {
        &self.clamped
    }
    fn setter(&mut self) -> &mut F32Input {
        &mut self.input
    }
}
impl_watchable!(F32InputClamped, f32);
impl_setter!(F32InputClamped, f32);

impl Inheritable for InheritedInput<F32InputClamped> {
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + 'static) -> Self {
        let p = self.input();
        InheritedInput::new(
            F32InputClamped::builder(F32Input::new(p.get()))
                .min(p.min.clone())
                .max(p.max.clone())
                .precision(p.precision.clone())
                .build(),
            DynWatchable::new(self.clone()),
            self_name,
        )
    }
}
impl<X: Into<f32>> From<X> for F32InputClamped {
    fn from(value: X) -> Self {
        Self::builder(F32Input::from(value)).build()
    }
}
impl<X: Into<f32>> From<X> for InheritedInput<F32InputClamped> {
    fn from(value: X) -> Self {
        Self::from(F32InputClamped::from(value))
    }
}

/// f32 input component
#[wasm_getters]
#[wasm_bindgen]
#[builder_into_comp]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct F32InputComp {
    /// The data of the component
    #[builder(start_fn, into)]
    data: DynWatchableSetter<f32>,
    /// Whether to supply step buttons
    #[getter]
    #[setter(Option<f32>)]
    step_size: OptionF32Watchable,
    /// Whether to apply rounding when performing a step
    #[getter]
    #[setter(bool, false)]
    step_round: BoolWatchable,    
    /// Whether this input is disabled
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable,
    /// Wraps the output component
    #[builder(default=IdentityWrapper::new())]
    wrapper: Rc<dyn CompWrapper>,
}
impl F32InputComp {
    pub fn wrap_builder<I: Into<DynWatchableSetter<f32>>>(
        wrapper: impl CompWrapper + InputWrapper<I> + Clone + 'static,
    ) -> F32InputCompBuilder<SetWrapper> {
        Self::builder(wrapper.get_input()).wrapper(Rc::new(wrapper))
    }
    fn watchable(&self) -> &DynWatchableSetter<f32> {
        &self.data
    }
    fn setter(&mut self) -> &mut DynWatchableSetter<f32> {
        &mut self.data
    }
}
impl_watchable!(F32InputComp, f32);
impl_setter!(F32InputComp, f32);

impl Into<F32InputComp> for F32Input {
    fn into(self) -> F32InputComp {
        F32InputComp::builder(self).build()
    }
}
impl Into<F32InputComp> for F32InputClamped {
    fn into(self) -> F32InputComp {
        F32InputComp::builder(self).build()
    }
}
impl Into<Component> for F32Input {
    fn into(self) -> Component {
        Into::<F32InputComp>::into(self).into()
    }
}
impl Into<Component> for F32InputClamped {
    fn into(self) -> Component {
        Into::<F32InputComp>::into(self).into()
    }
}
impl Into<Component> for F32InputComp {
    fn into(self) -> Component {
        let wrapper = self.wrapper.clone();
        let comp = Component::new(ComponentOption::F32Input(self));
        wrapper.wrap(comp)
    }
}
