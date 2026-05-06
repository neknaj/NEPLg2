extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;

use super::borrow_state::BorrowTable;
use super::model::{BorrowState, Place, PlaceRoot};
use super::place_utils::places_overlap;
use super::report::{ResourceBorrowDiagnostic, ResourceBorrowOperation};

pub(super) fn check_end_scope(
    function: &str,
    diagnostics: &mut Vec<ResourceBorrowDiagnostic>,
    borrows: &mut BorrowTable,
    locals: &[Place],
    result: Option<&Place>,
    span: Span,
) {
    for local in locals {
        for binding in borrows.bindings_with_source_overlapping(local) {
            let token_is_same_scope_local = locals
                .iter()
                .any(|scope_local| places_overlap(&binding.token, scope_local));
            if token_is_same_scope_local {
                continue;
            }
            let token_is_block_result =
                result.is_some_and(|result| places_overlap(result, &binding.token));
            let token_is_outer_local = matches!(binding.token.root, PlaceRoot::Local(_));
            if !token_is_block_result && !token_is_outer_local {
                continue;
            }
            let active = borrows.state(&binding.source);
            if matches!(
                active,
                BorrowState::Shared { .. } | BorrowState::Unique { .. }
            ) {
                diagnostics.push(ResourceBorrowDiagnostic::BorrowConflict {
                    function: String::from(function),
                    operation: ResourceBorrowOperation::ReturnValue,
                    place: binding.token,
                    active,
                    span,
                });
            }
        }
    }
    for local in locals {
        borrows.release_token_tree(local);
    }
}
