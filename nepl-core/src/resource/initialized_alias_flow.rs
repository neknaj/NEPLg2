extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::initialized_alias_flow_raw::function_raw_cell_address_return_aliases;
use super::initialized_alias_flow_value_projection::function_value_projection_return_aliases;
use super::model::{Place, PlaceProjection, ResourceExprKind, ResourceFunction, ResourceModule};
use super::place_utils::type_preserves_raw_address_alias;
use super::summary_index::{FunctionSummary, SummaryIndex};
use super::summary_worklist::SummaryWorklist;

pub(super) use super::initialized_alias_flow_apply::{
    apply_direct_call_raw_alias_summary, apply_indirect_call_raw_alias_summary,
    construct_raw_cell_address_alias_fields,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellAddressReturnSummary {
    pub(super) function: String,
    pub(super) parameters: Vec<Place>,
    pub(super) aliases: Vec<RawCellAddressReturnAlias>,
}

pub(super) type RawCellAddressReturnSummaryIndex<'a> =
    SummaryIndex<'a, RawCellAddressReturnSummary>;

impl FunctionSummary for RawCellAddressReturnSummary {
    fn function_name(&self) -> &str {
        &self.function
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellAddressReturnAlias {
    pub(super) parameter_index: usize,
    pub(super) parameter_projection: Vec<PlaceProjection>,
    pub(super) parameter_ty: TypeId,
    pub(super) return_projection: Vec<PlaceProjection>,
    pub(super) return_ty: TypeId,
}

pub(super) fn expr_kind_preserves_raw_alias(kind: ResourceExprKind) -> bool {
    matches!(
        kind,
        ResourceExprKind::LocalRead
            | ResourceExprKind::Call
            | ResourceExprKind::IndirectCall
            | ResourceExprKind::Intrinsic
            | ResourceExprKind::Borrow
            | ResourceExprKind::Branch
            | ResourceExprKind::Match
            | ResourceExprKind::Construct
    )
}

pub(super) fn compute_raw_cell_address_return_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
) -> Vec<RawCellAddressReturnSummary> {
    let mut worklist = SummaryWorklist::new(module);
    let mut summaries = Vec::new();
    while let Some(function_index) = worklist.pop() {
        let function = &module.functions[function_index];
        let summary_index = RawCellAddressReturnSummaryIndex::new(&summaries);
        let summary = function_raw_cell_address_return_summary(function, &summary_index, types);
        if update_raw_cell_address_return_summary(&mut summaries, summary) {
            worklist.notify_changed(function_index);
        }
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] resource_raw_alias_summary_recomputations={} summaries={}",
            worklist.recomputations(),
            summaries.len()
        );
    }
    summaries
}

fn update_raw_cell_address_return_summary(
    summaries: &mut Vec<RawCellAddressReturnSummary>,
    summary: RawCellAddressReturnSummary,
) -> bool {
    let has_aliases = !summary.aliases.is_empty();
    let position = summaries
        .iter()
        .position(|existing| existing.function == summary.function);
    match (has_aliases, position) {
        (true, Some(index)) if summaries[index] == summary => false,
        (true, Some(index)) => {
            summaries[index] = summary;
            true
        }
        (true, None) => {
            summaries.push(summary);
            true
        }
        (false, Some(index)) => {
            summaries.remove(index);
            true
        }
        (false, None) => false,
    }
}

fn function_raw_cell_address_return_summary(
    function: &ResourceFunction,
    summary_index: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> RawCellAddressReturnSummary {
    let mut aliases = function_value_projection_return_aliases(function, summary_index, types);
    for (index, param) in function.params.iter().enumerate() {
        for alias in function_raw_cell_address_return_aliases(
            function,
            index,
            &param.place,
            summary_index,
            types,
        ) {
            push_unique_return_alias(&mut aliases, alias);
        }
    }
    RawCellAddressReturnSummary {
        function: function.name.clone(),
        parameters: function
            .params
            .iter()
            .map(|param| param.place.clone())
            .collect(),
        aliases,
    }
}

pub(super) fn expr_output_preserves_raw_alias(
    types: &TypeCtx,
    kind: ResourceExprKind,
    output: &Place,
) -> bool {
    matches!(kind, ResourceExprKind::Deref) && type_preserves_raw_address_alias(types, output.ty)
}

pub(super) fn push_unique_return_alias(
    aliases: &mut Vec<RawCellAddressReturnAlias>,
    alias: RawCellAddressReturnAlias,
) {
    if !aliases.iter().any(|existing| existing == &alias) {
        aliases.push(alias);
    }
}

#[cfg(test)]
#[path = "initialized_alias_flow_tests.rs"]
mod tests;
