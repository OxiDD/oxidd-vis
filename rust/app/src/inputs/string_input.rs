use std::rc::Rc;

use app_macros::{builder_into_comp, wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    impl_setter, impl_watchable,
    inputs::{
        string_input::string_input_comp_builder::SetWrapper,
        wrapper::{CompWrapper, IdentityWrapper, InputWrapper},
        InheritLabel, Inheritable, InheritedInput,
    },
    new_wasm_interface::{Component, ComponentOption},
    util::{
        logging::console,
        watchables::{
            signaller::Signaller, BoolWatchable, DynSignaller, DynWatchable, DynWatchableSetter,
            IntoWatchable, IntoWatchableSetter, MutateSetter, Mutator, OptionBoolWatchable,
            OptionU32Watchable, Setter, StringField, Watchable, WatchableSetter,
        },
    },
};

/// String input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct StringInput(StringField);
impl StringInput {
    pub fn new(val: String) -> Self {
        StringInput(StringField::new(val).into())
    }
    fn watchable(&self) -> &StringField {
        &self.0
    }
    fn setter(&mut self) -> &mut StringField {
        &mut self.0
    }
}
impl_watchable!(StringInput, String);
impl_setter!(StringInput, String);

impl Inheritable for InheritedInput<StringInput> {
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + 'static) -> Self {
        InheritedInput::new(
            StringInput::new((*self.get()).clone()),
            DynWatchable::new(self.clone()),
            self_name,
        )
    }
}
impl<X: Into<String>> From<X> for StringInput {
    fn from(value: X) -> Self {
        Self::new(value.into())
    }
}
impl<X: Into<String>> From<X> for InheritedInput<StringInput> {
    fn from(value: X) -> Self {
        Self::from(StringInput::from(value))
    }
}

/// StringInput component
#[wasm_getters]
#[wasm_bindgen]
#[builder_into_comp]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct StringInputComp {
    /// The data of the component
    #[builder(start_fn, into)]
    data: DynWatchableSetter<String>,
    /// Whether to allow multiline inputs
    #[getter]
    #[setter(bool, false)]
    multiline: BoolWatchable,
    /// The minimum number of lines to show
    #[getter]
    #[setter(Option<u32>)]
    multiline_min: OptionU32Watchable,
    /// The maximum number of lines to show
    #[getter]
    #[setter(Option<u32>)]
    multiline_max: OptionU32Watchable,
    /// Whether to dynamically adjust the shown number of lines based on content
    #[getter]
    #[setter(bool, false)]
    multiline_dynamic: BoolWatchable,
    /// Whether the user may resize the text field
    #[getter]
    #[setter(bool, false)]
    multiline_resizable: BoolWatchable,
    /// Whether the data should only be changed when the field is blurred, or enter/ctrl-enter is pressed
    #[getter]
    #[setter(bool, false)]
    late_submit: BoolWatchable,
    /// Whether this input is readonly
    #[getter]
    #[setter(bool, false)]
    readonly: BoolWatchable,
    /// Whether this input is disabled
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable,
    /// Wraps the output component
    #[builder(default=IdentityWrapper::new())]
    wrapper: Rc<dyn CompWrapper>,
}
impl StringInputComp {
    pub fn wrap_builder<I: Into<DynWatchableSetter<String>>>(
        wrapper: impl CompWrapper + InputWrapper<I> + Clone + 'static,
    ) -> StringInputCompBuilder<SetWrapper> {
        Self::builder(wrapper.get_input()).wrapper(Rc::new(wrapper))
    }
    fn watchable(&self) -> &DynWatchableSetter<String> {
        &self.data
    }
    fn setter(&mut self) -> &mut DynWatchableSetter<String> {
        &mut self.data
    }
}
impl_watchable!(StringInputComp, String);
impl_setter!(StringInputComp, String);

impl Into<StringInputComp> for StringInput {
    fn into(self) -> StringInputComp {
        StringInputComp::builder(self).build()
    }
}
impl Into<Component> for StringInput {
    fn into(self) -> Component {
        Into::<StringInputComp>::into(self).into()
    }
}
impl Into<Component> for StringInputComp {
    fn into(self) -> Component {
        self.wrapper
            .clone()
            .wrap(Component::new(ComponentOption::StringInput(self)))
    }
}
