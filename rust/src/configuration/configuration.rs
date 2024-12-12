use std::ops::Deref;

use crate::util::{free_id_manager::FreeIdManager, rc_refcell::MutRcRefCell};

use super::configuration_object::{AbstractConfigurationObject, Abstractable};

pub struct Configuration<C: Abstractable> {
    config: C,
}

impl<C: Abstractable> Configuration<C> {
    pub fn new(config: C) -> Configuration<C> {
        Configuration { config }
    }
}
// impl Deref for Configuration<>

impl<C: Abstractable> Abstractable for Configuration<C> {
    fn get_abstract(&self) -> AbstractConfigurationObject {
        self.config.get_abstract()
    }
}

impl<C: Abstractable> Drop for Configuration<C> {
    fn drop(&mut self) {
        self.config.get_abstract().dispose()
    }
}
