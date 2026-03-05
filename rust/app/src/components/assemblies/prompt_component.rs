use std::cell::RefCell;

use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::{
        AlignMain, ButtonComp, ButtonListener, CompositeComp, ContainerComp, ModalComp, dyn_component::DynComp
    },
    new_wasm_interface::Component,
    util::watchables::{
        BoolField, BoolWatchable, Changed, Derived, Observer, OptionF32Watchable, StringWatchable, U32Field, U32Watchable, WatchableState, Watching
    },
};

/// Assembly component representing a generic prompt shown inside a modal.
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct PromptComp {
    /// The content rendered inside the prompt (e.g. text, inputs, buttons).
    #[getter]
    #[builder(finish_fn, into)]
    content: DynComp,
    /// Whether the prompt is currently visible.
    #[getter]
    #[setter(u32)]
    open_count: U32Watchable,
    /// Optional width of the prompt modal.
    #[getter]
    #[setter(Option<f32>)]
    width: OptionF32Watchable,
    /// Optional height of the prompt modal.
    #[getter]
    #[setter(Option<f32>)]
    height: OptionF32Watchable,
    /// Cancel text
    #[getter]
    #[setter(String, "cancel")]
    cancel_text: StringWatchable,
    /// Submit text
    #[getter]
    #[setter(String, "ok")]
    submit_text: StringWatchable,
    /// The number of times submitted
    #[getter]
    #[builder(skip=U32Field::new(0))]
    submit_count: U32Field,
    /// The number of times canceled
    #[getter]
    #[builder(skip=U32Field::new(0))]
    cancel_count: U32Field,
    /// The underlying modal component used to display the prompt.
    #[getter]
    #[builder(skip=PromptComp::make_modal(
        content.clone(), 
        open_count.clone(), 
        submit_count.clone(), 
        cancel_count.clone(),
        width.clone(), 
        height.clone()
    ))]
    modal: ModalComp,
}

impl PromptComp {
    fn make_modal(
        content: DynComp,
        open_count: U32Watchable,
        submit_count: U32Field,
        cancel_count: U32Field,
        width: OptionF32Watchable,
        height: OptionF32Watchable,
    ) -> ModalComp {

        let submit = ButtonComp::builder().primary(true).data(submit_count).text("Submit").build();
        let close = ButtonComp::builder().data(cancel_count).text("Cancel").build();

        let click_outside_count = U32Field::new(0);

        let shown = {
            let open_count_changed = Changed::new(open_count);
            
            let (submit_clicks, close_clicks, click_outside_count) = (submit.clicks(), close.clicks(), click_outside_count.clone());
            Derived::new(move |t| {
                let opened = open_count_changed.watch(t);

                close_clicks.watch(t);
                submit_clicks.watch(t);
                click_outside_count.watch(t);
                
                *opened
            })
        };

        let buttons = CompositeComp::builder()
            .gap(1.0)
            .horizontal(true)
            .main_align(AlignMain::End)
            .build((close, submit));
        let content = CompositeComp::builder().gap(1.0).build((content.into_component(), buttons));
        let container = ContainerComp::builder().margin(1.0).build(content);
        ModalComp::builder()
            .shown(shown)
            .click_outside(click_outside_count)
            .width(width)
            .height(height)
            .build(container)
    }
}

impl PromptComp {
    pub fn on_submit<L: FnMut() -> () + 'static>(&self, listener: L) -> Observer {
        self.submit_count
            .observe(Box::new(ButtonListener::new(listener)))
    }
    pub fn on_cancel<L: FnMut() -> () + 'static>(&self, listener: L) -> Observer {
        self.cancel_count
            .observe(Box::new(ButtonListener::new(listener)))
    }
}

impl Into<Component> for PromptComp {
    fn into(self) -> Component {
        self.modal.into()
    }
}
