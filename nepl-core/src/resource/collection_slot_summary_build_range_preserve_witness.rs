use super::collection_slot_summary_build_range_preserve_op::{
    op_preserves_place, op_preserves_place_after_drop_witness,
};
use super::collection_slot_summary_build_range_preserve_witness_op::{
    op_loads_from_place, op_preserves_place_during_drop_witness,
    paired_witness_load_call_preserves_place, unsafe_load_call_matches_raw_load,
};
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_scalar_flow_ops::propagate_i32_scalar_ops;
use super::model::{Place, ResourceOp};

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
        let paired_witness_load_call = op_index + 1 == witness_load_index
            && ops
                .get(witness_load_index)
                .is_some_and(|load| unsafe_load_call_matches_raw_load(op, load));
        if !paired_witness_load_call
            && op_index != witness_load_index
            && op_loads_from_place(&raw_aliases, op, protected)
        {
            return false;
        }
        let preserves = if paired_witness_load_call {
            paired_witness_load_call_preserves_place(op, protected)
        } else if op_index == witness_load_index {
            op_preserves_place(engine, &raw_aliases, op, protected)
        } else if op_index <= witness_drop_index {
            op_preserves_place_during_drop_witness(&raw_aliases, op, protected)
        } else {
            op_preserves_place_after_drop_witness(engine, &raw_aliases, op, protected)
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
