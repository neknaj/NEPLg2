use super::borrow_state::BorrowTable;
use super::function_alias::FunctionAliasTable;
use super::model::{Place, PlaceRoot, ResourceCallTarget};
use super::summary::BorrowTokenReturnSummaryIndex;

pub(super) fn propagate_call_return_token(
    borrows: &mut BorrowTable,
    summaries: &BorrowTokenReturnSummaryIndex<'_>,
    output: &Place,
    target: &ResourceCallTarget,
    args: &[Place],
) {
    let ResourceCallTarget::User { name, .. } = target else {
        return;
    };
    let Some(summary) = summaries.get(name) else {
        return;
    };
    for arg in summary
        .parameter_indices
        .iter()
        .filter_map(|index| args.get(*index))
    {
        if borrows.copy_or_move_token_tree(arg, output, true) {
            return;
        }
    }
}

pub(super) fn propagate_indirect_call_return_token(
    borrows: &mut BorrowTable,
    function_aliases: &FunctionAliasTable,
    summaries: &BorrowTokenReturnSummaryIndex<'_>,
    output: &Place,
    callee: &Place,
    args: &[Place],
) {
    let functions = function_aliases.functions(callee);
    if functions.is_empty() {
        propagate_unknown_indirect_call_return_token(borrows, output, args);
        return;
    }
    for function in functions {
        if let Some(summary) = summaries.get(function) {
            for arg in summary
                .parameter_indices
                .iter()
                .filter_map(|index| args.get(*index))
            {
                if borrows.copy_or_move_token_tree(arg, output, true) {
                    return;
                }
            }
        }
    }
}

pub(super) fn release_call_temporary_argument_tokens(
    borrows: &mut BorrowTable,
    output: &Place,
    args: &[Place],
) {
    for arg in args
        .iter()
        .filter(|arg| *arg != output && matches!(arg.root, PlaceRoot::Temporary(_)))
    {
        borrows.release_token_tree(arg);
    }
}

fn propagate_unknown_indirect_call_return_token(
    borrows: &mut BorrowTable,
    output: &Place,
    args: &[Place],
) {
    for arg in args.iter().filter(|arg| arg.ty == output.ty) {
        if borrows.copy_or_move_token_tree(arg, output, true) {
            return;
        }
    }
}
