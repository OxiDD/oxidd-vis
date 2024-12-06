use itertools::{EitherOrBoth, Itertools};

use crate::{
    types::util::drawing::diagram_layout::{LayerLayout, LayerStyle},
    util::transition::Transition,
};

pub fn transition_layers<LS: LayerStyle>(
    old: &Vec<LayerLayout<LS>>,
    new: &Vec<LayerLayout<LS>>,
    duration: u32,
    old_time: u32,
    time: u32,
) -> Vec<LayerLayout<LS>> {
    let mut out = Vec::new();

    let transition_out = |old_layer: &LayerLayout<LS>, out: &mut Vec<LayerLayout<LS>>| {
        let exists = old_layer.exists.get(time);
        if exists > 0. {
            out.push(LayerLayout {
                exists: Transition {
                    old_time,
                    duration,
                    old: exists,
                    new: 0.,
                },
                ..old_layer.clone()
            });
        }
    };

    let mut old_iter = old.iter().peekable();
    for new_layer in new {
        // Progress to the right old layer
        while let Some(&old_layer) = old_iter.peek() {
            if old_layer.exists.new >= 1. && old_layer.start_layer >= new_layer.start_layer {
                break;
            }
            old_iter.next();
            transition_out(&old_layer, &mut out);
        }

        // Try to transition from old to new
        if let Some(&old_layer) = old_iter.peek() {
            if old_layer.start_layer == new_layer.start_layer
                && old_layer.end_layer == new_layer.end_layer
            {
                old_iter.next();
                out.push(LayerLayout {
                    bottom: Transition {
                        old_time,
                        duration,
                        old: old_layer.bottom.get(time),
                        new: new_layer.bottom.new,
                    },
                    top: Transition {
                        old_time,
                        duration,
                        old: old_layer.top.get(time),
                        new: new_layer.top.new,
                    },
                    exists: Transition {
                        old_time,
                        duration,
                        old: old_layer.exists.get(time),
                        new: new_layer.exists.new,
                    },
                    index: Transition {
                        old_time,
                        duration,
                        old: old_layer.index.get(time),
                        new: new_layer.index.new,
                    },
                    ..new_layer.clone()
                });
                continue;
            }
        }

        // Otherwise insert new
        if old.len() == 0 {
            out.push(new_layer.clone()); // Don't transition in when there is no old
        } else {
            let center = (new_layer.bottom.old + new_layer.top.old) / 2.;
            out.push(LayerLayout {
                top: Transition {
                    old_time,
                    duration,
                    old: center,
                    new: new_layer.top.new,
                },
                bottom: Transition {
                    old_time,
                    duration,
                    old: center,
                    new: new_layer.bottom.new,
                },
                exists: Transition {
                    old_time,
                    duration,
                    old: 0.,
                    new: new_layer.exists.new,
                },
                ..new_layer.clone()
            });
        }
    }

    // Transition out any other old layers
    for old_layer in old_iter {
        transition_out(&old_layer, &mut out);
    }

    out
}

pub fn transition_layers_shift<LS: LayerStyle>(
    old: &Vec<LayerLayout<LS>>,
    new: &Vec<LayerLayout<LS>>,
    duration: u32,
    old_time: u32,
    time: u32,
) -> Vec<LayerLayout<LS>> {
    let prev_bottom = old.iter().last().map(|last_old| last_old.bottom.get(time));

    new.iter()
        .zip_longest(old.iter())
        .filter_map(|p| match p {
            EitherOrBoth::Both(new_layer, old_layer) => Some(LayerLayout {
                bottom: Transition {
                    old_time,
                    duration,
                    old: old_layer.bottom.get(time),
                    new: new_layer.bottom.new,
                },
                top: Transition {
                    old_time,
                    duration,
                    old: old_layer.top.get(time),
                    new: new_layer.top.new,
                },
                exists: Transition {
                    old_time,
                    duration,
                    old: old_layer.exists.get(time),
                    new: new_layer.exists.new,
                },
                ..new_layer.clone()
            }),
            EitherOrBoth::Left(new_layer) => Some(LayerLayout {
                top: Transition {
                    old_time,
                    duration,
                    old: prev_bottom.unwrap_or(new_layer.top.new),
                    new: new_layer.top.new,
                },
                bottom: Transition {
                    old_time,
                    duration,
                    old: prev_bottom.unwrap_or(new_layer.bottom.new),
                    new: new_layer.bottom.new,
                },
                exists: Transition {
                    old_time,
                    duration,
                    old: 0.,
                    new: new_layer.exists.new,
                },
                ..new_layer.clone()
            }),
            EitherOrBoth::Right(old_layer) => {
                let exists = old_layer.exists.get(time);
                if exists > 0.0 {
                    Some(old_layer.clone())
                } else {
                    None
                }
            }
        })
        .collect()
}
