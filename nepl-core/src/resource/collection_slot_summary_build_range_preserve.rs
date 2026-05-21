use super::collection_slot_summary_build_range_preserve_op::{
    op_preserves_place, op_preserves_place_after_drop_witness,
};
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_scalar_flow_ops::propagate_i32_scalar_ops;
use super::model::{Place, ResourceOp};

pub(super) fn body_preserves_place(
    engine: &ResourceCheckEngine<'_>,
    initial_raw_aliases: &RawCellAddressAliases,
    ops: &[ResourceOp],
    protected: &Place,
) -> bool {
    body_preserves_place_with(
        engine,
        initial_raw_aliases,
        ops,
        protected,
        op_preserves_place,
    )
}

pub(super) fn body_preserves_place_after_drop_witness(
    engine: &ResourceCheckEngine<'_>,
    initial_raw_aliases: &RawCellAddressAliases,
    ops: &[ResourceOp],
    protected: &Place,
) -> bool {
    body_preserves_place_with(
        engine,
        initial_raw_aliases,
        ops,
        protected,
        op_preserves_place_after_drop_witness,
    )
}

fn body_preserves_place_with(
    engine: &ResourceCheckEngine<'_>,
    initial_raw_aliases: &RawCellAddressAliases,
    ops: &[ResourceOp],
    protected: &Place,
    mut op_preserves: impl FnMut(
        &ResourceCheckEngine<'_>,
        &RawCellAddressAliases,
        &ResourceOp,
        &Place,
    ) -> bool,
) -> bool {
    let mut raw_aliases = initial_raw_aliases.clone();
    let mut function_aliases = FunctionAliasTable::default();
    for op in ops {
        if !op_preserves(engine, &raw_aliases, op, protected) {
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
