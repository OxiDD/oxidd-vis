use std::rc::Rc;

use app_macros::{builder_into_comp, wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{impl_setter, impl_watchable, inputs::{InheritLabel, Inheritable, InheritedInput, u32_input::u32_input_comp_builder::SetWrapper, wrapper::{CompWrapper, IdentityWrapper, InputWrapper}}, new_wasm_interface::{Component, ComponentOption}, util::watchables::{
     BoolWatchable, Constant, DataState, Derived, DynSignaller, DynWatchable, DynWatchableSetter, IntoWatchable, IntoWatchableSetter, JsListener, Listener, MutateSetter, Mutator, Observer, OptionBoolWatchable, OptionU32Watchable, Setter, U32Field, U32Watchable, Watchable, WatchableState, Watching, signaller::Signaller
}};


/// Number input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct U32Input(U32Field);
impl U32Input {
    pub fn new(val: u32) -> Self {
        U32Input(U32Field::new(val).into())
    }
    fn watchable(&self) -> &U32Field {
        &self.0
    }
    fn setter(&mut self) -> &mut U32Field {
        &mut self.0
    }
}
impl_watchable!(U32Input, u32);
impl_setter!(U32Input, u32);

impl Inheritable for InheritedInput<U32Input> {
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + 'static) -> Self {
        InheritedInput::new(
            U32Input::new(*self.get()),
            DynWatchable::new(self.clone()),
            self_name,
        )
    }
}
impl<X: Into<u32>> From<X> for U32Input {
    fn from(value: X) -> Self {
        Self::new(value.into())
    }
}
impl<X: Into<u32>> From<X> for InheritedInput<U32Input> {
    fn from(value: X) -> Self {
        Self::from(U32Input::from(value))
    }
}


/// Clamped u32 input
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct U32InputClamped {
    /// The raw input
    #[getter]
    #[builder(start_fn, into)]
    input: U32Input,
    /// The minimum value
    #[getter]
    #[setter(Option<u32>)]
    min: OptionU32Watchable,
    /// The maximum value
    #[getter]
    #[setter(Option<u32>)]
    max: OptionU32Watchable,
    /// The precision to use for the number, e.g. `0.5` would round to the nearest half
    #[getter]
    #[setter(Option<u32>)]
    precision: OptionU32Watchable,
    /// The output clamped value based on the input and constraints
    #[builder(skip=clamp(
        U32Watchable::new(input.clone()), 
        min.clone(), 
        max.clone(), 
        precision.clone()
    ))]
    #[getter]
    clamped: U32Watchable,
}

fn clamp(
    val: U32Watchable,
    min: OptionU32Watchable,
    max: OptionU32Watchable,
    precision: OptionU32Watchable,
) -> U32Watchable {
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
            val = (((val as f32) / (precision as f32)).round() as u32) * precision;
        }
        val
    });
    U32Watchable::new(clamped)
}
impl U32InputClamped {
    pub fn clamp(&self, val: U32Watchable) -> U32Watchable {
        let min = self.min.clone();
        let max = self.max.clone();
        let precision = self.precision.clone();
        clamp(val, min, max, precision)
    }
    fn watchable(&self) -> &U32Watchable {
        &self.clamped
    }
    fn setter(&mut self) -> &mut U32Input {
        &mut self.input
    }
}
impl_watchable!(U32InputClamped, u32);
impl_setter!(U32InputClamped, u32);

impl Inheritable for InheritedInput<U32InputClamped> {
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + 'static) -> Self {
        let p = self.input();
        InheritedInput::new(
            U32InputClamped::builder(U32Input::new(p.get()))
                .min(p.min.clone())
                .max(p.max.clone())
                .precision(p.precision.clone())
                .build(),
            DynWatchable::new(self.clone()),
            self_name,
        )
    }
}
impl<X: Into<u32>> From<X> for U32InputClamped {
    fn from(value: X) -> Self {
        Self::builder(U32Input::from(value)).build()
    }
}
impl<X: Into<u32>> From<X> for InheritedInput<U32InputClamped> {
    fn from(value: X) -> Self {
        Self::from(U32InputClamped::from(value))
    }
}


/// u32 input component
#[wasm_getters]
#[wasm_bindgen]
#[builder_into_comp]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct U32InputComp {
    /// The data of the component
    #[builder(start_fn, into)]
    data: DynWatchableSetter<u32>,
    /// Whether to supply step buttons
    #[getter]
    #[setter(Option<u32>)]
    step_size: OptionU32Watchable,
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
impl U32InputComp {
    pub fn wrap_builder<I: Into<DynWatchableSetter<u32>>>(
        wrapper: impl CompWrapper + InputWrapper<I> + Clone + 'static,
    ) -> U32InputCompBuilder<SetWrapper> {
        Self::builder(wrapper.get_input()).wrapper(Rc::new(wrapper))
    }
    fn watchable(&self) -> &DynWatchableSetter<u32> {
        &self.data
    }
    fn setter(&mut self) -> &mut DynWatchableSetter<u32> {
        &mut self.data
    }
}
impl_watchable!(U32InputComp, u32);
impl_setter!(U32InputComp, u32);

impl Into<U32InputComp> for U32Input {
    fn into(self) -> U32InputComp {
        U32InputComp::builder(self).build()
    }
}
impl Into<U32InputComp> for U32InputClamped {
    fn into(self) -> U32InputComp {
        U32InputComp::builder(self).build()
    }
}

impl Into<Component> for U32Input {
    fn into(self) -> Component {
        Into::<U32InputComp>::into(self).into()
    }
}
impl Into<Component> for U32InputClamped {
    fn into(self) -> Component {
        Into::<U32InputComp>::into(self).into()
    }
}
impl Into<Component> for U32InputComp {
    fn into(self) -> Component {
        let wrapper = self.wrapper.clone();
        let comp = Component::new(ComponentOption::U32Input(self));
        wrapper.wrap(comp)
    }
}