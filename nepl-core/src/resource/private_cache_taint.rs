extern crate alloc;

use alloc::vec::Vec;

use crate::effects::PrivateEffectRegion;

use super::effect_place_prefix::{
    groups_overlap, place_has_prefix, push_unique_place, push_unique_places, replace_place_prefix,
};
use super::model::{AggregateKind, Place};
use super::place_utils::construct_aggregate_field_place;

/// Resource IR 内で sealed private cache region に由来する値を追跡する表。
///
/// この表は `PrivateCache` を `Pure` へ mask する proof そのものではない。cache handle、
/// reference、raw pointer、owner token などが public result へ到達しないことを証明するための
/// 入力であり、proof 発行側はこの追跡結果に加えて hit/miss、stats、clear、lookup result の
/// owned/copy 性を別途確認する必要がある。
#[derive(Debug, Clone, Default)]
pub(super) struct PrivateCacheRegionTaintTable {
    groups: Vec<PrivateCacheRegionTaintGroup>,
}

#[derive(Debug, Clone)]
struct PrivateCacheRegionTaintGroup {
    places: Vec<Place>,
    regions: Vec<PrivateEffectRegion>,
}

impl PrivateCacheRegionTaintTable {
    pub(super) fn mark(&mut self, place: &Place, region: PrivateEffectRegion) {
        if !region.is_sealed() {
            return;
        }
        self.union_group(core::slice::from_ref(place), core::slice::from_ref(&region));
    }

    pub(super) fn regions(&self, place: &Place) -> Vec<PrivateEffectRegion> {
        let mut regions = Vec::new();
        for group in &self.groups {
            if group
                .places
                .iter()
                .any(|existing| places_overlap(existing, place))
            {
                push_unique_regions(&mut regions, &group.regions);
            }
        }
        regions
    }

    pub(super) fn copy_taint(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let groups = self.groups_with_replaced_prefix(source, target, true);
        self.clear(target);
        for group in groups {
            self.union_group(&group.places, &group.regions);
        }
    }

    pub(super) fn move_taint(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let groups = self.groups_with_replaced_prefix(source, target, true);
        self.clear(target);
        self.clear(source);
        for mut group in groups {
            group
                .places
                .retain(|place| !place_has_prefix(place, source));
            self.union_group(&group.places, &group.regions);
        }
    }

    pub(super) fn merge_taint(&mut self, source: &Place, target: &Place) {
        for group in self.groups_with_replaced_prefix(source, target, false) {
            self.union_group(&group.places, &group.regions);
        }
    }

    pub(super) fn clear(&mut self, place: &Place) {
        for group in &mut self.groups {
            group
                .places
                .retain(|existing| !place_has_prefix(existing, place));
        }
        self.groups.retain(|group| !group.places.is_empty());
    }

    pub(super) fn merge_paths(paths: &[PrivateCacheRegionTaintTable]) -> Self {
        let mut out = PrivateCacheRegionTaintTable::default();
        for path in paths {
            for group in &path.groups {
                out.union_group(&group.places, &group.regions);
            }
        }
        out
    }

    fn groups_with_replaced_prefix(
        &self,
        source: &Place,
        target: &Place,
        drop_target_prefix: bool,
    ) -> Vec<PrivateCacheRegionTaintGroup> {
        let mut out = Vec::new();
        for group in &self.groups {
            let mut mapped = Vec::new();
            for place in &group.places {
                if let Some(replacement) = replace_place_prefix(place, source, target) {
                    push_unique_place(&mut mapped, replacement);
                }
            }
            if mapped.is_empty() {
                continue;
            }

            let mut merged = if drop_target_prefix {
                group
                    .places
                    .iter()
                    .filter(|place| !place_has_prefix(place, target))
                    .cloned()
                    .collect()
            } else {
                group.places.clone()
            };
            push_unique_places(&mut merged, &mapped);
            out.push(PrivateCacheRegionTaintGroup {
                places: merged,
                regions: group.regions.clone(),
            });
        }
        out
    }

    fn union_group(&mut self, places: &[Place], regions: &[PrivateEffectRegion]) {
        let mut merged_places = places.to_vec();
        let mut merged_regions = regions.to_vec();
        let mut retained = Vec::new();
        for existing in self.groups.drain(..) {
            if groups_overlap(&existing.places, &merged_places) {
                push_unique_places(&mut merged_places, &existing.places);
                push_unique_regions(&mut merged_regions, &existing.regions);
            } else {
                retained.push(existing);
            }
        }
        if !merged_places.is_empty() && !merged_regions.is_empty() {
            retained.push(PrivateCacheRegionTaintGroup {
                places: merged_places,
                regions: merged_regions,
            });
        }
        self.groups = retained;
    }
}

pub(super) fn construct_private_cache_taint_fields(
    taints: &mut PrivateCacheRegionTaintTable,
    output: &Place,
    kind: &AggregateKind,
    inputs: &[Place],
) {
    taints.clear(output);
    for (index, input) in inputs.iter().enumerate() {
        let field = construct_aggregate_field_place(output, kind, index, input);
        taints.merge_taint(input, &field);
    }
}

fn places_overlap(left: &Place, right: &Place) -> bool {
    place_has_prefix(left, right) || place_has_prefix(right, left)
}

fn push_unique_regions(target: &mut Vec<PrivateEffectRegion>, source: &[PrivateEffectRegion]) {
    for region in source {
        if !target.contains(region) {
            target.push(*region);
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use crate::effects::{PrivateEffectRegion, PrivateEffectRegionId};
    use crate::resource::model::{AggregateKind, Place, PlaceProjection};
    use crate::types::TypeId;

    use super::{construct_private_cache_taint_fields, PrivateCacheRegionTaintTable};

    #[test]
    fn taint_table_tracks_exact_sealed_region_through_copy_and_move() {
        let region = PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(3));
        let source = Place::local(String::from("cache"), TypeId(0));
        let copied = Place::local(String::from("copied"), TypeId(0));
        let moved = Place::local(String::from("moved"), TypeId(0));

        let mut taints = PrivateCacheRegionTaintTable::default();
        taints.mark(&source, region);
        taints.copy_taint(&source, &copied);
        taints.move_taint(&copied, &moved);

        assert_eq!(taints.regions(&source), alloc::vec![region]);
        assert!(taints.regions(&copied).is_empty());
        assert_eq!(taints.regions(&moved), alloc::vec![region]);
    }

    #[test]
    fn taint_table_ignores_unsealed_intrinsic_region() {
        let source = Place::local(String::from("cache"), TypeId(0));
        let mut taints = PrivateCacheRegionTaintTable::default();

        taints.mark(&source, PrivateEffectRegion::UnsealedIntrinsic);

        assert!(taints.regions(&source).is_empty());
    }

    #[test]
    fn aggregate_taint_is_visible_from_the_whole_output_place() {
        let region = PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(4));
        let input = Place::local(String::from("cache"), TypeId(0));
        let output = Place::local(String::from("pair"), TypeId(1));
        let field = output.clone().with_projection(
            PlaceProjection::TupleField {
                index: 0,
                offset_bytes: 0,
            },
            TypeId(0),
        );

        let mut taints = PrivateCacheRegionTaintTable::default();
        taints.mark(&input, region);
        construct_private_cache_taint_fields(
            &mut taints,
            &output,
            &AggregateKind::Tuple {
                field_offsets: alloc::vec![0],
            },
            core::slice::from_ref(&input),
        );

        assert_eq!(taints.regions(&field), alloc::vec![region]);
        assert_eq!(taints.regions(&output), alloc::vec![region]);
    }
}
