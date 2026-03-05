use std::rc::Rc;

use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{new_wasm_interface::{Component, ComponentOption}, util::watchables::{
    BoolWatchable, ClonableWatchableUtils, Constant, DataState, Derived, F32Field, F32Watchable, Field, IntoWatchable, JsListener, Listener, Mutator, Observer, OptionBoolWatchable, OptionF32Watchable, Watchable, WatchableState, Watching, impl_watchable, signaller::Signaller
}};

/// f32 input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct F32Input(F32Field);
impl F32Input {
    pub fn new(val: f32) -> Self {
        F32Input(F32Field::new(val))
    }
    pub fn from<V: Into<f32>>(val: V) -> Self {
        F32Input(F32Field::from(val))
    }
    fn watchable(&self) -> &F32Field {
        &self.0
    }
    pub fn set(&mut self, val: f32) -> Signaller {
        self.0.set(val)
    }
}
impl_watchable!(F32Input, f32);
#[wasm_bindgen]
impl F32Input {
    #[wasm_bindgen(js_name="set")]
    pub fn set_js(&mut self, val: f32) -> Mutator {
        self.0.set_js(val)
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
    pub fn set(&mut self, val: f32) -> Signaller {
        self.input.set(val)
    }
    fn watchable(&self) -> &F32Watchable {
        &self.clamped
    }
}
impl_watchable!(F32InputClamped, f32);
#[wasm_bindgen]
impl F32InputClamped {
    #[wasm_bindgen(js_name="set")]
    pub fn set_js(&mut self, val: f32) -> Mutator {
        self.input.set_js(val)
    }
}

impl Into<F32InputClamped> for F32Input {
    fn into(self) -> F32InputClamped {
        F32InputClamped::builder(self).build()
    }
}


/// f32 input component
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct F32InputComp {
    /// The data of the component
    #[getter]
    #[builder(start_fn, into)]
    data: F32InputClamped,
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
    disabled: BoolWatchable
}
impl F32InputComp {
    fn watchable(&self) -> &F32InputClamped {
        &self.data
    }
}
impl_watchable!(F32InputComp, f32);

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
        Component::new(ComponentOption::F32Input(self))
    }
}
