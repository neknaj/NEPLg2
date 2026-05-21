use super::collection_slot_summary_build_range_preserve_op::{
    op_preserves_place, op_preserves_place_after_drop_witness,
};
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_scalar_flow_ops::propagate_i32_scalar_ops;
use super::model::{EffectOp, Place, RawMemoryOp, ResourceOp};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn body_preserves_place_with_drop_witness(
    engine: &ResourceCheckEngine<'_>,
    initial_raw_aliases: &RawCellAddressAliases,
    ops: &[ResourceOp],
    protected: &Place,
    witness_load_index: usize,
    witness_drop_index: usize,
) -> bool {
    let mut raw_aliases = initial_raw_aliases.clone();
    let mut function_aliases = FunctionAliasTable::default();
    for (op_index, op) in ops.iter().enumerate() {
        if op_index != witness_load_index && op_loads_from_place(&raw_aliases, op, protected) {
            return false;
        }
        let preserves = if op_index > witness_drop_index {
            op_preserves_place_after_drop_witness(engine, &raw_aliases, op, protected)
        } else {
            op_preserves_place(engine, &raw_aliases, op, protected)
        };
        if !preserves {
            return false;
        }
        propagate_i32_scalar_ops(
            &mut raw_aliases,
            &mut function_aliases,
            core::slice::from_ref(op),
            engine.i32_scalar_summaries,
            engine.raw_alias_summaries,
            engine.types,
        );
    }
    true
}

fn op_loads_from_place(
    raw_aliases: &RawCellAddressAliases,
    op: &ResourceOp,
    protected: &Place,
) -> bool {
    match op {
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Load,
            args,
            ..
        }
        | ResourceOp::Call {
            effect:
                EffectOp::UnsafeMemory {
                    operation: RawMemoryOp::Load,
                },
            args,
            ..
        } => args
            .iter()
            .any(|arg| place_touches(raw_aliases, arg, protected)),
        ResourceOp::Call { .. } | ResourceOp::CallEffect { .. } | ResourceOp::RawMemory { .. } => {
            false
        }
        _ => false,
    }
}

fn place_touches(raw_aliases: &RawCellAddressAliases, left: &Place, right: &Place) -> bool {
    places_touch(left, right)
        || places_touch(
            &raw_aliases.canonicalize_owner_cell_address(left),
            &raw_aliases.canonicalize_owner_cell_address(right),
        )
        || places_touch(
            &raw_aliases.canonicalize_scalar(left),
            &raw_aliases.canonicalize_scalar(right),
        )
}

fn places_touch(left: &Place, right: &Place) -> bool {
    place_suffix_after_prefix(left, right).is_some()
        || place_suffix_after_prefix(right, left).is_some()
}
