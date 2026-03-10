use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::dyn_component::{ComponentWatchable, DynComp},
    make_typed_dyn_watchable,
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::F32Watchable,
};

#[wasm_bindgen]
#[derive(Clone)]
pub enum TooltipDelay {
    Long,
    Medium,
    Zero,
}
make_typed_dyn_watchable!(TooltipDelayWatchable, TooltipDelay);

#[wasm_bindgen]
#[derive(Clone)]
pub enum DirectionHint {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    LeftTop,
    LeftCenter,
    LeftBottom,
    RightTop,
    RightCenter,
    RightBottom,
    None,
}
make_typed_dyn_watchable!(DirectionHintWatchable, DirectionHint);

/// Tooltip component that renders a tooltip on hover
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct TooltipComp {
    /// The main component that is rendered by default.
    #[getter]
    #[builder(finish_fn, into)]
    content: DynComp,
    /// The delay before the tooltip closes
    #[getter]
    #[setter(f32, 0.0)]
    close_delay: F32Watchable,
    /// The delay before the tooltip opens
    #[getter]
    #[setter(TooltipDelay, TooltipDelay::Medium)]
    delay: TooltipDelayWatchable,
    /// The direction hint to open in
    #[getter]
    #[setter(DirectionHint, DirectionHint::None)]
    direction: DirectionHintWatchable,
    /// The tooltip hover content.
    #[getter]
    #[builder(into)]
    tooltip: DynComp,
}

impl TooltipComp {
    pub fn new(tooltip: impl Into<DynComp> + 'static, main: impl Into<DynComp> + 'static) -> Self {
        Self::builder().tooltip(tooltip).build(main)
    }
}

impl Into<Component> for TooltipComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::Tooltip(self))
    }
}
