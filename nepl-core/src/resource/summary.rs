use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::borrow_state::BorrowTable;
use super::check::{ResourceBorrowCheckEngine, ResourceOwnerCheckEngine};
use super::function_alias::FunctionAliasTable;
use super::model::{
    OwnerState, Place, PlaceProjection, ResourceFunction, ResourceModule, ResourceTerminator,
    StorageId,
};
use super::owner_state::OwnerTable;
use super::place_utils::{place_suffix_after_prefix, push_unique_usize};
use super::report::{ResourceBorrowCheckDeferred, ResourceOwnerCheckDeferred};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BorrowTokenReturnSummary {
    pub(super) function: String,
    pub(super) parameter_indices: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerReturnSummary {
    pub(super) function: String,
    pub(super) parameter_indices: Vec<usize>,
    pub(super) returns_fresh_owner: bool,
    pub(super) projection_returns: Vec<OwnerProjectionReturnSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerProjectionReturnSummary {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) parameter_indices: Vec<usize>,
    pub(super) returns_fresh_owner: bool,
}

pub(super) fn compute_borrow_token_return_summaries(
    module: &ResourceModule,
) -> Vec<BorrowTokenReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let mut parameter_indices = Vec::new();
            for (index, param) in function.params.iter().enumerate() {
                if function_returns_borrow_token(function, &param.place, &summaries) {
                    parameter_indices.push(index);
                }
            }
            if !parameter_indices.is_empty() {
                next.push(BorrowTokenReturnSummary {
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

fn function_returns_borrow_token(
    function: &ResourceFunction,
    parameter: &Place,
    summaries: &[BorrowTokenReturnSummary],
) -> bool {
    let mut engine = ResourceBorrowCheckEngine {
        function: function.name.as_str(),
        summaries,
        diagnostics: Vec::new(),
        deferred: ResourceBorrowCheckDeferred::default(),
    };
    let mut borrows = BorrowTable::default();
    let mut function_aliases = FunctionAliasTable::default();
    borrows.add_shared(parameter, parameter);
    for block in &function.blocks {
        engine.check_ops(&mut borrows, &mut function_aliases, &block.ops);
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            if borrows
                .binding(value)
                .is_some_and(|binding| binding.source == *parameter)
            {
                return true;
            }
        }
    }
    false
}

pub(super) fn compute_owner_return_summaries(module: &ResourceModule) -> Vec<OwnerReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let summary = function_owner_return_summary(function, &summaries);
            if summary.returns_fresh_owner
                || !summary.parameter_indices.is_empty()
                || !summary.projection_returns.is_empty()
            {
                next.push(summary);
            }
        }
        if next == summaries {
            return summaries;
        }
        summaries = next;
    }
    summaries
}

fn function_owner_return_summary(
    function: &ResourceFunction,
    summaries: &[OwnerReturnSummary],
) -> OwnerReturnSummary {
    let mut engine = ResourceOwnerCheckEngine {
        function: function.name.as_str(),
        summaries,
        diagnostics: Vec::new(),
        deferred: ResourceOwnerCheckDeferred::default(),
    };
    let mut owners = OwnerTable::default();
    let mut parameter_storages = Vec::new();
    for param in &function.params {
        owners.allocate(&param.place);
        if let Some(OwnerState::Live { storage }) = owners.state(&param.place) {
            parameter_storages.push(storage);
        }
    }

    let mut parameter_indices = Vec::new();
    let mut returns_fresh_owner = false;
    let mut projection_returns = Vec::new();
    let mut function_aliases = FunctionAliasTable::default();
    for block in &function.blocks {
        engine.check_ops(&mut owners, &mut function_aliases, &block.ops);
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            match owners.state(value) {
                Some(OwnerState::Live { storage }) => {
                    if let Some(index) = parameter_storages
                        .iter()
                        .position(|parameter_storage| *parameter_storage == storage)
                    {
                        push_unique_usize(&mut parameter_indices, index);
                    } else {
                        returns_fresh_owner = true;
                    }
                }
                Some(OwnerState::MaybeFreed) => {
                    returns_fresh_owner = true;
                }
                Some(OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed)
                | None => {}
            }
            for entry in owners.descendant_entries(value) {
                if let OwnerState::Live { storage } = entry.state {
                    if let Some(suffix) = place_suffix_after_prefix(&entry.place, value) {
                        record_projection_owner_return(
                            &mut projection_returns,
                            suffix,
                            entry.place.ty,
                            storage,
                            &parameter_storages,
                        );
                    }
                }
            }
        }
    }

    OwnerReturnSummary {
        function: function.name.clone(),
        parameter_indices,
        returns_fresh_owner,
        projection_returns,
    }
}

fn record_projection_owner_return(
    projection_returns: &mut Vec<OwnerProjectionReturnSummary>,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
    storage: StorageId,
    parameter_storages: &[StorageId],
) {
    let entry_index = projection_returns
        .iter()
        .position(|entry| entry.suffix == suffix && entry.ty == ty)
        .unwrap_or_else(|| {
            projection_returns.push(OwnerProjectionReturnSummary {
                suffix: suffix.clone(),
                ty,
                parameter_indices: Vec::new(),
                returns_fresh_owner: false,
            });
            projection_returns.len() - 1
        });
    if let Some(parameter_index) = parameter_storages
        .iter()
        .position(|parameter_storage| *parameter_storage == storage)
    {
        push_unique_usize(
            &mut projection_returns[entry_index].parameter_indices,
            parameter_index,
        );
    } else {
        projection_returns[entry_index].returns_fresh_owner = true;
    }
}
