extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::initialized_alias_difference::I32DifferenceFacts;
use super::initialized_alias_host_size::HostSizeFacts;
use super::initialized_alias_offset::I32OffsetFacts;
use super::initialized_alias_origin::RawValueOrigins;
use super::initialized_alias_rank::{
    owner_alias_place_has_raw_projection, owner_cell_alias_rank, prefer_stable_canonical,
};
use super::initialized_alias_relation::I32RelationFacts;
use super::initialized_alias_scalar::I32AliasFacts;
use super::initialized_alias_scalar_copy::ScalarFactCopies;
use super::initialized_alias_scale::I32ScaleFacts;
use super::initialized_alias_type_size::I32TypeSizeFacts;
use super::initialized_alias_utils::{groups_overlap, push_unique_projected_alias};
use super::model::Place;
use super::place_utils::{
    place_suffix_after_prefix, place_with_suffix, push_unique_place, replace_place_prefix,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProjectedRawCellAddressAlias {
    pub(super) left_projection: Vec<super::model::PlaceProjection>,
    pub(super) left_ty: TypeId,
    pub(super) right_projection: Vec<super::model::PlaceProjection>,
    pub(super) right_ty: TypeId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct RawCellAddressAliases {
    groups: Vec<Vec<Place>>,
    marked: Vec<Place>,
    raw_view_origins: RawValueOrigins,
    scalar_origins: RawValueOrigins,
    pub(super) i32_facts: I32AliasFacts,
    pub(super) i32_differences: I32DifferenceFacts,
    pub(super) i32_relations: I32RelationFacts,
    pub(super) i32_scales: I32ScaleFacts,
    pub(super) i32_offsets: I32OffsetFacts,
    pub(super) i32_type_sizes: I32TypeSizeFacts,
    pub(super) host_size_facts: HostSizeFacts,
}

impl RawCellAddressAliases {
    pub(super) fn mark(&mut self, place: &Place) {
        self.clear(place);
        push_unique_place(&mut self.marked, place);
        self.union_group(core::slice::from_ref(place));
    }

    pub(super) fn ensure_marked(&mut self, place: &Place) {
        push_unique_place(&mut self.marked, place);
        self.union_group(core::slice::from_ref(place));
    }

    /// Copies an existing raw-address alias group and scalar facts without treating ordinary
    /// i32 value copies as raw-address aliases.
    pub(super) fn copy_alias_if_tracked(&mut self, source: &Place, target: &Place) {
        self.copy_alias_if_tracked_with_mode(source, target, true);
    }

    pub(super) fn copy_alias_if_tracked_preserving_target(
        &mut self,
        source: &Place,
        target: &Place,
    ) {
        self.copy_alias_if_tracked_with_mode(source, target, false);
    }

    /// Copies stable scalar-value facts without creating a raw-address owner alias.
    pub(super) fn copy_scalar_facts_if_tracked(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let source_origin = self.canonicalize_scalar(source);
        let fact_copies = self.scalar_fact_copies_for_aliases(source, target);
        self.clear_scalar_metadata(target);
        self.copy_scalar_facts_to_fresh_target(source, target, source_origin, fact_copies);
    }

    /// Cleared target へ source の安定した scalar 事実を写す。
    ///
    /// 呼び出し側は target に残っている古い scalar metadata を事前に消しておく必要がある。
    /// i32 の 0-offset 代入のように、差分事実を追加した直後に同じ値の条件や定数も
    /// 併せて保持したい処理で、差分事実を消さずに scalar 事実だけを復元するために使う。
    pub(super) fn copy_scalar_facts_to_fresh_target(
        &mut self,
        source: &Place,
        target: &Place,
        source_origin: Place,
        fact_copies: ScalarFactCopies,
    ) {
        fact_copies.apply_to(self);
        self.scalar_origins.copy_stable_origin(source, target);
        self.scalar_origins
            .record_stable_origin(target, &source_origin);
    }

    pub(super) fn copy_scalar_origin_from_raw_view_if_stable(
        &mut self,
        source: &Place,
        target: &Place,
    ) {
        let origin = self.canonicalize_scalar(&self.raw_view_origins.origin_for(source));
        self.scalar_origins.record_stable_origin(target, &origin);
    }

    fn copy_alias_if_tracked_with_mode(
        &mut self,
        source: &Place,
        target: &Place,
        clear_target: bool,
    ) {
        if source == target {
            return;
        }
        let source_origin = self.canonicalize_scalar(source);
        let fact_copies = self.scalar_fact_copies_for_aliases(source, target);
        let groups = self.groups_with_replaced_prefix(source, target);
        if clear_target {
            self.clear(target);
        }
        for group in groups {
            self.union_group(&group);
        }
        fact_copies.apply_to(self);
        self.raw_view_origins.copy_stable_origin(source, target);
        self.scalar_origins.copy_stable_origin(source, target);
        self.scalar_origins
            .record_stable_origin(target, &source_origin);
    }

    pub(super) fn scalar_fact_copies_for_aliases(
        &self,
        source: &Place,
        target: &Place,
    ) -> ScalarFactCopies {
        let mut sources = Vec::new();
        push_unique_place(&mut sources, source);
        for alias in self.scalar_aliases_for(source) {
            push_unique_place(&mut sources, &alias);
        }
        let mut copies = ScalarFactCopies::default();
        for source in sources {
            copies.extend_from(self, &source, target);
        }
        copies
    }

    /// Records an explicit raw-address relation while preserving any already-tracked aliases and
    /// scalar facts reachable from the source.
    pub(super) fn copy_explicit_raw_address_alias(&mut self, source: &Place, target: &Place) {
        if source == target {
            self.mark(target);
            return;
        }
        self.copy_alias_if_tracked(source, target);
        let mut group = Vec::new();
        push_unique_place(&mut group, source);
        push_unique_place(&mut group, target);
        push_unique_place(&mut group, &self.raw_view_origins.origin_for(source));
        push_unique_place(&mut group, &self.raw_view_origins.origin_for(target));
        self.union_group(&group);
    }

    pub(super) fn copy_explicit_raw_address_alias_preserving_target(
        &mut self,
        source: &Place,
        target: &Place,
    ) {
        if source == target {
            self.mark(target);
            return;
        }
        self.copy_alias_if_tracked_preserving_target(source, target);
        let mut group = Vec::new();
        push_unique_place(&mut group, source);
        push_unique_place(&mut group, target);
        push_unique_place(&mut group, &self.raw_view_origins.origin_for(source));
        push_unique_place(&mut group, &self.raw_view_origins.origin_for(target));
        self.union_group(&group);
    }
    pub(super) fn record_raw_address_view_origin(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        self.clear_raw_address_metadata(target);
        self.raw_view_origins.record_view_origin(source, target);
    }

    pub(super) fn move_owner_aliases(&mut self, source: &Place, target: &Place) {
        if source == target {
            push_unique_place(&mut self.marked, target);
            self.union_group(core::slice::from_ref(target));
            return;
        }
        let moved_marks = self
            .marked
            .iter()
            .filter(|marked| owner_alias_place_has_raw_projection(marked, source))
            .filter_map(|marked| {
                replace_place_prefix(marked, source, target).map(|moved| (moved, marked.clone()))
            })
            .collect::<Vec<_>>();
        let moved_facts = self.i32_facts.facts_with_replaced_prefix(source, target);
        let moved_differences = self
            .i32_differences
            .facts_with_replaced_prefix(source, target);
        let moved_relations = self
            .i32_relations
            .facts_with_replaced_prefix(source, target);
        let moved_scales = self.i32_scales.facts_with_replaced_prefix(source, target);
        let moved_offsets = self.i32_offsets.facts_with_replaced_prefix(source, target);
        let moved_type_sizes = self
            .i32_type_sizes
            .facts_with_replaced_prefix(source, target);
        let moved_host_sizes = self
            .host_size_facts
            .facts_with_replaced_prefix(source, target);
        let groups = self
            .groups_with_replaced_prefix(source, target)
            .into_iter()
            .filter(|group| {
                group
                    .iter()
                    .any(|place| owner_alias_place_has_raw_projection(place, target))
            })
            .collect::<Vec<_>>();
        self.clear(target);
        for group in groups {
            self.union_group(&group);
        }
        self.clear(source);
        push_unique_place(&mut self.marked, target);
        self.union_group(core::slice::from_ref(target));
        for (moved, origin) in moved_marks {
            push_unique_place(&mut self.marked, &moved);
            self.union_group(core::slice::from_ref(&moved));
            self.raw_view_origins.record_stable_origin(&moved, &origin);
        }
        self.i32_facts.extend(moved_facts);
        self.i32_differences.extend(moved_differences);
        self.i32_relations.extend(moved_relations);
        self.i32_scales.extend(moved_scales);
        self.i32_offsets.extend(moved_offsets);
        self.i32_type_sizes.extend(moved_type_sizes);
        self.host_size_facts.extend(moved_host_sizes);
    }

    pub(super) fn canonicalize(&self, place: &Place) -> Place {
        if let Some(canonical) = self.canonicalize_group_member(place) {
            return canonical;
        }
        let origin = self.raw_view_origins.origin_for(place);
        if origin != *place {
            if let Some(canonical) = self.canonicalize_group_member(&origin) {
                return canonical;
            }
            return origin;
        }
        place.clone()
    }

    pub(super) fn canonicalize_owner_cell_address(&self, place: &Place) -> Place {
        self.aliases_for(place)
            .into_iter()
            .min_by_key(owner_cell_alias_rank)
            .unwrap_or_else(|| place.clone())
    }

    pub(super) fn prefer_canonical(&mut self, place: &Place) {
        for group in &mut self.groups {
            let Some(index) = group.iter().position(|alias| alias == place) else {
                continue;
            };
            if index != 0 {
                let alias = group.remove(index);
                group.insert(0, alias);
            }
        }
    }

    pub(super) fn contains_exact(&self, place: &Place) -> bool {
        self.groups
            .iter()
            .any(|group| group.iter().any(|alias| alias == place))
    }

    pub(super) fn value_is_known_raw_address(&self, place: &Place) -> bool {
        self.contains_exact(place) || self.aliases_for(place).len() > 1
    }

    pub(super) fn contains_marked_alias(&self, place: &Place) -> bool {
        self.aliases_for(place)
            .iter()
            .any(|alias| self.marked.iter().any(|marked| marked == alias))
    }

    pub(super) fn aliases_for(&self, place: &Place) -> Vec<Place> {
        let mut out = Vec::new();
        for group in self.alias_groups_for(place) {
            for alias in group {
                push_unique_place(&mut out, &alias);
            }
        }
        if out.is_empty() {
            push_unique_place(&mut out, place);
        }
        out
    }

    pub(super) fn raw_address_aliases_for_value(&self, place: &Place) -> Vec<Place> {
        let mut out = self.aliases_for(place);
        for origin in self.raw_view_origins.origins_for(place) {
            if origin == *place {
                continue;
            }
            push_unique_place(&mut out, &origin);
            for alias in self.aliases_for(&origin) {
                push_unique_place(&mut out, &alias);
            }
        }
        out
    }

    pub(super) fn tracked_places(&self) -> Vec<Place> {
        let mut out = Vec::new();
        for group in &self.groups {
            for place in group {
                push_unique_place(&mut out, place);
            }
        }
        for place in &self.marked {
            push_unique_place(&mut out, place);
        }
        out
    }

    pub(super) fn prefix_aliases_for(&self, place: &Place) -> Vec<Place> {
        let mut out = Vec::new();
        for group in &self.groups {
            for group_place in group {
                let Some(suffix) = place_suffix_after_prefix(place, group_place) else {
                    continue;
                };
                for alias in group {
                    push_unique_place(&mut out, &place_with_suffix(alias, &suffix, place.ty));
                }
            }
        }
        out
    }

    pub(super) fn projected_aliases_between(
        &self,
        left_base: &Place,
        right_base: &Place,
    ) -> Vec<ProjectedRawCellAddressAlias> {
        let mut out = Vec::new();
        for group in &self.groups {
            let left_places = group
                .iter()
                .filter_map(|place| {
                    place_suffix_after_prefix(place, left_base).map(|suffix| (suffix, place.ty))
                })
                .collect::<Vec<_>>();
            if left_places.is_empty() {
                continue;
            }
            let right_places = group
                .iter()
                .filter_map(|place| {
                    place_suffix_after_prefix(place, right_base).map(|suffix| (suffix, place.ty))
                })
                .collect::<Vec<_>>();
            for (left_projection, left_ty) in &left_places {
                for (right_projection, right_ty) in &right_places {
                    push_unique_projected_alias(
                        &mut out,
                        ProjectedRawCellAddressAlias {
                            left_projection: left_projection.clone(),
                            left_ty: *left_ty,
                            right_projection: right_projection.clone(),
                            right_ty: *right_ty,
                        },
                    );
                }
            }
        }
        out
    }

    pub(super) fn clear(&mut self, place: &Place) {
        self.clear_raw_address_metadata(place);
        self.clear_scalar_metadata(place);
    }

    pub(super) fn clear_raw_address_facts(&mut self, place: &Place) {
        self.clear_raw_address_metadata(place);
    }

    pub(super) fn clear_scalar_facts(&mut self, place: &Place) {
        self.clear_scalar_metadata(place);
    }

    pub(super) fn can_prove_i32_value_condition(&self) -> bool {
        self.i32_facts.has_condition_sources()
            || self.i32_relations.has_facts()
            || self.i32_scales.has_facts()
            || self.i32_offsets.has_i32_constant_endpoint()
    }

    fn clear_scalar_metadata(&mut self, place: &Place) {
        self.scalar_origins.clear_prefix(place);
        self.i32_facts.clear_prefix(place);
        self.i32_differences.clear_prefix(place);
        self.i32_relations.clear_prefix(place);
        self.i32_scales.clear_prefix(place);
        self.i32_offsets.clear_prefix(place);
        self.i32_type_sizes.clear_prefix(place);
        self.host_size_facts.clear_prefix(place);
    }

    fn clear_raw_address_metadata(&mut self, place: &Place) {
        for group in &mut self.groups {
            group.retain(|existing| place_suffix_after_prefix(existing, place).is_none());
        }
        self.groups.retain(|group| !group.is_empty());
        self.marked
            .retain(|existing| place_suffix_after_prefix(existing, place).is_none());
        self.raw_view_origins.clear_prefix(place);
    }

    pub(super) fn merge_paths(paths: &[RawCellAddressAliases]) -> Self {
        let mut out = RawCellAddressAliases::default();
        for path in paths {
            for group in &path.groups {
                out.union_group(group);
            }
            for marked in &path.marked {
                push_unique_place(&mut out.marked, marked);
            }
        }
        out.raw_view_origins =
            RawValueOrigins::merge_paths(paths.iter().map(|path| &path.raw_view_origins));
        out.scalar_origins =
            RawValueOrigins::merge_paths(paths.iter().map(|path| &path.scalar_origins));
        out.i32_facts = I32AliasFacts::merge_paths(paths.iter().map(|path| &path.i32_facts));
        out.i32_differences =
            I32DifferenceFacts::merge_paths(paths.iter().map(|path| &path.i32_differences));
        out.i32_relations =
            I32RelationFacts::merge_paths(paths.iter().map(|path| &path.i32_relations));
        out.i32_scales = I32ScaleFacts::merge_paths(paths.iter().map(|path| &path.i32_scales));
        out.i32_offsets = I32OffsetFacts::merge_paths(paths.iter().map(|path| &path.i32_offsets));
        out.i32_type_sizes =
            I32TypeSizeFacts::merge_paths(paths.iter().map(|path| &path.i32_type_sizes));
        out.host_size_facts =
            HostSizeFacts::merge_paths(paths.iter().map(|path| &path.host_size_facts));
        out
    }

    fn canonicalize_group_member(&self, place: &Place) -> Option<Place> {
        for group in &self.groups {
            for alias in group {
                if let Some(suffix) = place_suffix_after_prefix(place, alias) {
                    return Some(place_with_suffix(&group[0], &suffix, place.ty));
                }
            }
        }
        None
    }

    fn alias_groups_for(&self, place: &Place) -> Vec<Vec<Place>> {
        let mut out = Vec::new();
        for group in &self.groups {
            let mut mapped = Vec::new();
            for alias in group {
                if let Some(suffix) = place_suffix_after_prefix(place, alias) {
                    for group_alias in group {
                        push_unique_place(
                            &mut mapped,
                            &place_with_suffix(group_alias, &suffix, place.ty),
                        );
                    }
                    break;
                }
            }
            if !mapped.is_empty() {
                out.push(mapped);
            }
        }
        out
    }

    fn groups_with_replaced_prefix(&self, source: &Place, target: &Place) -> Vec<Vec<Place>> {
        let mut out = Vec::new();
        for group in &self.groups {
            let mut mapped = Vec::new();
            let mut replaces_group_member = false;
            for place in group {
                if let Some(replacement) = replace_place_prefix(place, source, target) {
                    push_unique_place(&mut mapped, &replacement);
                    replaces_group_member = true;
                }
            }
            if mapped.is_empty() {
                for alias in group {
                    if let Some(suffix) = place_suffix_after_prefix(source, alias) {
                        for group_alias in group {
                            push_unique_place(
                                &mut mapped,
                                &place_with_suffix(group_alias, &suffix, source.ty),
                            );
                        }
                        push_unique_place(&mut mapped, target);
                        break;
                    }
                }
            }
            if mapped.is_empty() {
                continue;
            }

            if !replaces_group_member {
                out.push(mapped);
                continue;
            }

            let mut merged: Vec<Place> = group
                .iter()
                .filter(|place| place_suffix_after_prefix(place, target).is_none())
                .cloned()
                .collect();
            for place in &mapped {
                push_unique_place(&mut merged, place);
            }
            out.push(merged);
        }

        out
    }

    pub(super) fn scalar_aliases_for(&self, place: &Place) -> Vec<Place> {
        let mut out = self.aliases_for(place);
        for origin in self.scalar_origins.origins_for(place) {
            if origin == *place {
                continue;
            }
            push_unique_place(&mut out, &origin);
            for alias in self.aliases_for(&origin) {
                push_unique_place(&mut out, &alias);
            }
            for reverse_origin in self.scalar_origins.places_with_origin(&origin) {
                push_unique_place(&mut out, &reverse_origin);
                for reverse_alias in self.aliases_for(&reverse_origin) {
                    push_unique_place(&mut out, &reverse_alias);
                }
            }
        }
        for reverse_origin in self.scalar_origins.places_with_origin(place) {
            push_unique_place(&mut out, &reverse_origin);
            for reverse_alias in self.aliases_for(&reverse_origin) {
                push_unique_place(&mut out, &reverse_alias);
            }
        }
        out
    }

    fn union_group(&mut self, group: &[Place]) {
        let mut merged = Vec::new();
        let mut retained = Vec::new();
        for existing in self.groups.drain(..) {
            if groups_overlap(&existing, group) {
                for place in &existing {
                    push_unique_place(&mut merged, place);
                }
            } else {
                retained.push(existing);
            }
        }
        for place in group {
            push_unique_place(&mut merged, place);
        }
        if !merged.is_empty() {
            prefer_stable_canonical(&mut merged);
            retained.push(merged);
        }
        self.groups = retained;
    }
}
