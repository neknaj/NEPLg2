extern crate alloc;

use alloc::vec::Vec;

use super::model::{Place, PlaceRoot};
use super::place_utils::{place_suffix_after_prefix, place_with_suffix};

#[derive(Debug, Clone, Default)]
pub(super) struct RawValueOrigins {
    origins: Vec<ValueOrigin>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValueOrigin {
    place: Place,
    origin: Place,
}

impl RawValueOrigins {
    pub(super) fn record_view_origin(&mut self, source: &Place, target: &Place) {
        let origin = self.origin_for(source);
        self.set(target, &origin);
    }

    pub(super) fn copy_stable_origin(&mut self, source: &Place, target: &Place) {
        if !value_origin_copy_is_relevant(source, target) {
            return;
        }
        let resolved_origin = self.origin_for(source);
        let origin = if value_origin_place_is_stable(&resolved_origin) {
            Some(resolved_origin)
        } else if value_origin_place_is_stable(source) {
            Some(source.clone())
        } else {
            None
        };
        if let Some(origin) = origin {
            self.set(target, &origin);
        }
    }

    pub(super) fn origin_for(&self, place: &Place) -> Place {
        let mut current = place.clone();
        for _ in 0..=self.origins.len() {
            let Some(next) = self.step(&current) else {
                break;
            };
            if next == current {
                break;
            }
            current = next;
        }
        current
    }

    pub(super) fn clear_prefix(&mut self, place: &Place) {
        self.origins.retain(|origin| {
            place_suffix_after_prefix(&origin.place, place).is_none()
                && place_suffix_after_prefix(&origin.origin, place).is_none()
        });
    }

    pub(super) fn merge_paths<'a>(
        paths: impl IntoIterator<Item = &'a RawValueOrigins>,
    ) -> RawValueOrigins {
        let paths = paths.into_iter().collect::<Vec<_>>();
        let mut out = RawValueOrigins::default();
        if let Some(first) = paths.first() {
            for origin in &first.origins {
                if paths
                    .iter()
                    .skip(1)
                    .all(|path| path.origins.iter().any(|existing| existing == origin))
                {
                    out.set(&origin.place, &origin.origin);
                }
            }
        }
        out
    }

    fn step(&self, place: &Place) -> Option<Place> {
        self.origins
            .iter()
            .filter_map(|origin| {
                place_suffix_after_prefix(place, &origin.place)
                    .map(|suffix| place_with_suffix(&origin.origin, &suffix, place.ty))
            })
            .min_by_key(|candidate| candidate.projections.len())
    }

    fn set(&mut self, place: &Place, origin: &Place) {
        self.origins.retain(|existing| existing.place != *place);
        if place == origin
            || !value_origin_place_is_stable(place) && !value_origin_place_is_stable(origin)
        {
            return;
        }
        self.origins.push(ValueOrigin {
            place: place.clone(),
            origin: origin.clone(),
        });
    }
}

fn value_origin_copy_is_relevant(source: &Place, target: &Place) -> bool {
    value_origin_place_is_stable(source) || value_origin_place_is_stable(target)
}

fn value_origin_place_is_stable(place: &Place) -> bool {
    matches!(
        place.root,
        PlaceRoot::Local(_) | PlaceRoot::Return | PlaceRoot::Storage(_)
    )
}
