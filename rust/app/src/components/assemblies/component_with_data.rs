use std::any::Any;

use crate::{new_wasm_interface::Component, util::watchables::Derived};

/// A component combined with data, such that the resulting component will keep the data alive
pub struct ComponentWithData {
    component: Component,
    data: Box<dyn Any>,
}
impl ComponentWithData {
    pub fn new<D: 'static>(comp: impl Into<Component>, data: D) -> Self {
        ComponentWithData {
            component: comp.into(),
            data: Box::new(data),
        }
    }
}
impl Into<Component> for ComponentWithData {
    fn into(self) -> Component {
        let data = self.data;
        let component = self.component;
        Derived::new(move |_| {
            // Keep data alive until the component itself is dropped
            let _alive = &data;
            component.clone()
        })
        .into()
    }
}
