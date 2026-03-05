use std::rc::Rc;

use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{new_wasm_interface::{Component, ComponentOption}, util::watchables::{
     BoolWatchable, Constant, DataState, Derived, IntoWatchable, JsListener, Listener, Mutator, Observer, OptionBoolWatchable, OptionU32Watchable, U32Field, U32Watchable, Watchable, WatchableState, Watching, impl_watchable, signaller::Signaller
}};

/// u32 input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct U32Input(U32Field);
impl U32Input {
    pub fn new(val: u32) -> Self {
        U32Input(U32Field::new(val))
    }
    pub fn from<V: Into<u32>>(val: V) -> Self {
        U32Input(U32Field::from(val))
    }
    fn watchable(&self) -> &U32Field {
        &self.0
    }
    pub fn set(&mut self, val: u32) -> Signaller {
        self.0.set(val)
    }
}
impl_watchable!(U32Input, u32);
#[wasm_bindgen]
impl U32Input {
    #[wasm_bindgen(js_name="set")]
    pub fn set_js(&mut self, val: u32) -> Mutator {
        self.0.set_js(val)
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
    pub fn set(&mut self, val: u32) -> Signaller {
        self.input.set(val)
    }
    fn watchable(&self) -> &U32Watchable {
        &self.clamped
    }
}
impl_watchable!(U32InputClamped, u32);
#[wasm_bindgen]
impl U32InputClamped {
    #[wasm_bindgen(js_name="set")]
    pub fn set_js(&mut self, val: u32) -> Mutator {
        self.input.set_js(val)
    }
}
impl Into<U32InputClamped> for U32Input {
    fn into(self) -> U32InputClamped {
        U32InputClamped::builder().input(self).build()
    }
}


/// u32 input component
#[derive(Clone)]
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder)]
pub struct U32InputComp {
    /// The data of the component
    #[getter]
    #[builder(into)]
    data: U32InputClamped,
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
    disabled: BoolWatchable
}
impl U32InputComp {
    fn watchable(&self) -> &U32InputClamped {
        &self.data
    }
}
impl_watchable!(U32InputComp, u32);

impl Into<U32InputComp> for U32Input {
    fn into(self) -> U32InputComp {
        U32InputComp::builder().data(self).build()
    }
}
impl Into<U32InputComp> for U32InputClamped {
    fn into(self) -> U32InputComp {
        U32InputComp::builder().data(self).build()
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
        Component::new(ComponentOption::U32Input(self))
    }
}