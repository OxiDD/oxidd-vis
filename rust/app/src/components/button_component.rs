use std::{cell::RefCell, default};

use app_macros::{wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    components::text_component::TextComp,
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        signaller::Signaller, BoolWatchable, ClonableWatchableUtils, Constant, DataState,
        DynSignaller, DynWatchable, Field, IntoWatchable, JsListener, Listener, Mutator, Observer,
        OptionBoolWatchable, OptionStringWatchable, ReadonlyField, Setter, StringWatchable,
        U32Field, U32Watchable, Watchable, WatchableState,
    },
};

/// Button component.
#[wasm_getters]
#[wasm_bindgen]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct ButtonComp {
    #[builder(default=U32Field::new(0))]
    data: U32Field,
    /// The text to label the button with.
    #[getter]
    #[setter(Option<String>)]
    text: OptionStringWatchable,
    /// The icon to show in the button. See https://developer.microsoft.com/en-us/fluentui#/styles/web/icons for available icons.
    #[getter]
    #[setter(Option<String>)]
    icon: OptionStringWatchable,
    /// Whether this is a primary button.
    #[getter]
    #[setter(bool, false)]
    primary: BoolWatchable,
    /// Whether this input is disabled.
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable,
    /// The number of clicks that happened
    #[getter]
    #[builder(skip=data.read())]
    clicks: U32Watchable,
}

impl ButtonComp {
    pub fn new<W: IntoWatchable<String> + 'static>(text: W) -> Self {
        ButtonComp::builder()
            .text(text.into_watchable().option())
            .build()
    }
    #[must_use = "When the observer is dropped, observation automatically stops"]
    pub fn on_click<L: FnMut() -> () + 'static>(&self, listener: L) -> Observer {
        self.data.observe(Box::new(ButtonListener::new(listener)))
    }
    pub fn click(&mut self) -> DynSignaller {
        self.data.set(self.data.get() + 1)
    }
}
#[wasm_bindgen]
impl ButtonComp {
    #[must_use = "When the observer is dropped, observation automatically stops"]
    #[wasm_bindgen(js_name = onClick)]
    pub fn on_click_js(&self, on_click: js_sys::Function) -> Observer {
        self.data.observe(Box::new(JsListener::new(on_click, true)))
    }
    #[must_use = "Only once the mutator is committed, will the click be performed"]
    #[wasm_bindgen(js_name = click)]
    pub fn click_js(&mut self) -> Mutator {
        self.data.set_js(self.data.get() + 1)
    }
}

pub struct ButtonListener<L: FnMut() -> () + 'static>(RefCell<L>);
impl<L: FnMut() -> () + 'static> ButtonListener<L> {
    pub fn new(listener: L) -> Self {
        ButtonListener(RefCell::new(listener))
    }
}
impl<L: FnMut() -> () + 'static> Listener for ButtonListener<L> {
    fn state_changed(&self, state: DataState) {
        if state == DataState::UpToDate {
            (self.0.borrow_mut())()
        }
    }
}

impl Into<Component> for ButtonComp {
    fn into(self) -> Component {
        Component::new(ComponentOption::Button(self))
    }
}
impl<C: Into<TextComp>> From<C> for ButtonComp {
    fn from(value: C) -> Self {
        ButtonComp::new(value.into().text())
    }
}
