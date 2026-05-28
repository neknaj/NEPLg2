extern crate alloc;

use alloc::vec::Vec;

use super::initialized_scalar_flow::{I32ScalarConcreteVariant, I32ScalarConcreteVariants};
use super::initialized_scalar_flow_return_facts::push_unique_i32_scalar_return_projection;
use super::model::{AggregateKind, Place, PlaceProjection, ResourceOp};
use super::place_utils::{
    construct_aggregate_field_place, projection_result_type, replace_place_prefix,
};
use super::variant_name::normalize_variant_name;
use crate::types::TypeCtx;

impl I32ScalarConcreteVariants {
    pub(super) fn clear(&mut self, place: &Place) {
        self.entries.retain(|entry| {
            super::place_utils::place_suffix_after_prefix(&entry.place, place).is_none()
        });
    }

    pub(super) fn set(&mut self, place: &Place, variant: &str) {
        self.clear(place);
        self.entries.push(I32ScalarConcreteVariant {
            place: place.clone(),
            variant: normalize_variant_name(variant),
        });
    }

    pub(super) fn copy(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let copied = self
            .entries
            .iter()
            .filter_map(|entry| {
                replace_place_prefix(&entry.place, source, target).map(|place| {
                    I32ScalarConcreteVariant {
                        place,
                        variant: entry.variant.clone(),
                    }
                })
            })
            .collect::<Vec<_>>();
        self.clear(target);
        for entry in copied {
            self.push_unique(entry);
        }
    }

    pub(super) fn variant_for(&self, place: &Place) -> Option<&str> {
        self.entries
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.variant.as_str())
    }

    pub(super) fn push_variant_projection_paths(
        &self,
        value: &Place,
        projections: &mut Vec<Vec<PlaceProjection>>,
    ) {
        for entry in &self.entries {
            let Some(prefix) = super::place_utils::place_suffix_after_prefix(&entry.place, value)
            else {
                continue;
            };
            let mut projection = prefix;
            projection.push(PlaceProjection::EnumPayload {
                variant: entry.variant.clone(),
            });
            push_unique_i32_scalar_return_projection(projections, &projection);
        }
    }

    pub(super) fn projection_is_possible(
        &self,
        types: &TypeCtx,
        value: &Place,
        projection: &[PlaceProjection],
    ) -> bool {
        let mut prefix = Vec::new();
        let mut current_ty = value.ty;
        for item in projection {
            if let PlaceProjection::EnumPayload { variant } = item {
                let enum_place = super::place_utils::place_with_suffix(value, &prefix, current_ty);
                if let Some(known) = self.variant_for(&enum_place) {
                    if known != normalize_variant_name(variant) {
                        return false;
                    }
                }
            }
            current_ty = projection_result_type(types, current_ty, item).unwrap_or(current_ty);
            prefix.push(item.clone());
        }
        true
    }

    pub(super) fn push_unique(&mut self, entry: I32ScalarConcreteVariant) {
        if self.entries.iter().any(|existing| existing == &entry) {
            return;
        }
        self.entries.push(entry);
    }
}

pub(super) fn propagate_i32_scalar_concrete_variant_op(
    variants: &mut I32ScalarConcreteVariants,
    op: &ResourceOp,
) {
    match op {
        ResourceOp::DeclareLocal {
            place, initializer, ..
        } => {
            if let Some(initializer) = initializer {
                variants.copy(initializer, place);
            } else {
                variants.clear(place);
            }
        }
        ResourceOp::Read { source, output, .. }
        | ResourceOp::Move { source, output, .. }
        | ResourceOp::Assign {
            target: output,
            value: source,
            ..
        } => variants.copy(source, output),
        ResourceOp::Construct {
            output,
            kind,
            inputs,
            ..
        } => {
            variants.clear(output);
            for (index, input) in inputs.iter().enumerate() {
                let field = construct_aggregate_field_place(output, kind, index, input);
                variants.copy(input, &field);
            }
            if let AggregateKind::Enum { variant, .. } = kind {
                variants.set(output, variant);
            }
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            let mut condition_variants = variants.clone();
            propagate_i32_scalar_concrete_variant_ops(&mut condition_variants, condition_ops);
            let mut body_variants = condition_variants.clone();
            propagate_i32_scalar_concrete_variant_ops(&mut body_variants, body_ops);
            *variants = merge_i32_scalar_concrete_variants(&[condition_variants, body_variants]);
        }
        ResourceOp::Expr { output, .. }
        | ResourceOp::Call { output, .. }
        | ResourceOp::IndirectCall { output, .. }
        | ResourceOp::FunctionValue { output, .. }
        | ResourceOp::RawMemory { output, .. }
        | ResourceOp::Borrow { output, .. } => variants.clear(output),
        ResourceOp::Drop { place, .. } => variants.clear(place),
        ResourceOp::Branch { .. }
        | ResourceOp::Match { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::CollectionSlotTransformRange { .. } => {}
    }
}

pub(super) fn propagate_i32_scalar_concrete_variant_ops(
    variants: &mut I32ScalarConcreteVariants,
    ops: &[ResourceOp],
) {
    for op in ops {
        propagate_i32_scalar_concrete_variant_op(variants, op);
    }
}

pub(super) fn merge_i32_scalar_concrete_variants(
    paths: &[I32ScalarConcreteVariants],
) -> I32ScalarConcreteVariants {
    let mut out = I32ScalarConcreteVariants::default();
    let Some(first) = paths.first() else {
        return out;
    };
    for entry in &first.entries {
        if paths
            .iter()
            .skip(1)
            .all(|path| path.entries.iter().any(|path_entry| path_entry == entry))
        {
            out.push_unique(entry.clone());
        }
    }
    out
}
