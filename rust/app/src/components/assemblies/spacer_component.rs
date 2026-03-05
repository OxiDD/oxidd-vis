use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::{container_component::ContainerComp, dyn_component::DynComp, CompositeItemComp},
    new_wasm_interface::Component,
    util::watchables::{F32Watchable, OptionF32Watchable, WatchableUtils},
};

/// Helper component that creates spacing around a child using a `ContainerComp`.
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct SpacerComp {
    /// Relative fill ratio compared to siblings.
    #[getter]
    #[setter(f32, 1.0)]
    ratio: F32Watchable,
    /// Optional target width in logical units.
    #[getter]
    #[setter(Option<f32>)]
    width: OptionF32Watchable,
    /// Optional target height in logical units.
    #[getter]
    #[setter(Option<f32>)]
    height: OptionF32Watchable,
    /// The spaced component, implemented via a `ContainerComp` with `min_width`/`min_height`.
    #[getter]
    #[builder(skip=SpacerComp::space(ratio.clone(), width.clone(), height.clone()))]
    spaced: Component,
}

impl SpacerComp {
    fn space(
        ratio: F32Watchable,
        width: OptionF32Watchable,
        height: OptionF32Watchable,
    ) -> Component {
        CompositeItemComp::builder()
            .grow_ratio(ratio)
            .build(
                ContainerComp::builder()
                    .min_width(width)
                    .min_height(height)
                    .build(()),
            )
            .into()
    }
}

impl Into<Component> for SpacerComp {
    fn into(self) -> Component {
        self.spaced
    }
}
