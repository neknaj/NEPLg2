use alloc::string::String;
use alloc::vec::Vec;

use super::effect_check::ResourceEffectBoundaryEngine;
use super::effect_counts::ResourceEffectCounts;
use super::effect_identity::{RawIdentityTable, RawPointerAliasTable};
use super::effect_raw_memory_identity::RawMemoryIdentityTable;
use super::function_alias::FunctionAliasTable;
use super::model::{Place, RawMemoryOp, ResourceFunction, ResourceModule, ResourceTerminator};
use super::summary_index::{FunctionSummary, SummaryIndex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawIdentityReturnSummary {
    pub(super) function: String,
    pub(super) parameter_indices: Vec<usize>,
    pub(super) internal_alloc_operations: Vec<RawMemoryOp>,
}

pub(super) type RawIdentityReturnSummaryIndex<'a> = SummaryIndex<'a, RawIdentityReturnSummary>;

impl FunctionSummary for RawIdentityReturnSummary {
    fn function_name(&self) -> &str {
        &self.function
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawPointerReturnSummary {
    pub(super) function: String,
    pub(super) parameter_indices: Vec<usize>,
}

pub(super) type RawPointerReturnSummaryIndex<'a> = SummaryIndex<'a, RawPointerReturnSummary>;

impl FunctionSummary for RawPointerReturnSummary {
    fn function_name(&self) -> &str {
        &self.function
    }
}

pub(super) fn compute_raw_identity_return_summaries(
    module: &ResourceModule,
    pointer_summaries: &[RawPointerReturnSummary],
) -> Vec<RawIdentityReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        let summary_index = RawIdentityReturnSummaryIndex::new(&summaries);
        let pointer_summary_index = RawPointerReturnSummaryIndex::new(pointer_summaries);
        for function in &module.functions {
            let mut parameter_indices = Vec::new();
            for (index, param) in function.params.iter().enumerate() {
                let mut identities = RawIdentityTable::default();
                identities.mark(&param.place, RawMemoryOp::Alloc);
                if function_returns_marked_identity(
                    function,
                    identities,
                    &summary_index,
                    &pointer_summary_index,
                ) {
                    parameter_indices.push(index);
                }
            }
            let internal_alloc_operations = function_returns_internal_alloc_identity_operations(
                function,
                &summary_index,
                &pointer_summary_index,
            );
            if !parameter_indices.is_empty() || !internal_alloc_operations.is_empty() {
                next.push(RawIdentityReturnSummary {
                    function: function.name.clone(),
                    parameter_indices,
                    internal_alloc_operations,
                });
            }
        }
        if next == summaries {
            return summaries;
        }
        summaries = next;
    }
    summaries
}

fn function_returns_internal_alloc_identity_operations(
    function: &ResourceFunction,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
    pointer_summaries: &RawPointerReturnSummaryIndex<'_>,
) -> Vec<RawMemoryOp> {
    let identities = RawIdentityTable::default();
    function_returned_identity_operations_with_engine(
        function,
        identities,
        summaries,
        pointer_summaries,
        true,
    )
}

fn function_returns_marked_identity(
    function: &ResourceFunction,
    identities: RawIdentityTable,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
    pointer_summaries: &RawPointerReturnSummaryIndex<'_>,
) -> bool {
    !function_returned_identity_operations_with_engine(
        function,
        identities,
        summaries,
        pointer_summaries,
        false,
    )
    .is_empty()
}

fn function_returned_identity_operations_with_engine(
    function: &ResourceFunction,
    mut identities: RawIdentityTable,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
    pointer_summaries: &RawPointerReturnSummaryIndex<'_>,
    track_alloc_identities: bool,
) -> Vec<RawMemoryOp> {
    let mut engine = ResourceEffectBoundaryEngine {
        function: function.name.as_str(),
        effect: function.effect,
        summaries,
        pointer_summaries,
        track_alloc_identities,
        diagnostics: Vec::new(),
        counts: ResourceEffectCounts::default(),
    };
    let mut function_aliases = FunctionAliasTable::default();
    let mut pointer_aliases = RawPointerAliasTable::default();
    let mut raw_memory_identities = RawMemoryIdentityTable::default();
    let mut operations = Vec::new();
    for block in &function.blocks {
        engine.check_ops(
            &mut identities,
            &mut pointer_aliases,
            &mut function_aliases,
            &mut raw_memory_identities,
            &block.ops,
        );
        if let ResourceTerminator::Return {
            value: Some(place), ..
        } = &block.terminator
        {
            push_unique_operations(&mut operations, &identities.operations(place));
        }
    }
    operations
}

pub(super) fn compute_raw_pointer_return_summaries(
    module: &ResourceModule,
) -> Vec<RawPointerReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        let summary_index = RawPointerReturnSummaryIndex::new(&summaries);
        for function in &module.functions {
            let mut parameter_indices = Vec::new();
            for (index, param) in function.params.iter().enumerate() {
                let mut pointer_aliases = RawPointerAliasTable::default();
                pointer_aliases.mark(&param.place);
                if function_returns_pointer_alias(
                    function,
                    &param.place,
                    pointer_aliases,
                    &summary_index,
                ) {
                    parameter_indices.push(index);
                }
            }
            if !parameter_indices.is_empty() {
                next.push(RawPointerReturnSummary {
                    function: function.name.clone(),
                    parameter_indices,
                });
            }
        }
        if next == summaries {
            return summaries;
        }
        summaries = next;
    }
    summaries
}

fn function_returns_pointer_alias(
    function: &ResourceFunction,
    parameter: &Place,
    mut pointer_aliases: RawPointerAliasTable,
    pointer_summaries: &RawPointerReturnSummaryIndex<'_>,
) -> bool {
    let empty_identity_summaries: &[RawIdentityReturnSummary] = &[];
    let empty_identity_summary_index = RawIdentityReturnSummaryIndex::new(empty_identity_summaries);
    let mut engine = ResourceEffectBoundaryEngine {
        function: function.name.as_str(),
        effect: function.effect,
        summaries: &empty_identity_summary_index,
        pointer_summaries,
        track_alloc_identities: false,
        diagnostics: Vec::new(),
        counts: ResourceEffectCounts::default(),
    };
    let mut identities = RawIdentityTable::default();
    let mut function_aliases = FunctionAliasTable::default();
    let mut raw_memory_identities = RawMemoryIdentityTable::default();
    for block in &function.blocks {
        engine.check_ops(
            &mut identities,
            &mut pointer_aliases,
            &mut function_aliases,
            &mut raw_memory_identities,
            &block.ops,
        );
        if let ResourceTerminator::Return {
            value: Some(place), ..
        } = &block.terminator
        {
            if pointer_aliases.aliases(place, parameter) {
                return true;
            }
        }
    }
    false
}

fn push_unique_operations(target: &mut Vec<RawMemoryOp>, source: &[RawMemoryOp]) {
    for operation in source {
        if !target.contains(operation) {
            target.push(*operation);
        }
    }
    target.sort();
}
