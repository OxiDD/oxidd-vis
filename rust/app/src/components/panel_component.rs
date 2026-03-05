use std::rc::Rc;

use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::{
        button_component::ButtonComp, dyn_component::DynComp,
        panel_component::panel_comp_builder::SetOpenCount,
    },
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        make_typed_dyn_watchable, BoolWatchable, F32Watchable, OptionU32Watchable, StringField,
        StringWatchable, U32Watchable,
    },
};

/// Where and how a panel should open relative to its container.
#[wasm_bindgen]
#[derive(Clone)]
pub enum PanelOpenSide {
    In,
    North,
    South,
    East,
    West,
}
make_typed_dyn_watchable!(PanelOpenSideWatchable, PanelOpenSide);

/// When a panel should open automatically.
#[wasm_bindgen]
#[derive(Clone)]
pub enum PanelAutoOpen {
    Never,
    Always,
    /// Automatically open if the target panel is already present, and this only creates a new tab (but not new panel)
    IfExistingPanel,
}
make_typed_dyn_watchable!(PanelAutoOpenWatchable, PanelAutoOpen);

/// Panel component that configures and displays a child component in a separate panel.
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
#[builder(derive(Clone))]
pub struct PanelComp {
    /// The child component that is shown inside the panel.
    #[getter]
    #[builder(finish_fn, into)]
    content: DynComp,
    /// Stable identifier used for persistent layout state.
    #[getter]
    #[builder(into)]
    id: String,
    /// The display name shown for this panel.
    #[getter]
    name: StringField,
    /// The number of times the panel is requested to open (increase to open).
    #[getter]
    #[setter(u32)]
    open_count: U32Watchable,
    /// Category used to group this panel with others in the same area.
    #[getter]
    #[setter(String, "")]
    panel_category: StringWatchable,
    /// Which side this panel should open on.
    #[getter]
    #[setter(PanelOpenSide, PanelOpenSide::In)]
    open_side: PanelOpenSideWatchable,
    /// The number of ancestor panels to go up to open relative to.
    #[getter]
    #[setter(Option<u32>, Some(0))]
    relative_to_ancestor: OptionU32Watchable,
    /// Relative size this panel should open with compared to siblings.
    #[getter]
    #[setter(f32, 1.0)]
    open_ratio: F32Watchable,
    /// When this panel should open automatically.
    #[getter]
    #[setter(PanelAutoOpen, PanelAutoOpen::IfExistingPanel)]
    auto_open: PanelAutoOpenWatchable,
    // TODO: add grouping mechanics
}

impl Into<Component> for PanelComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::Panel(self))
    }
}

// A helper to store an unfinished panel builder
pub trait PartialPanelCompBuilder {
    fn build(&self, open_count: U32Watchable, content: Component) -> PanelComp;
}
impl<S> PartialPanelCompBuilder for PanelCompBuilder<S>
where
    S::OpenCount: panel_comp_builder::IsUnset,
    S::Name: panel_comp_builder::IsSet,
    S::Id: panel_comp_builder::IsSet,
    S: panel_comp_builder::State,
{
    fn build(&self, open_count: U32Watchable, content: Component) -> PanelComp {
        let builder = self.clone().open_count(open_count);
        PanelCompBuilder::<SetOpenCount<S>>::build(builder, content)
    }
}
impl<S: 'static> Into<Rc<dyn PartialPanelCompBuilder>> for PanelCompBuilder<S>
where
    S::OpenCount: panel_comp_builder::IsUnset,
    S::Name: panel_comp_builder::IsSet,
    S::Id: panel_comp_builder::IsSet,
    S: panel_comp_builder::State,
{
    fn into(self) -> Rc<dyn PartialPanelCompBuilder> {
        Rc::new(self)
    }
}
