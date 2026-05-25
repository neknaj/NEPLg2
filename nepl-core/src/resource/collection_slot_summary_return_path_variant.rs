extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_model::{
    CollectionSlotLifecycleReturnPath, CollectionSlotLifecycleSummaryI32Operand,
    CollectionSlotLifecycleSummaryOp,
};
use super::collection_slot_summary_return_path_condition::return_path_preconditions_match_callsite;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::Place;
use super::summary_projection::{
    instantiate_summary_suffix_on_base, SummaryPlace, SummaryProjection,
};
use super::variant_name::variant_names_match;

pub(super) fn return_path_matches_callsite_variants(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    raw_aliases: &RawCellAddressAliases,
    variants: &PendingVariantRawCellInitializations,
    path: &CollectionSlotLifecycleReturnPath,
) -> bool {
    return_path_preconditions_match_callsite(engine, args, raw_aliases, &path.preconditions)
        && summary_ops_match_callsite_variants(engine, args, variants, &path.ops)
        && path.return_transfers.iter().all(|transfer| {
            summary_place_matches_callsite_variants(engine, args, variants, &transfer.source)
        })
}

fn summary_ops_match_callsite_variants(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    variants: &PendingVariantRawCellInitializations,
    ops: &[CollectionSlotLifecycleSummaryOp],
) -> bool {
    ops.iter()
        .all(|op| summary_op_matches_callsite_variants(engine, args, variants, op))
}

fn summary_op_matches_callsite_variants(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    variants: &PendingVariantRawCellInitializations,
    op: &CollectionSlotLifecycleSummaryOp,
) -> bool {
    match op {
        CollectionSlotLifecycleSummaryOp::Event { target, .. } => {
            summary_place_matches_callsite_variants(engine, args, variants, target)
        }
        CollectionSlotLifecycleSummaryOp::Relocate {
            old_storage,
            new_storage,
            ..
        } => {
            summary_place_matches_callsite_variants(engine, args, variants, old_storage)
                && summary_place_matches_callsite_variants(engine, args, variants, new_storage)
        }
        CollectionSlotLifecycleSummaryOp::DropTraversal {
            storage,
            initialized_count,
            ..
        } => {
            summary_place_matches_callsite_variants(engine, args, variants, storage)
                && summary_i32_operand_matches_callsite_variants(
                    engine,
                    args,
                    variants,
                    initialized_count,
                )
        }
        CollectionSlotLifecycleSummaryOp::TransformRange {
            source_storage,
            source_initialized_count,
            output_storage,
            output_initialized_count,
            ..
        } => {
            summary_place_matches_callsite_variants(engine, args, variants, source_storage)
                && summary_place_matches_callsite_variants(
                    engine,
                    args,
                    variants,
                    source_initialized_count,
                )
                && summary_place_matches_callsite_variants(engine, args, variants, output_storage)
                && summary_place_matches_callsite_variants(
                    engine,
                    args,
                    variants,
                    output_initialized_count,
                )
        }
        CollectionSlotLifecycleSummaryOp::TransformRangeSourceDrain {
            source_storage,
            source_initialized_count,
            ..
        } => {
            summary_place_matches_callsite_variants(engine, args, variants, source_storage)
                && summary_place_matches_callsite_variants(
                    engine,
                    args,
                    variants,
                    source_initialized_count,
                )
        }
        CollectionSlotLifecycleSummaryOp::Merge { paths } => paths
            .iter()
            .any(|path| summary_ops_match_callsite_variants(engine, args, variants, path)),
        CollectionSlotLifecycleSummaryOp::Loop {
            condition_ops,
            body_ops,
        } => {
            summary_ops_match_callsite_variants(engine, args, variants, condition_ops)
                && summary_ops_match_callsite_variants(engine, args, variants, body_ops)
        }
    }
}

fn summary_i32_operand_matches_callsite_variants(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    variants: &PendingVariantRawCellInitializations,
    operand: &CollectionSlotLifecycleSummaryI32Operand,
) -> bool {
    match operand {
        CollectionSlotLifecycleSummaryI32Operand::Place(place) => {
            summary_place_matches_callsite_variants(engine, args, variants, place)
        }
        CollectionSlotLifecycleSummaryI32Operand::KnownI32 { .. } => true,
    }
}

fn summary_place_matches_callsite_variants(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    variants: &PendingVariantRawCellInitializations,
    place: &SummaryPlace,
) -> bool {
    let Some(base) = args.get(place.parameter_index) else {
        return true;
    };
    let mut prefix = base.clone();
    let mut suffix_prefix = Vec::new();
    for projection in &place.suffix {
        if let SummaryProjection::EnumPayload { variant } = projection {
            if let Some(concrete) = variants.concrete_variant(&prefix) {
                if !variant_names_match(concrete, variant) {
                    return false;
                }
            }
        }
        suffix_prefix.push(projection.clone());
        let Some(next_prefix) =
            instantiate_summary_suffix_on_base(engine, args, base, &suffix_prefix, place.ty)
        else {
            return true;
        };
        prefix = next_prefix;
    }
    true
}
