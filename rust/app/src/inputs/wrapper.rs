use crate::new_wasm_interface::Component;
use std::rc::Rc;

pub trait CompWrapper {
    fn wrap(&self, comp: Component) -> Component;
}
pub trait InputWrapper<Input> {
    fn get_input(&self) -> Input;
}

pub struct IdentityWrapper;
impl IdentityWrapper {
    pub fn new() -> Rc<Self> {
        Rc::new(IdentityWrapper)
    }
}
impl CompWrapper for IdentityWrapper {
    fn wrap(&self, comp: Component) -> Component {
        comp
    }
}
