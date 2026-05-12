use super::model::{EffectOp, Place};
use super::report::ResourceOwnerCheckDeferred;

pub(super) fn raw_owner_alias_moves_into_wrapper(source: &Place, target: &Place) -> bool {
    target.projections.len() > source.projections.len()
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
