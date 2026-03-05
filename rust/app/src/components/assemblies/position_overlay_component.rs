use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::{dyn_component::DynComp, Align, CompositeComp, FillComp, OverlayComp, SpacerComp},
    new_wasm_interface::{Component, ComponentOption},
    util::{
        logging::console,
        watchables::{F32Watchable, WatchableUtils},
    },
};

/// Component that positions a child component using x/y offsets.
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct PositionOverlayComp {
    /// The main content to put an overlay over.
    #[getter]
    #[builder(finish_fn, into)]
    main: DynComp,
    /// The content component rendered at the given position.
    #[getter]
    #[builder(into)]
    content: DynComp,
    /// Horizontal offset from the origin.
    #[getter]
    #[setter(f32, 0.0)]
    x: F32Watchable,
    /// Vertical offset from the origin.s
    #[getter]
    #[setter(f32, 0.0)]
    y: F32Watchable,
    /// The positioned component
    #[getter]
    #[builder(skip=PositionOverlayComp::position(main.clone(), content.clone(), x.clone(), y.clone()))]
    positioned: OverlayComp,
}

impl PositionOverlayComp {
    pub fn position(
        main: DynComp,
        content: DynComp,
        x: F32Watchable,
        y: F32Watchable,
    ) -> OverlayComp {
        console::log!("{} {}", x.get(), y.get());
        let horizontal = CompositeComp::builder().horizontal(true).build((
            SpacerComp::builder().ratio(x.clone()).build(),
            content.into_component(),
            SpacerComp::builder().ratio(x.map(|v| 1.0 - *v)).build(),
        ));
        let vertical = CompositeComp::builder()
            .perpendicular_align(Align::Stretch)
            .horizontal(false)
            .build((
                SpacerComp::builder().ratio(y.clone()).build(),
                horizontal,
                SpacerComp::builder().ratio(y.map(|v| 1.0 - *v)).build(),
            ));
        OverlayComp::builder()
            .overlay(FillComp::new(vertical))
            .build(main)
    }
}

impl PositionOverlayComp {
    pub fn top_left(
        overlay: impl Into<DynComp> + 'static,
        child: impl Into<DynComp> + 'static,
    ) -> Self {
        PositionOverlayComp::builder()
            .x(0.0)
            .y(0.0)
            .content(overlay)
            .build(child)
    }
    pub fn top_center(
        overlay: impl Into<DynComp> + 'static,
        child: impl Into<DynComp> + 'static,
    ) -> Self {
        PositionOverlayComp::builder()
            .x(0.5)
            .y(0.0)
            .content(overlay)
            .build(child)
    }
    pub fn top_right(
        overlay: impl Into<DynComp> + 'static,
        child: impl Into<DynComp> + 'static,
    ) -> Self {
        PositionOverlayComp::builder()
            .x(1.0)
            .y(0.0)
            .content(overlay)
            .build(child)
    }
    pub fn center_left(
        overlay: impl Into<DynComp> + 'static,
        child: impl Into<DynComp> + 'static,
    ) -> Self {
        PositionOverlayComp::builder()
            .x(0.0)
            .y(0.5)
            .content(overlay)
            .build(child)
    }

    pub fn center(
        overlay: impl Into<DynComp> + 'static,
        child: impl Into<DynComp> + 'static,
    ) -> Self {
        PositionOverlayComp::builder()
            .x(0.5)
            .y(0.5)
            .content(overlay)
            .build(child)
    }
    pub fn center_right(
        overlay: impl Into<DynComp> + 'static,
        child: impl Into<DynComp> + 'static,
    ) -> Self {
        PositionOverlayComp::builder()
            .x(1.0)
            .y(0.5)
            .content(overlay)
            .build(child)
    }
    pub fn bottom_left(
        overlay: impl Into<DynComp> + 'static,
        child: impl Into<DynComp> + 'static,
    ) -> Self {
        PositionOverlayComp::builder()
            .x(0.0)
            .y(1.0)
            .content(overlay)
            .build(child)
    }
    pub fn bottom_center(
        overlay: impl Into<DynComp> + 'static,
        child: impl Into<DynComp> + 'static,
    ) -> Self {
        PositionOverlayComp::builder()
            .x(0.5)
            .y(1.0)
            .content(overlay)
            .build(child)
    }
    pub fn bottom_right(
        overlay: impl Into<DynComp> + 'static,
        child: impl Into<DynComp> + 'static,
    ) -> Self {
        PositionOverlayComp::builder()
            .x(1.0)
            .y(1.0)
            .content(overlay)
            .build(child)
    }
}

impl Into<Component> for PositionOverlayComp {
    fn into(self) -> Component {
        self.positioned.into()
    }
}
