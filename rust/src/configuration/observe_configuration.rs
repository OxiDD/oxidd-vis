use crate::util::rc_refcell::MutRcRefCell;

use super::configuration_object::{AbstractConfigurationObject, Abstractable};

pub fn after_configuration_change<C: Abstractable, F: FnMut() -> () + 'static>(
    config: &C,
    after_change: F,
) -> impl FnOnce() -> () {
    observe_configuration(config, after_change, false, true)
}

pub fn on_configuration_change<C: Abstractable, F: FnMut() -> () + 'static>(
    config: &C,
    after_change: F,
) -> impl FnOnce() -> () {
    observe_configuration(config, after_change, true, false)
}

pub fn observe_configuration<C: Abstractable, F: FnMut() -> () + 'static>(
    config: &C,
    on_change: F,
    init: bool,
    // Whether to invoke after a change finished, instead of as soon as it happens (for batching)
    after_change: bool,
) -> impl FnOnce() -> () {
    let on_change = MutRcRefCell::new(on_change);
    let mut remove_dirty_ids = Vec::<(AbstractConfigurationObject, usize)>::new();
    let mut remove_change_ids = Vec::<(AbstractConfigurationObject, usize)>::new();
    let dirty = MutRcRefCell::new(false);

    // Run through all elements in the config, and setup a listener for each
    let c = config.get_abstract();
    let mut queue = vec![c];
    while let Some(mut config_el) = queue.pop() {
        let local_dirty = dirty.clone();
        let local_after_change = on_change.clone();
        remove_dirty_ids.push((
            config_el.clone(),
            config_el.add_dirty_listener(move || {
                if !*local_dirty.get() {
                    *local_dirty.get() = true;

                    if !after_change {
                        (local_after_change.get())();
                    }
                }
            }),
        ));

        let local_dirty = dirty.clone();
        let local_after_change = on_change.clone();
        remove_change_ids.push((
            config_el.clone(),
            config_el.add_change_listener(move || {
                if *local_dirty.get() {
                    *local_dirty.get() = false;

                    if after_change {
                        (local_after_change.get())();
                    }
                }
            }),
        ));

        for child in config_el.get_children() {
            queue.push(child);
        }
    }

    // Init if needed
    if init {
        (on_change.get())();
    }

    // Cleanup function
    move || {
        for (mut config_el, id) in remove_dirty_ids {
            config_el.remove_dirty_listener(id);
        }
        for (mut config_el, id) in remove_change_ids {
            config_el.remove_change_listener(id);
        }
    }
}
