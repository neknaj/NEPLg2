extern crate alloc;

use crate::types::TypeCtx;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_alias_flow_projection::substitute_summary_projection_offsets;
use super::model::{AggregateKind, Place, ResourceCallTarget};
use super::place_utils::{construct_aggregate_field_place, projected_place_with_concrete_type};

pub(super) fn construct_raw_cell_address_alias_fields(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    kind: &AggregateKind,
    inputs: &[Place],
) {
    for (index, input) in inputs.iter().enumerate() {
        let field = construct_aggregate_field_place(output, kind, index, input);
        raw_aliases.copy_alias_if_tracked(input, &field);
    }
}

pub(super) fn apply_direct_call_raw_alias_summary(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    target: &ResourceCallTarget,
    args: &[Place],
    summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> bool {
    let ResourceCallTarget::User { name, .. } = target else {
        return false;
    };
    let Some(summary) = summaries.get(name) else {
        return false;
    };
    apply_raw_alias_summary(raw_aliases, output, args, summary, types)
}

pub(super) fn apply_indirect_call_raw_alias_summary(
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &FunctionAliasTable,
    output: &Place,
    callee: &Place,
    args: &[Place],
    summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> bool {
    let functions = function_aliases.functions(callee);
    let mut applied = false;
    for function in functions {
        if let Some(summary) = summaries.get(function) {
            applied |= apply_raw_alias_summary(raw_aliases, output, args, summary, types);
        }
    }
    applied
}

fn apply_raw_alias_summary(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    args: &[Place],
    summary: &super::initialized_alias_flow::RawCellAddressReturnSummary,
    types: &TypeCtx,
) -> bool {
    let mut applied = false;
    for (alias, arg) in summary
        .aliases
        .iter()
        .filter_map(|alias| args.get(alias.parameter_index).map(|arg| (alias, arg)))
    {
        let parameter_projection = substitute_summary_projection_offsets(
            raw_aliases,
            &alias.parameter_projection,
            summary,
            args,
        );
        let source = projected_place_with_concrete_type(
            types,
            arg,
            &parameter_projection,
            alias.parameter_ty,
        );
        let return_fallback_ty = if alias.return_ty == alias.parameter_ty {
            source.ty
        } else {
            alias.return_ty
        };
        let return_projection = substitute_summary_projection_offsets(
            raw_aliases,
            &alias.return_projection,
            summary,
            args,
        );
        let target = projected_place_with_concrete_type(
            types,
            output,
            &return_projection,
            return_fallback_ty,
        );
        raw_aliases.copy_alias_if_tracked(&source, &target);
        applied = true;
    }
    applied
}
