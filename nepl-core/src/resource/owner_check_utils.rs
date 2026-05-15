use super::model::{EffectOp, Place, PlaceProjection};
use super::report::ResourceOwnerCheckDeferred;

pub(super) fn raw_owner_alias_moves_into_wrapper(source: &Place, target: &Place) -> bool {
    target.projections.len() > source.projections.len()
        || raw_owner_alias_moves_between_wrapper_raw_fields(source, target)
}

fn raw_owner_alias_moves_between_wrapper_raw_fields(source: &Place, target: &Place) -> bool {
    source.root != target.root
        && source.projections.len() == target.projections.len()
        && matches!(source.projections.last(), Some(projection) if is_zero_offset_first_field(projection))
        && matches!(target.projections.last(), Some(projection) if is_zero_offset_first_field(projection))
}

fn is_zero_offset_first_field(projection: &PlaceProjection) -> bool {
    matches!(
        projection,
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0
        }
    )
}

pub(super) fn merge_owner_deferred(
    target: &mut ResourceOwnerCheckDeferred,
    source: ResourceOwnerCheckDeferred,
) {
    target.branch_merges += source.branch_merges;
    target.loop_merges += source.loop_merges;
    target.match_merges += source.match_merges;
}

pub(super) fn direct_raw_memory_effect(effect: &EffectOp) -> bool {
    matches!(
        effect,
        EffectOp::InternalAlloc { .. } | EffectOp::UnsafeMemory { .. }
    )
}
