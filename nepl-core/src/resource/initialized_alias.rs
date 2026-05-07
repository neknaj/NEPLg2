extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::initialized_alias_origin::RawValueOrigins;
use super::initialized_alias_rank::{
    owner_alias_place_has_raw_projection, owner_cell_alias_rank, prefer_stable_canonical,
};
use super::initialized_alias_relation::I32RelationFacts;
use super::initialized_alias_scalar::I32AliasFacts;
use super::initialized_alias_scale::I32ScaleFacts;
use super::model::{I32ValueCondition, Place, ResourceI32RelationOp};
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

#[derive(Debug, Clone, Default)]
pub(super) struct RawCellAddressAliases {
    groups: Vec<Vec<Place>>,
    marked: Vec<Place>,
    value_origins: RawValueOrigins,
    i32_facts: I32AliasFacts,
    i32_relations: I32RelationFacts,
    i32_scales: I32ScaleFacts,
}

impl RawCellAddressAliases {
    pub(super) fn mark(&mut self, place: &Place) {
        self.clear(place);
        push_unique_place(&mut self.marked, place);
        self.union_group(core::slice::from_ref(place));
    }

    /// Copies an existing raw-address alias group and scalar facts without treating ordinary
    /// i32 value copies as raw-address aliases.
    pub(super) fn copy_alias_if_tracked(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let fact_copies = self.i32_facts.facts_with_replaced_prefix(source, target);
        let relation_copies = self
            .i32_relations
            .facts_with_replaced_prefix(source, target);
        let scale_copies = self.i32_scales.facts_with_replaced_prefix(source, target);
        let groups = self.groups_with_replaced_prefix(source, target);
        self.clear(target);
        for group in groups {
            self.union_group(&group);
        }
        self.i32_facts.extend(fact_copies);
        self.i32_relations.extend(relation_copies);
        self.i32_scales.extend(scale_copies);
        self.value_origins.copy_stable_origin(source, target);
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
        push_unique_place(&mut group, &self.value_origins.origin_for(source));
        push_unique_place(&mut group, &self.value_origins.origin_for(target));
        self.union_group(&group);
    }
    pub(super) fn record_raw_address_view_origin(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        self.clear(target);
        self.value_origins.record_view_origin(source, target);
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
            .filter_map(|marked| replace_place_prefix(marked, source, target))
            .collect::<Vec<_>>();
        let moved_facts = self.i32_facts.facts_with_replaced_prefix(source, target);
        let moved_relations = self
            .i32_relations
            .facts_with_replaced_prefix(source, target);
        let moved_scales = self.i32_scales.facts_with_replaced_prefix(source, target);
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
        for moved in moved_marks {
            push_unique_place(&mut self.marked, &moved);
            self.union_group(core::slice::from_ref(&moved));
        }
        self.i32_facts.extend(moved_facts);
        self.i32_relations.extend(moved_relations);
        self.i32_scales.extend(moved_scales);
    }

    pub(super) fn canonicalize(&self, place: &Place) -> Place {
        if let Some(canonical) = self.canonicalize_group_member(place) {
            return canonical;
        }
        let origin = self.value_origins.origin_for(place);
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

    pub(super) fn set_i32_value(&mut self, place: &Place, value: i32) {
        let place = self.canonicalize(place);
        self.i32_facts.set_value(&place, value);
    }

    pub(super) fn add_i32_condition(&mut self, place: &Place, condition: I32ValueCondition) {
        let place = self.canonicalize(place);
        self.i32_facts.add_condition(&place, condition);
    }

    pub(super) fn add_i32_relation(
        &mut self,
        left: &Place,
        op: ResourceI32RelationOp,
        right: &Place,
    ) {
        let left = self.canonicalize(left);
        let right = self.canonicalize(right);
        self.i32_relations.add_relation(&left, op, &right);
    }

    pub(super) fn add_i32_scale(&mut self, source: &Place, target: &Place, scale: usize) {
        let source = self.canonicalize(source);
        self.i32_scales.add_scale(&source, target, scale);
    }

    pub(super) fn i32_value(&self, place: &Place) -> Option<i32> {
        self.i32_facts
            .value_for_aliases(&self.scalar_aliases_for(place))
    }

    pub(super) fn i32_condition_truth(
        &self,
        place: &Place,
        condition: I32ValueCondition,
    ) -> Option<bool> {
        if let Some(value) = self.i32_value(place) {
            return Some(condition.holds(value));
        }
        self.i32_facts
            .condition_truth_for_aliases(&self.scalar_aliases_for(place), condition)
    }

    pub(super) fn i32_relation_truth(
        &self,
        left: &Place,
        op: ResourceI32RelationOp,
        right: &Place,
    ) -> Option<bool> {
        if let (Some(left_value), Some(right_value)) = (self.i32_value(left), self.i32_value(right))
        {
            return Some(relation_holds(left_value, op, right_value));
        }
        self.i32_relations.relation_truth_for_aliases(
            &self.scalar_aliases_for(left),
            op,
            &self.scalar_aliases_for(right),
        )
    }

    pub(super) fn i32_scaled_source(&self, place: &Place) -> Option<(Place, usize)> {
        let mut out = None;
        for (source, scale) in self
            .i32_scales
            .scaled_sources_for_aliases(&self.scalar_aliases_for(place))
        {
            let candidate = (self.canonicalize(&source), scale);
            match &out {
                Some(existing) if existing != &candidate => return None,
                Some(_) => {}
                None => out = Some(candidate),
            }
        }
        out
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
                let Some(suffix) = place_suffix_after_prefix(group_place, place) else {
                    continue;
                };
                for alias in group {
                    if let Some(prefix) = place_without_suffix(alias, &suffix, place.ty) {
                        push_unique_place(&mut out, &prefix);
                    }
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
        for group in &mut self.groups {
            group.retain(|existing| place_suffix_after_prefix(existing, place).is_none());
        }
        self.groups.retain(|group| !group.is_empty());
        self.marked
            .retain(|existing| place_suffix_after_prefix(existing, place).is_none());
        self.value_origins.clear_prefix(place);
        self.i32_facts.clear_prefix(place);
        self.i32_relations.clear_prefix(place);
        self.i32_scales.clear_prefix(place);
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
        out.value_origins =
            RawValueOrigins::merge_paths(paths.iter().map(|path| &path.value_origins));
        out.i32_facts = I32AliasFacts::merge_paths(paths.iter().map(|path| &path.i32_facts));
        out.i32_relations =
            I32RelationFacts::merge_paths(paths.iter().map(|path| &path.i32_relations));
        out.i32_scales = I32ScaleFacts::merge_paths(paths.iter().map(|path| &path.i32_scales));
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

    fn scalar_aliases_for(&self, place: &Place) -> Vec<Place> {
        let mut out = self.aliases_for(place);
        let origin = self.value_origins.origin_for(place);
        if origin != *place {
            push_unique_place(&mut out, &origin);
            for alias in self.aliases_for(&origin) {
                push_unique_place(&mut out, &alias);
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

#[cfg(test)]
mod tests {
    use super::super::model::ResourceId;
    use super::*;
    use alloc::string::String;
    use ResourceI32RelationOp::Lt;

    fn local(name: &str) -> Place {
        Place::local(String::from(name), TypeId(1))
    }

    #[test]
    fn i32_relation_facts_follow_alias_copy() {
        let source = local("i");
        let target = local("j");
        let len = local("len");
        let mut aliases = RawCellAddressAliases::default();

        aliases.add_i32_relation(&source, Lt, &len);
        aliases.copy_alias_if_tracked(&source, &target);

        assert_eq!(aliases.i32_relation_truth(&target, Lt, &len), Some(true));
    }

    #[test]
    fn i32_scale_facts_follow_stable_value_copies() {
        let source = local("i");
        let source_read = Place::temporary(ResourceId(1), source.ty);
        let scaled_tmp = Place::temporary(ResourceId(2), source.ty);
        let scaled_local = local("off");
        let scaled_read = Place::temporary(ResourceId(3), source.ty);
        let mut aliases = RawCellAddressAliases::default();

        aliases.copy_alias_if_tracked(&source, &source_read);
        aliases.add_i32_scale(&source_read, &scaled_tmp, 4);
        aliases.copy_alias_if_tracked(&scaled_tmp, &scaled_local);
        aliases.copy_alias_if_tracked(&scaled_local, &scaled_read);

        assert_eq!(aliases.i32_scaled_source(&scaled_read), Some((source, 4)));
    }

    #[test]
    fn i32_relation_facts_match_stable_value_origin_copies() {
        let left = local("i");
        let right = local("len");
        let right_read = Place::temporary(ResourceId(4), right.ty);
        let mut aliases = RawCellAddressAliases::default();

        aliases.copy_alias_if_tracked(&right, &right_read);
        aliases.add_i32_relation(&left, Lt, &right);

        assert_eq!(
            aliases.i32_relation_truth(&left, Lt, &right_read),
            Some(true)
        );
    }

    #[test]
    fn i32_relation_merge_keeps_only_path_common_proofs() {
        let i = local("i");
        let len = local("len");
        let mut left = RawCellAddressAliases::default();
        let right = RawCellAddressAliases::default();

        left.add_i32_relation(&i, Lt, &len);
        let merged = RawCellAddressAliases::merge_paths(&[left.clone(), right]);
        assert_eq!(merged.i32_relation_truth(&i, Lt, &len), None);

        let merged = RawCellAddressAliases::merge_paths(&[left.clone(), left]);
        assert_eq!(merged.i32_relation_truth(&i, Lt, &len), Some(true));
    }
}

fn relation_holds(left: i32, op: ResourceI32RelationOp, right: i32) -> bool {
    match op {
        ResourceI32RelationOp::Eq => left == right,
        ResourceI32RelationOp::Ne => left != right,
        ResourceI32RelationOp::Lt => left < right,
        ResourceI32RelationOp::Le => left <= right,
        ResourceI32RelationOp::Gt => left > right,
        ResourceI32RelationOp::Ge => left >= right,
    }
}

fn groups_overlap(left: &[Place], right: &[Place]) -> bool {
    left.iter().any(|place| right.contains(place))
}

fn push_unique_projected_alias(
    aliases: &mut Vec<ProjectedRawCellAddressAlias>,
    alias: ProjectedRawCellAddressAlias,
) {
    if !aliases.iter().any(|existing| existing == &alias) {
        aliases.push(alias);
    }
}

fn place_without_suffix(
    place: &Place,
    suffix: &[super::model::PlaceProjection],
    ty: TypeId,
) -> Option<Place> {
    if suffix.len() > place.projections.len() {
        return None;
    }
    let prefix_len = place.projections.len() - suffix.len();
    if place.projections[prefix_len..] != *suffix {
        return None;
    }
    let mut out = place.clone();
    out.projections.truncate(prefix_len);
    out.ty = ty;
    Some(out)
}
