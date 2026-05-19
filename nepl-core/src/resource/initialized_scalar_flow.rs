extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::cell_state::CellTable;
use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_scalar_flow_ops::propagate_i32_scalar_ops;
use super::initialized_str_layout::seed_str_storage_layout;
use super::model::{
    Place, PlaceProjection, ResourceCallTarget, ResourceFunction, ResourceModule,
    ResourceTerminator,
};
use super::place_utils::{
    place_suffix_after_prefix, projected_place_with_concrete_type, reference_target_place,
    type_can_seed_raw_address_alias,
};
use super::summary_index::{FunctionSummary, SummaryIndex};
use super::summary_worklist::SummaryWorklist;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32ScalarReturnSummary {
    pub(super) function: String,
    pub(super) parameters: Vec<Place>,
    pub(super) aliases: Vec<I32ScalarReturnAlias>,
    pub(super) constant: Option<i32>,
}

pub(super) type I32ScalarReturnSummaryIndex<'a> = SummaryIndex<'a, I32ScalarReturnSummary>;

impl FunctionSummary for I32ScalarReturnSummary {
    fn function_name(&self) -> &str {
        &self.function
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32ScalarReturnAlias {
    pub(super) parameter_index: usize,
    pub(super) parameter_projection: Vec<PlaceProjection>,
    pub(super) scalar_ty: TypeId,
}

pub(super) fn compute_i32_scalar_return_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
) -> Vec<I32ScalarReturnSummary> {
    let mut worklist = SummaryWorklist::new(module);
    let mut summaries = Vec::new();
    while let Some(function_index) = worklist.pop() {
        let function = &module.functions[function_index];
        let scalar_summary_index = I32ScalarReturnSummaryIndex::new(&summaries);
        let summary = function_i32_scalar_return_summary(
            function,
            &scalar_summary_index,
            raw_alias_summaries,
            types,
        );
        if update_i32_scalar_return_summary(&mut summaries, summary) {
            worklist.notify_changed(function_index);
        }
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] resource_i32_scalar_summary_recomputations={} summaries={}",
            worklist.recomputations(),
            summaries.len()
        );
    }
    summaries
}

pub(super) fn apply_direct_call_i32_scalar_summary(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    target: &ResourceCallTarget,
    args: &[Place],
    summaries: &I32ScalarReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> bool {
    let ResourceCallTarget::User { name, .. } = target else {
        return false;
    };
    let Some(summary) = summaries.get(name) else {
        return false;
    };
    apply_i32_scalar_summary(raw_aliases, output, args, summary, types)
}

fn update_i32_scalar_return_summary(
    summaries: &mut Vec<I32ScalarReturnSummary>,
    summary: I32ScalarReturnSummary,
) -> bool {
    let has_facts = !summary.aliases.is_empty() || summary.constant.is_some();
    let position = summaries
        .iter()
        .position(|existing| existing.function == summary.function);
    match (has_facts, position) {
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

fn function_i32_scalar_return_summary(
    function: &ResourceFunction,
    scalar_summaries: &I32ScalarReturnSummaryIndex<'_>,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> I32ScalarReturnSummary {
    let mut guaranteed_aliases = None;
    let mut guaranteed_constant = None;
    for block in &function.blocks {
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut function_aliases = FunctionAliasTable::default();
        for param in &function.params {
            if type_can_seed_raw_address_alias(types, param.place.ty) {
                raw_aliases.mark(&param.place);
            }
            let mut cells = CellTable::default();
            seed_str_storage_layout(types, &mut cells, &mut raw_aliases, &param.place);
            if let Some(target_ty) = reference_target_type(types, param.place.ty) {
                let target = reference_target_place(&param.place, target_ty);
                if type_can_seed_raw_address_alias(types, target.ty) {
                    raw_aliases.mark(&target);
                }
                seed_str_storage_layout(types, &mut cells, &mut raw_aliases, &target);
            }
        }
        propagate_i32_scalar_ops(
            &mut raw_aliases,
            &mut function_aliases,
            &block.ops,
            scalar_summaries,
            raw_alias_summaries,
            types,
        );
        if let ResourceTerminator::Return { value, .. } = &block.terminator {
            let path_aliases = value
                .as_ref()
                .map(|value| collect_i32_scalar_return_aliases(function, &raw_aliases, value))
                .unwrap_or_default();
            merge_guaranteed_facts(&mut guaranteed_aliases, path_aliases);
            let path_constant = value
                .as_ref()
                .and_then(|value| raw_aliases.i32_value(value));
            merge_guaranteed_constant(&mut guaranteed_constant, path_constant);
        }
    }
    I32ScalarReturnSummary {
        function: function.name.clone(),
        parameters: function
            .params
            .iter()
            .map(|param| param.place.clone())
            .collect(),
        aliases: guaranteed_aliases.unwrap_or_default(),
        constant: guaranteed_constant.flatten(),
    }
}

fn collect_i32_scalar_return_aliases(
    function: &ResourceFunction,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
) -> Vec<I32ScalarReturnAlias> {
    let mut aliases = Vec::new();
    for scalar_alias in raw_aliases.scalar_aliases_for_value(value) {
        for (parameter_index, param) in function.params.iter().enumerate() {
            let Some(parameter_projection) = place_suffix_after_prefix(&scalar_alias, &param.place)
            else {
                continue;
            };
            push_unique_i32_scalar_return_alias(
                &mut aliases,
                I32ScalarReturnAlias {
                    parameter_index,
                    parameter_projection,
                    scalar_ty: scalar_alias.ty,
                },
            );
        }
    }
    aliases
}

fn apply_i32_scalar_summary(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    args: &[Place],
    summary: &I32ScalarReturnSummary,
    types: &TypeCtx,
) -> bool {
    let mut applied = false;
    if let Some(value) = summary.constant {
        raw_aliases.set_i32_value(output, value);
        applied = true;
    }
    for (alias, arg) in summary
        .aliases
        .iter()
        .filter_map(|alias| args.get(alias.parameter_index).map(|arg| (alias, arg)))
    {
        let source = projected_place_with_concrete_type(
            types,
            arg,
            &alias.parameter_projection,
            alias.scalar_ty,
        );
        raw_aliases.copy_scalar_facts_if_tracked(&source, output);
        applied = true;
    }
    applied
}

fn push_unique_i32_scalar_return_alias(
    aliases: &mut Vec<I32ScalarReturnAlias>,
    alias: I32ScalarReturnAlias,
) {
    if aliases.iter().any(|existing| existing == &alias) {
        return;
    }
    aliases.push(alias);
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

fn merge_guaranteed_constant(guaranteed: &mut Option<Option<i32>>, path: Option<i32>) {
    match guaranteed {
        Some(existing) if *existing == path => {}
        Some(existing) => {
            *existing = None;
        }
        None => {
            *guaranteed = Some(path);
        }
    }
}

fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        crate::types::TypeKind::Reference(target, _) => Some(*target),
        _ => None,
    }
}
