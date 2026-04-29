use alloc::vec::Vec;

use super::borrow_check::ResourceBorrowCheckEngine;
use super::borrow_state::BorrowTable;
use super::function_alias::FunctionAliasTable;
use super::model::{Place, ResourceFunction, ResourceModule, ResourceTerminator};
use super::report::ResourceBorrowCheckDeferred;
use super::summary::BorrowTokenReturnSummary;

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
