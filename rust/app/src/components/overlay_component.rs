use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::{
        dyn_component::{ComponentWatchable, DynComp},
        overlay_component::overlay_comp_builder::{SetX, SetY},
    },
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{Constant, F32Watchable, Field, IntoWatchable},
};

/// Overlay component that renders one component over another.
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct OverlayComp {
    /// The content of the overlay.
    #[getter]
    #[builder(finish_fn, into)]
    content: DynComp,
    /// Horizontal offset from the origin.
    #[getter]
    #[setter(f32, 0.0)]
    x: F32Watchable,
    /// Vertical offset from the origin.
    #[getter]
    #[setter(f32, 0.0)]
    y: F32Watchable,
}

macro_rules! position_setter {
    ($name:ident, $x:tt, $y:tt) => {
        impl<S: overlay_comp_builder::State> OverlayCompBuilder<S>
        where
            S::X: overlay_comp_builder::IsUnset,
            S::Y: overlay_comp_builder::IsUnset,
        {
            pub fn $name(self) -> OverlayCompBuilder<SetX<SetY<S>>> {
                self.y($y).x($x)
            }
        }

        impl OverlayComp {
            pub fn $name(main: impl Into<DynComp> + 'static) -> Self {
                Self::builder().$name().build(main)
            }
        }
    };
}
position_setter!(top_left, 0.0, 0.0);
position_setter!(top, 0.5, 0.0);
position_setter!(top_right, 1.0, 0.0);
position_setter!(left, 0.0, 0.5);
position_setter!(center, 0.5, 0.5);
position_setter!(right, 1.0, 0.5);
position_setter!(bottom_left, 0.0, 1.0);
position_setter!(bottom, 0.5, 1.0);
position_setter!(bottom_right, 1.0, 1.0);

impl Into<Component> for OverlayComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::Overlay(self))
    }
}
