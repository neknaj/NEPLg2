extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeCtx;

use super::lower_temporary_scope_op::push_copy_state_only_temporary_from_op;
use super::model::{Place, ResourceOp};

pub(super) fn push_line_copy_state_only_temporary_scope(
    types: &TypeCtx,
    ops: &mut Vec<ResourceOp>,
    op_start: usize,
    result: Option<Place>,
    span: Span,
) {
    let temporary_locals = copy_state_only_temporaries_from_ops(types, &ops[op_start..]);
    if temporary_locals.is_empty() {
        return;
    }
    ops.push(ResourceOp::EndScope {
        locals: temporary_locals,
        result,
        span,
    });
}

fn copy_state_only_temporaries_from_ops(types: &TypeCtx, ops: &[ResourceOp]) -> Vec<Place> {
    let mut temporaries = Vec::new();
    for op in ops {
        push_copy_state_only_temporary_from_op(types, op, &mut temporaries);
    }
    temporaries
}
