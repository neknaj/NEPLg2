extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::cell_state::CellTable;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummary;
use super::initialized_projection_domain::{
    normalize_storage_offsets, widen_projection, MAX_EXACT_PROJECTION_FACTS_PER_SHAPE,
};
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary::{
    RawCellDestructionParamAddress, RawCellInitializationParamCell,
    RawCellInitializationReturnCell, RawCellInitializationVariantCondition,
    RawCellInitializationVariantParamCell, RawCellInitializationVariantParamRange,
    RawCellInitializationVariantParamRequirement, RawCellMoveParamAddress,
};
use super::initialized_summary_cells::{
    collect_param_initialized_raw_cells, collect_return_initialized_raw_cells,
};
use super::initialized_summary_destruction::check_ops_and_collect_param_destructions;
use super::initialized_summary_variant_build::collect_variant_param_initialized_raw_cells_from_return;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{ResourceFunction, ResourceModule, ResourceTerminator};
use super::raw_address_seed::should_seed_raw_address_parameter;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckDeferred;

pub(super) fn compute_raw_cell_initialization_function_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
) -> Vec<RawCellInitializationFunctionSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut changed = false;
        for function in &module.functions {
            let mut summary = function_raw_cell_initialization_summary(
                function,
                types,
                raw_alias_summaries,
                &summaries,
            );
            normalize_raw_cell_initialization_summary(&mut summary);
            changed |= update_raw_cell_initialization_summary(&mut summaries, summary);
        }
        if !changed {
            return summaries;
        }
    }
    summaries
}

fn update_raw_cell_initialization_summary(
    summaries: &mut Vec<RawCellInitializationFunctionSummary>,
    summary: RawCellInitializationFunctionSummary,
) -> bool {
    match summaries.binary_search_by(|existing| existing.function.cmp(&summary.function)) {
        Ok(index) if raw_cell_initialization_summary_is_empty(&summary) => {
            summaries.remove(index);
            true
        }
        Ok(index) if summaries[index] != summary => {
            summaries[index] = summary;
            true
        }
        Ok(_) => false,
        Err(_) if raw_cell_initialization_summary_is_empty(&summary) => false,
        Err(index) => {
            summaries.insert(index, summary);
            true
        }
    }
}

fn raw_cell_initialization_summary_is_empty(
    summary: &RawCellInitializationFunctionSummary,
) -> bool {
    summary.return_cells.is_empty()
        && summary.param_cells.is_empty()
        && summary.variant_param_cells.is_empty()
        && summary.variant_param_ranges.is_empty()
        && summary.variant_required_param_cells.is_empty()
        && summary.variant_conditions.is_empty()
        && summary.param_destructions.is_empty()
        && summary.param_moves.is_empty()
}

fn function_raw_cell_initialization_summary(
    function: &ResourceFunction,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
    raw_init_summaries: &[RawCellInitializationFunctionSummary],
) -> RawCellInitializationFunctionSummary {
    let mut engine = ResourceCheckEngine {
        function: function.name.as_str(),
        types,
        raw_alias_summaries,
        raw_init_summaries,
        diagnostics: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
    };
    let mut cells = CellTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut function_aliases = FunctionAliasTable::default();
    let mut pending_reallocs = PendingRawReallocs::default();
    for param in &function.params {
        cells.mark_initialized(&param.place);
        if should_seed_raw_address_parameter(function, &param.place, types) {
            raw_aliases.mark(&param.place);
        }
    }

    let mut out = RawCellInitializationFunctionSummary {
        function: function.name.clone(),
        return_cells: Vec::new(),
        param_cells: Vec::new(),
        variant_param_cells: Vec::new(),
        variant_param_ranges: Vec::new(),
        variant_required_param_cells: Vec::new(),
        variant_conditions: Vec::new(),
        param_destructions: Vec::new(),
        param_moves: Vec::new(),
    };
    let mut guaranteed_return_cells = None;
    let mut guaranteed_param_cells = None;
    for block in &function.blocks {
        let mut variant_initializations = PendingVariantRawCellInitializations::default();
        check_ops_and_collect_param_destructions(
            &mut out.param_destructions,
            &mut out.param_moves,
            &mut engine,
            &mut cells,
            &mut raw_aliases,
            &mut function_aliases,
            &mut pending_reallocs,
            &mut variant_initializations,
            &function.params,
            raw_init_summaries,
            &block.ops,
        );
        if let ResourceTerminator::Return { value, .. } = &block.terminator {
            let mut path_return_cells = Vec::new();
            if let Some(value) = value {
                collect_return_initialized_raw_cells(
                    &mut path_return_cells,
                    &cells,
                    &raw_aliases,
                    value,
                );
            }
            merge_guaranteed_facts(&mut guaranteed_return_cells, path_return_cells);

            let mut path_param_cells = Vec::new();
            collect_param_initialized_raw_cells(
                &mut path_param_cells,
                &cells,
                &raw_aliases,
                &function.params,
            );
            merge_guaranteed_facts(&mut guaranteed_param_cells, path_param_cells);
        }
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            collect_variant_param_initialized_raw_cells_from_return(
                &mut out.variant_param_cells,
                &mut out.variant_param_ranges,
                &mut out.variant_required_param_cells,
                &mut out.variant_conditions,
                function,
                types,
                raw_alias_summaries,
                raw_init_summaries,
                &block.ops,
                value,
            );
        }
    }
    out.return_cells = guaranteed_return_cells.unwrap_or_default();
    out.param_cells = guaranteed_param_cells.unwrap_or_default();
    out
}

fn merge_guaranteed_facts<T: Clone + Eq>(guaranteed: &mut Option<Vec<T>>, path: Vec<T>) {
    match guaranteed {
        Some(existing) => {
            existing.retain(|fact| path.contains(fact));
        }
        None => {
            *guaranteed = Some(path);
        }
    }
}

fn normalize_raw_cell_initialization_summary(summary: &mut RawCellInitializationFunctionSummary) {
    summary.return_cells = normalize_unique(
        core::mem::take(&mut summary.return_cells),
        normalize_return_cell,
    );
    summary.param_cells = normalize_unique(
        core::mem::take(&mut summary.param_cells),
        normalize_param_cell,
    );
    summary.variant_param_cells = normalize_unique(
        core::mem::take(&mut summary.variant_param_cells),
        normalize_variant_param_cell,
    );
    summary.variant_param_ranges = normalize_unique(
        core::mem::take(&mut summary.variant_param_ranges),
        normalize_variant_param_range,
    );
    summary.variant_required_param_cells = normalize_unique(
        core::mem::take(&mut summary.variant_required_param_cells),
        normalize_variant_param_requirement,
    );
    summary.variant_conditions = normalize_unique(
        core::mem::take(&mut summary.variant_conditions),
        normalize_variant_condition,
    );
    summary.param_destructions = normalize_widened(
        core::mem::take(&mut summary.param_destructions),
        normalize_param_destruction,
        widen_param_destruction,
    );
    summary.param_moves = normalize_widened(
        core::mem::take(&mut summary.param_moves),
        normalize_param_move,
        widen_param_move,
    );
}

fn normalize_unique<T: Ord>(input: Vec<T>, normalize: fn(T) -> T) -> Vec<T> {
    let mut out = input.into_iter().map(normalize).collect::<Vec<_>>();
    out.sort();
    out.dedup();
    out
}

fn normalize_widened<T: Ord>(
    input: Vec<T>,
    normalize: fn(T) -> T,
    widen: fn(&T, &T) -> Option<T>,
) -> Vec<T> {
    let mut input = input.into_iter().map(normalize).collect::<Vec<_>>();
    input.sort();
    let mut out = Vec::new();
    for item in input {
        push_widened(&mut out, item, widen);
    }
    out.sort();
    out.dedup();
    out
}

fn push_widened<T: Ord>(out: &mut Vec<T>, item: T, widen: fn(&T, &T) -> Option<T>) {
    if out.iter().any(|existing| existing == &item) {
        return;
    }
    let compatible_count = out
        .iter()
        .filter(|existing| widen(existing, &item).is_some())
        .count();
    if compatible_count >= MAX_EXACT_PROJECTION_FACTS_PER_SHAPE {
        if let Some(widened) = out.iter().find_map(|existing| widen(existing, &item)) {
            if !out.iter().any(|existing| existing == &widened) {
                out.push(widened);
            }
            return;
        }
    }
    out.push(item);
}

fn normalize_return_cell(
    mut cell: RawCellInitializationReturnCell,
) -> RawCellInitializationReturnCell {
    cell.suffix = normalize_storage_offsets(cell.suffix);
    cell
}

fn normalize_param_cell(
    mut cell: RawCellInitializationParamCell,
) -> RawCellInitializationParamCell {
    cell.suffix = normalize_storage_offsets(cell.suffix);
    cell
}

fn normalize_variant_param_cell(
    mut cell: RawCellInitializationVariantParamCell,
) -> RawCellInitializationVariantParamCell {
    cell.suffix = normalize_storage_offsets(cell.suffix);
    cell
}

fn normalize_variant_param_range(
    mut range: RawCellInitializationVariantParamRange,
) -> RawCellInitializationVariantParamRange {
    range.address_suffix = normalize_storage_offsets(range.address_suffix);
    range.count_suffix = normalize_storage_offsets(range.count_suffix);
    range
}

fn normalize_variant_param_requirement(
    mut requirement: RawCellInitializationVariantParamRequirement,
) -> RawCellInitializationVariantParamRequirement {
    requirement.suffix = normalize_storage_offsets(requirement.suffix);
    requirement
}

fn normalize_variant_condition(
    mut condition: RawCellInitializationVariantCondition,
) -> RawCellInitializationVariantCondition {
    condition.suffix = normalize_storage_offsets(condition.suffix);
    condition
}

fn normalize_param_destruction(
    mut destruction: RawCellDestructionParamAddress,
) -> RawCellDestructionParamAddress {
    destruction.suffix = normalize_storage_offsets(destruction.suffix);
    destruction
}

fn widen_param_destruction(
    existing: &RawCellDestructionParamAddress,
    incoming: &RawCellDestructionParamAddress,
) -> Option<RawCellDestructionParamAddress> {
    if existing.param_index != incoming.param_index
        || existing.ty != incoming.ty
        || existing.operation != incoming.operation
    {
        return None;
    }
    Some(RawCellDestructionParamAddress {
        param_index: existing.param_index,
        suffix: widen_projection(&existing.suffix, &incoming.suffix)?,
        ty: existing.ty,
        operation: existing.operation,
    })
}

fn normalize_param_move(mut moved: RawCellMoveParamAddress) -> RawCellMoveParamAddress {
    moved.suffix = normalize_storage_offsets(moved.suffix);
    moved
}

fn widen_param_move(
    existing: &RawCellMoveParamAddress,
    incoming: &RawCellMoveParamAddress,
) -> Option<RawCellMoveParamAddress> {
    if existing.param_index != incoming.param_index
        || existing.address_ty != incoming.address_ty
        || existing.cell_ty != incoming.cell_ty
        || existing.operation != incoming.operation
    {
        return None;
    }
    Some(RawCellMoveParamAddress {
        param_index: existing.param_index,
        suffix: widen_projection(&existing.suffix, &incoming.suffix)?,
        address_ty: existing.address_ty,
        cell_ty: existing.cell_ty,
        operation: existing.operation,
    })
}

#[cfg(test)]
mod tests {
    use super::super::model::{PlaceProjection, ResourceOffset};
    use super::super::report::ResourceCheckOperation;
    use super::*;
    use crate::types::TypeId;
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn raw_cell_initialization_summary_normalization_uses_canonical_fact_order() {
        let ty = TypeId(1);
        let mut left = empty_summary("f");
        left.return_cells = vec![return_cell(8, ty), return_cell(0, ty), return_cell(8, ty)];
        left.param_moves = vec![param_move(16, ty), param_move(0, ty), param_move(16, ty)];

        let mut right = empty_summary("f");
        right.return_cells = vec![return_cell(0, ty), return_cell(8, ty)];
        right.param_moves = vec![param_move(0, ty), param_move(16, ty)];

        normalize_raw_cell_initialization_summary(&mut left);
        normalize_raw_cell_initialization_summary(&mut right);

        assert_eq!(left, right);
        assert_eq!(left.return_cells.len(), 2);
        assert_eq!(left.param_moves.len(), 2);
    }

    #[test]
    fn raw_cell_initialization_summary_update_keeps_canonical_function_order() {
        let ty = TypeId(1);
        let mut summaries = Vec::new();
        let mut b = empty_summary("b");
        b.return_cells.push(return_cell(0, ty));
        let mut a = empty_summary("a");
        a.return_cells.push(return_cell(0, ty));

        assert!(update_raw_cell_initialization_summary(
            &mut summaries,
            b.clone()
        ));
        assert!(update_raw_cell_initialization_summary(
            &mut summaries,
            a.clone()
        ));
        assert_eq!(summaries[0].function, "a");
        assert_eq!(summaries[1].function, "b");
        assert!(!update_raw_cell_initialization_summary(&mut summaries, a));
        assert!(update_raw_cell_initialization_summary(
            &mut summaries,
            empty_summary("b")
        ));
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].function, "a");
    }

    fn empty_summary(function: &str) -> RawCellInitializationFunctionSummary {
        RawCellInitializationFunctionSummary {
            function: function.to_string(),
            return_cells: Vec::new(),
            param_cells: Vec::new(),
            variant_param_cells: Vec::new(),
            variant_param_ranges: Vec::new(),
            variant_required_param_cells: Vec::new(),
            variant_conditions: Vec::new(),
            param_destructions: Vec::new(),
            param_moves: Vec::new(),
        }
    }

    fn return_cell(offset: usize, ty: TypeId) -> RawCellInitializationReturnCell {
        RawCellInitializationReturnCell {
            suffix: vec![PlaceProjection::StorageOffset(ResourceOffset::Exact(
                offset,
            ))],
            ty,
            holds_raw_address: true,
        }
    }

    fn param_move(offset: usize, ty: TypeId) -> RawCellMoveParamAddress {
        RawCellMoveParamAddress {
            param_index: 0,
            suffix: vec![PlaceProjection::StorageOffset(ResourceOffset::Exact(
                offset,
            ))],
            address_ty: ty,
            cell_ty: ty,
            operation: ResourceCheckOperation::RawMemoryLoadCell,
        }
    }
}
