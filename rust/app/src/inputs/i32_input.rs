use std::rc::Rc;

use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{new_wasm_interface::{Component, ComponentOption}, util::watchables::{
     BoolWatchable, Constant, DataState, Derived, I32Field, I32Watchable, IntoWatchable, JsListener, Listener, Mutator, Observer, OptionBoolWatchable, OptionI32Watchable, Watchable, WatchableState, Watching, impl_watchable, signaller::Signaller
}};

/// i32 input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct I32Input(I32Field);
impl I32Input {
    pub fn new(val: i32) -> Self {
        I32Input(I32Field::new(val))
    }
    pub fn from<V: Into<i32>>(val: V) -> Self {
        I32Input(I32Field::from(val))
    }
    fn watchable(&self) -> &I32Field {
        &self.0
    }
    pub fn set(&mut self, val: i32) -> Signaller {
        self.0.set(val)
    }
}
impl_watchable!(I32Input, i32);
#[wasm_bindgen]
impl I32Input {
    #[wasm_bindgen(js_name="set")]
    pub fn set_js(&mut self, val: i32) -> Mutator {
        self.0.set_js(val)
    }
}

/// Clamped i32 input
#[derive(Clone)]
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder)]
pub struct I32InputClamped {
    /// The raw input
    #[getter]
    input: I32Input,
    /// The minimum value
    #[getter]
    #[setter(Option<i32>)]
    min: OptionI32Watchable,
    /// The maximum value
    #[getter]
    #[setter(Option<i32>)]
    max: OptionI32Watchable,
    /// The precision to use for the number, e.g. `0.5` would round to the nearest half
    #[getter]
    #[setter(Option<i32>)]
    precision: OptionI32Watchable,
    /// The output clamped value based on the input and constraints
    #[builder(skip=clamp(
        I32Watchable::new(input.clone()), 
        min.clone(), 
        max.clone(), 
        precision.clone()
    ))]
    #[getter]
    clamped: I32Watchable,
}

fn clamp(
    val: I32Watchable,
    min: OptionI32Watchable,
    max: OptionI32Watchable,
    precision: OptionI32Watchable,
) -> I32Watchable {
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
            val = (((val as f32) / (precision as f32)).round() as i32) * precision;
        }
        val
    });
    I32Watchable::new(clamped)
}
impl I32InputClamped {
    pub fn clamp(&self, val: I32Watchable) -> I32Watchable {
        let min = self.min.clone();
        let max = self.max.clone();
        let precision = self.precision.clone();
        clamp(val, min, max, precision)
    }
    pub fn set(&mut self, val: i32) -> Signaller {
        self.input.set(val)
    }
    fn watchable(&self) -> &I32Watchable {
        &self.clamped
    }
}
impl_watchable!(I32InputClamped, i32);
#[wasm_bindgen]
impl I32InputClamped {
    #[wasm_bindgen(js_name="set")]
    pub fn set_js(&mut self, val: i32) -> Mutator {
        self.input.set_js(val)
    }
}
impl Into<I32InputClamped> for I32Input {
    fn into(self) -> I32InputClamped {
        I32InputClamped::builder().input(self).build()
    }
}


/// i32 input component
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct I32InputComp {
    /// The data of the component
    #[getter]
    #[builder(into)]
    data: I32InputClamped,
    /// Whether to supply step buttons
    #[getter]
    #[setter(Option<i32>)]
    step_size: OptionI32Watchable,
    /// Whether to apply rounding when performing a step
    #[getter]
    #[setter(bool, false)]
    step_round: BoolWatchable,    
    /// Whether this input is disabled
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable
}
impl I32InputComp {
    fn watchable(&self) -> &I32InputClamped {
        &self.data
    }
}
impl_watchable!(I32InputComp, i32);

impl Into<I32InputComp> for I32Input {
    fn into(self) -> I32InputComp {
        I32InputComp::builder().data(self).build()
    }
}
impl Into<I32InputComp> for I32InputClamped {
    fn into(self) -> I32InputComp {
        I32InputComp::builder().data(self).build()
    }
}

impl Into<Component> for I32Input {
    fn into(self) -> Component {
        Into::<I32InputComp>::into(self).into()
    }
}
impl Into<Component> for I32InputClamped {
    fn into(self) -> Component {
        Into::<I32InputComp>::into(self).into()
    }
}
impl Into<Component> for I32InputComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::I32Input(self))
    }
}