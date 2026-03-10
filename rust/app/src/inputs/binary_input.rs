use std::rc::Rc;

use app_macros::{builder_into_comp, wasm_getters, watchable_setters};
use bon::Builder;
use wasm_bindgen::prelude::*;

use crate::{
    impl_setter, impl_watchable,
    inputs::{
        binary_input::binary_input_comp_builder::SetWrapper,
        wrapper::{CompWrapper, IdentityWrapper, InputWrapper},
        InheritLabel, Inheritable, InheritedInput,
    },
    new_wasm_interface::{Component, ComponentOption},
    util::watchables::{
        signaller::Signaller, BoolWatchable, DynSignaller, DynWatchable, DynWatchableSetter, Field,
        IntoWatchable, Mutator, OptionStringField, OptionStringWatchable, Setter, StringWatchable,
    },
};

#[wasm_bindgen]
#[derive(Clone)]
pub struct FileData {
    data: Vec<u8>,
    name: String,
}
#[wasm_bindgen]
impl FileData {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, data: Vec<u8>) -> FileData {
        FileData { name, data }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

/// Binary input data
#[derive(Clone)]
#[wasm_bindgen]
pub struct BinaryInput(Field<Option<FileData>>);
impl BinaryInput {
    pub fn new() -> Self {
        BinaryInput(Field::new(None))
    }
    fn watchable(&self) -> &Field<Option<FileData>> {
        &self.0
    }
    fn setter(&mut self) -> &mut Field<Option<FileData>> {
        &mut self.0
    }
}
impl_watchable!(BinaryInput, Option<FileData>);
impl_setter!(BinaryInput, Option<FileData>);

impl Inheritable for InheritedInput<BinaryInput> {
    fn inherit(&self, self_name: impl IntoWatchable<InheritLabel> + 'static) -> Self {
        InheritedInput::new(
            BinaryInput::new(),
            DynWatchable::new(self.clone()),
            self_name,
        )
    }
}
impl<X: Into<Option<FileData>>> From<X> for BinaryInput {
    fn from(value: X) -> Self {
        BinaryInput(Field::new(value.into()))
    }
}
impl<X: Into<Option<FileData>>> From<X> for InheritedInput<BinaryInput> {
    fn from(value: X) -> Self {
        InheritedInput::from(BinaryInput::from(value))
    }
}

/// BinaryInput component
#[wasm_getters]
#[wasm_bindgen]
#[builder_into_comp]
#[watchable_setters]
#[derive(Builder, Clone)]
pub struct BinaryInputComp {
    /// The data of the component
    #[builder(start_fn, into)]
    data: DynWatchableSetter<Option<FileData>>,
    /// A comma separated list of extensions
    #[getter]
    #[setter(Option<String>)]
    formats: OptionStringWatchable,
    /// Whether the user should be able to remove the file
    #[getter]
    #[setter(bool, false)]
    allow_unset: BoolWatchable,
    /// Whether this input is disabled
    #[getter]
    #[setter(bool, false)]
    disabled: BoolWatchable,
    /// Wraps the output component
    #[builder(default=IdentityWrapper::new())]
    wrapper: Rc<dyn CompWrapper>,
}
impl BinaryInputComp {
    pub fn wrap_builder<I: Into<DynWatchableSetter<Option<FileData>>>>(
        wrapper: impl CompWrapper + InputWrapper<I> + Clone + 'static,
    ) -> BinaryInputCompBuilder<SetWrapper> {
        Self::builder(wrapper.get_input()).wrapper(Rc::new(wrapper))
    }
    fn watchable(&self) -> &DynWatchableSetter<Option<FileData>> {
        &self.data
    }
    fn setter(&mut self) -> &mut DynWatchableSetter<Option<FileData>> {
        &mut self.data
    }
}
impl_watchable!(BinaryInputComp, Option<FileData>);
impl_setter!(BinaryInputComp, Option<FileData>);

impl Into<BinaryInputComp> for BinaryInput {
    fn into(self) -> BinaryInputComp {
        BinaryInputComp::builder(self).build()
    }
}
impl Into<Component> for BinaryInput {
    fn into(self) -> Component {
        Into::<BinaryInputComp>::into(self).into()
    }
}
impl Into<Component> for BinaryInputComp {
    fn into(self) -> Component {
        let wrapper = self.wrapper.clone();
        let comp = Component::new(ComponentOption::BinaryInput(self));
        wrapper.wrap(comp)
    }
}
