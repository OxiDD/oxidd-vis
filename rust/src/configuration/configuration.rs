use std::ops::{Deref, DerefMut};

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
impl<C: Abstractable> Deref for Configuration<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}
impl<C: Abstractable> DerefMut for Configuration<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

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
