use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::{
        container_component::container_comp_builder::{
            SetMarginBottom, SetMarginLeft, SetMarginRight, SetMarginTop, SetPaddingBottom,
            SetPaddingLeft, SetPaddingRight, SetPaddingTop, State,
        },
        dyn_component::DynComp,
    },
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{make_typed_dyn_watchable, F32Watchable, IntoWatchable, OptionF32Watchable},
};

/// Common UI color palette for use in container and component styling.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UIBackgroundColor {
    /// A very light neutral
    NeutralLight,
    /// A medium neutral
    NeutralMid,
    /// A dark neutral
    NeutralDark,
    /// Primary highlight color
    HighlightPrimary,
    /// Secondary highlight color
    HighlightSecondary,
}

make_typed_dyn_watchable!(OptionUIBackgroundColorWatchable, Option<UIBackgroundColor>);

/// Container component that applies background color, padding, margin, and minimum size.
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct ContainerComp {
    /// The content component rendered inside the container.
    #[getter]
    #[builder(finish_fn, into)]
    content: DynComp,
    /// Background color of the container.
    #[getter]
    #[setter(Option<UIBackgroundColor>)]
    background_color: OptionUIBackgroundColorWatchable,
    /// Padding on the top side.
    #[getter]
    #[setter(f32, 0.0)]
    padding_top: F32Watchable,
    /// Padding on the right side.
    #[getter]
    #[setter(f32, 0.0)]
    padding_right: F32Watchable,
    /// Padding on the bottom side.
    #[getter]
    #[setter(f32, 0.0)]
    padding_bottom: F32Watchable,
    /// Padding on the left side.
    #[getter]
    #[setter(f32, 0.0)]
    padding_left: F32Watchable,
    /// Margin on the top side.
    #[getter]
    #[setter(f32, 0.0)]
    margin_top: F32Watchable,
    /// Margin on the right side.
    #[getter]
    #[setter(f32, 0.0)]
    margin_right: F32Watchable,
    /// Margin on the bottom side.
    #[getter]
    #[setter(f32, 0.0)]
    margin_bottom: F32Watchable,
    /// Margin on the left side.
    #[getter]
    #[setter(f32, 0.0)]
    margin_left: F32Watchable,
    /// Optional minimum width of the container.
    #[getter]
    #[setter(Option<f32>)]
    min_width: OptionF32Watchable,
    /// Optional minimum height of the container.
    #[getter]
    #[setter(Option<f32>)]
    min_height: OptionF32Watchable,
}

impl<S: State> ContainerCompBuilder<S>
where
    S::MarginLeft: container_comp_builder::IsUnset,
    S::MarginRight: container_comp_builder::IsUnset,
    S::MarginBottom: container_comp_builder::IsUnset,
    S::MarginTop: container_comp_builder::IsUnset,
{
    /// Sets all margins (top, right, bottom, left) to the same value.
    pub fn margin(
        self,
        margin: impl IntoWatchable<f32> + Clone + 'static,
    ) -> ContainerCompBuilder<SetMarginBottom<SetMarginTop<SetMarginRight<SetMarginLeft<S>>>>> {
        self.margin_x(margin.clone()).margin_y(margin)
    }
}

impl<S: State> ContainerCompBuilder<S>
where
    S::MarginLeft: container_comp_builder::IsUnset,
    S::MarginRight: container_comp_builder::IsUnset,
{
    /// Sets the horizontal margins (left and right).
    pub fn margin_x(
        self,
        margin: impl IntoWatchable<f32> + Clone + 'static,
    ) -> ContainerCompBuilder<SetMarginRight<SetMarginLeft<S>>> {
        self.margin_left(margin.clone()).margin_right(margin)
    }
}

impl<S: State> ContainerCompBuilder<S>
where
    S::MarginBottom: container_comp_builder::IsUnset,
    S::MarginTop: container_comp_builder::IsUnset,
{
    /// Sets the vertical margins (top and bottom).
    pub fn margin_y(
        self,
        margin: impl IntoWatchable<f32> + Clone + 'static,
    ) -> ContainerCompBuilder<SetMarginBottom<SetMarginTop<S>>> {
        self.margin_top(margin.clone()).margin_bottom(margin)
    }
}

impl<S: State> ContainerCompBuilder<S>
where
    S::PaddingLeft: container_comp_builder::IsUnset,
    S::PaddingRight: container_comp_builder::IsUnset,
    S::PaddingBottom: container_comp_builder::IsUnset,
    S::PaddingTop: container_comp_builder::IsUnset,
{
    /// Sets all padding (top, right, bottom, left) to the same value.
    pub fn padding(
        self,
        padding: impl IntoWatchable<f32> + Clone + 'static,
    ) -> ContainerCompBuilder<SetPaddingBottom<SetPaddingTop<SetPaddingRight<SetPaddingLeft<S>>>>>
    {
        self.padding_x(padding.clone()).padding_y(padding)
    }
}

impl<S: State> ContainerCompBuilder<S>
where
    S::PaddingLeft: container_comp_builder::IsUnset,
    S::PaddingRight: container_comp_builder::IsUnset,
{
    /// Sets the horizontal padding (left and right).
    pub fn padding_x(
        self,
        padding: impl IntoWatchable<f32> + Clone + 'static,
    ) -> ContainerCompBuilder<SetPaddingRight<SetPaddingLeft<S>>> {
        self.padding_left(padding.clone()).padding_right(padding)
    }
}

impl<S: State> ContainerCompBuilder<S>
where
    S::PaddingBottom: container_comp_builder::IsUnset,
    S::PaddingTop: container_comp_builder::IsUnset,
{
    /// Sets the vertical padding (top and bottom).
    pub fn padding_y(
        self,
        padding: impl IntoWatchable<f32> + Clone + 'static,
    ) -> ContainerCompBuilder<SetPaddingBottom<SetPaddingTop<S>>> {
        self.padding_top(padding.clone()).padding_bottom(padding)
    }
}

impl Into<Component> for ContainerComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::Container(self))
    }
}
