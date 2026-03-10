use std::rc::Rc;

use app_macros::{builder_into_comp, wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    impl_setter, impl_watchable,
    inputs::{
        bool_input::bool_input_comp_builder::SetWrapper,
        wrapper::{CompWrapper, IdentityWrapper, InputWrapper},
        InheritLabel, InheritLabelWatchable, Inheritable, InheritedInput,
    },
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        signaller::Signaller, BoolField, BoolWatchable, DynSignaller, DynWatchable,
        DynWatchableSetter, IntoWatchable, MutateSetter, Mutator, Setter, StringWatchable,
        Watchable,
    },
};

/// Boolean input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct BoolInput(BoolField);
impl BoolInput {
    pub fn new(val: bool) -> Self {
        BoolInput(BoolField::new(val))
    }
    fn watchable(&self) -> &BoolField {
        &self.0
    }
    fn setter(&mut self) -> &mut BoolField {
        &mut self.0
    }
    pub fn toggle(&mut self) -> DynSignaller {
        self.0.set(!self.0.get())
    }
}
impl_watchable!(BoolInput, bool);
impl_setter!(BoolInput, bool);

impl Inheritable for InheritedInput<BoolInput> {
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + 'static) -> Self {
        InheritedInput::new(
            BoolInput::new(*self.get()),
            DynWatchable::new(self.clone()),
            self_name,
        )
    }
}
impl<X: Into<bool>> From<X> for BoolInput {
    fn from(value: X) -> Self {
        Self::new(value.into())
    }
}
impl<X: Into<bool>> From<X> for InheritedInput<BoolInput> {
    fn from(value: X) -> Self {
        Self::from(BoolInput::from(value))
    }
}

/// BoolInput component
#[wasm_getters]
#[wasm_bindgen]
#[builder_into_comp]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct BoolInputComp {
    /// The data of the component
    #[builder(start_fn, into)]
    data: DynWatchableSetter<bool>,
    /// Whether this input is disabled
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable,
    /// Wraps the output component
    #[builder(default=IdentityWrapper::new())]
    wrapper: Rc<dyn CompWrapper>,
}
impl BoolInputComp {
    pub fn wrap_builder<I: Into<DynWatchableSetter<bool>>>(
        wrapper: impl CompWrapper + InputWrapper<I> + Clone + 'static,
    ) -> BoolInputCompBuilder<SetWrapper> {
        Self::builder(wrapper.get_input()).wrapper(Rc::new(wrapper))
    }
    fn watchable(&self) -> &DynWatchableSetter<bool> {
        &self.data
    }
    fn setter(&mut self) -> &mut DynWatchableSetter<bool> {
        &mut self.data
    }
}
impl_watchable!(BoolInputComp, bool);
impl_setter!(BoolInputComp, bool);

impl Into<BoolInputComp> for BoolInput {
    fn into(self) -> BoolInputComp {
        BoolInputComp::builder(self).build()
    }
}

impl Into<Component> for BoolInput {
    fn into(self) -> Component {
        Into::<BoolInputComp>::into(self).into()
    }
}
impl Into<Component> for BoolInputComp {
    fn into(self) -> Component {
        let wrapper = self.wrapper.clone();
        let comp = Component::new(ComponentOption::BoolInput(self));
        wrapper.wrap(comp)
    }
}
