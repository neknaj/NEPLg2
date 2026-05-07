extern crate alloc;

use alloc::string::String;

use crate::types::TypeId;

use super::borrow_state::{BorrowBinding, BorrowTable};
use super::model::{
    AggregateKind, Place, PlaceProjection, PlaceRoot, ResourceMatchPattern, ResourceOp,
    ResourceTerminator,
};
use super::place_utils::places_overlap;

pub(super) enum BorrowBindingFuture {
    Keep,
    Ended,
    Unused,
}

pub(super) fn scan_borrow_binding_future(
    ops: &[ResourceOp],
    binding: &BorrowBinding,
) -> BorrowBindingFuture {
    for op in ops {
        if op_keeps_borrow_binding(op, binding) {
            return BorrowBindingFuture::Keep;
        }
        if op_ends_borrow_binding(op, binding) {
            return BorrowBindingFuture::Ended;
        }
    }
    BorrowBindingFuture::Unused
}

fn op_keeps_borrow_binding(op: &ResourceOp, binding: &BorrowBinding) -> bool {
    resource_op_uses_place(op, &binding.token) || end_scope_checks_borrow_binding(op, binding)
}

fn op_ends_borrow_binding(op: &ResourceOp, binding: &BorrowBinding) -> bool {
    match op {
        ResourceOp::EndScope { locals, .. } => locals
            .iter()
            .any(|local| places_overlap(&binding.token, local)),
        _ => false,
    }
}

fn end_scope_checks_borrow_binding(op: &ResourceOp, binding: &BorrowBinding) -> bool {
    match op {
        ResourceOp::EndScope { locals, result, .. } => {
            let token_is_same_scope_local = locals
                .iter()
                .any(|local| places_overlap(&binding.token, local));
            let token_is_block_result = result
                .as_ref()
                .is_some_and(|result| places_overlap(result, &binding.token));
            let token_is_outer_local =
                matches!(binding.token.root, PlaceRoot::Local(_)) && !token_is_same_scope_local;
            let source_ends = locals
                .iter()
                .any(|local| places_overlap(local, &binding.source));
            source_ends && (token_is_block_result || token_is_outer_local)
        }
        _ => false,
    }
}

pub(super) fn terminator_uses_place(terminator: &ResourceTerminator, place: &Place) -> bool {
    match terminator {
        ResourceTerminator::Return { value, .. } => value
            .as_ref()
            .is_some_and(|value| place_mentions_token(value, place)),
        ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => false,
    }
}

pub(super) fn propagate_construct_borrow_tokens(
    borrows: &mut BorrowTable,
    output: &Place,
    kind: &AggregateKind,
    inputs: &[Place],
) {
    for (index, input) in inputs.iter().enumerate() {
        let Some(target) = aggregate_input_place(output, kind, index, input.ty) else {
            continue;
        };
        borrows.copy_or_move_token_tree(input, &target, false);
    }
}

pub(super) fn propagate_match_bind_borrow_token(
    borrows: &mut BorrowTable,
    op: &ResourceOp,
    pattern: &ResourceMatchPattern,
    bind_local: Option<&Place>,
) {
    let Some(bind_local) = bind_local else {
        return;
    };
    let ResourceOp::Match { scrutinee, .. } = op else {
        return;
    };
    let ResourceMatchPattern::Variant(variant) = pattern else {
        return;
    };
    let payload_variant = enum_payload_variant_name(variant);
    let payload = scrutinee.clone().with_projection(
        PlaceProjection::EnumPayload {
            variant: String::from(payload_variant),
        },
        bind_local.ty,
    );
    borrows.copy_or_move_token_tree(&payload, bind_local, false);
}

fn enum_payload_variant_name(pattern_variant: &str) -> &str {
    pattern_variant
        .rsplit("::")
        .next()
        .unwrap_or(pattern_variant)
}

fn resource_op_uses_place(op: &ResourceOp, place: &Place) -> bool {
    match op {
        ResourceOp::DeclareLocal {
            place: target,
            initializer,
            ..
        } => {
            place_mentions_token(target, place)
                || initializer
                    .as_ref()
                    .is_some_and(|initializer| place_mentions_token(initializer, place))
        }
        ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. } => {
            place_mentions_token(source, place) || place_mentions_token(output, place)
        }
        ResourceOp::Assign { target, value, .. } => {
            place_mentions_token(target, place) || place_mentions_token(value, place)
        }
        ResourceOp::Borrow { source, output, .. } => {
            place_mentions_token(source, place) || place_mentions_token(output, place)
        }
        ResourceOp::Drop { place: dropped, .. } => place_mentions_token(dropped, place),
        ResourceOp::EndScope { .. } => false,
        ResourceOp::Branch {
            output,
            condition,
            then_ops,
            then_value,
            else_ops,
            else_value,
            ..
        } => {
            place_mentions_token(output, place)
                || place_mentions_token(condition, place)
                || place_mentions_token(then_value, place)
                || place_mentions_token(else_value, place)
                || then_ops.iter().any(|op| resource_op_uses_place(op, place))
                || else_ops.iter().any(|op| resource_op_uses_place(op, place))
        }
        ResourceOp::Loop {
            condition,
            condition_ops,
            body_ops,
            ..
        } => {
            place_mentions_token(condition, place)
                || condition_ops
                    .iter()
                    .any(|op| resource_op_uses_place(op, place))
                || body_ops.iter().any(|op| resource_op_uses_place(op, place))
        }
        ResourceOp::Match {
            output,
            scrutinee,
            arms,
            ..
        } => {
            place_mentions_token(output, place)
                || place_mentions_token(scrutinee, place)
                || arms.iter().any(|arm| {
                    arm.bind_local
                        .as_ref()
                        .is_some_and(|bind| place_mentions_token(bind, place))
                        || place_mentions_token(&arm.value, place)
                        || arm.ops.iter().any(|op| resource_op_uses_place(op, place))
                })
        }
        ResourceOp::FunctionValue { output, .. } | ResourceOp::Expr { output, .. } => {
            place_mentions_token(output, place)
        }
        ResourceOp::Call { output, args, .. } => {
            place_mentions_token(output, place)
                || args.iter().any(|arg| place_mentions_token(arg, place))
        }
        ResourceOp::IndirectCall {
            output,
            callee,
            args,
            ..
        } => {
            place_mentions_token(output, place)
                || place_mentions_token(callee, place)
                || args.iter().any(|arg| place_mentions_token(arg, place))
        }
        ResourceOp::CallEffect { .. } => false,
        ResourceOp::StorageOrigin { target, .. } => place_mentions_token(target, place),
        ResourceOp::RawMemory { output, args, .. } => {
            place_mentions_token(output, place)
                || args.iter().any(|arg| place_mentions_token(arg, place))
        }
        ResourceOp::RawAddressAlias { source, target, .. }
        | ResourceOp::RawAddressView { source, target, .. } => {
            place_mentions_token(source, place) || place_mentions_token(target, place)
        }
        ResourceOp::Construct { output, inputs, .. } => {
            place_mentions_token(output, place)
                || inputs
                    .iter()
                    .any(|input| place_mentions_token(input, place))
        }
    }
}

fn place_mentions_token(place: &Place, token: &Place) -> bool {
    places_overlap(place, token)
}

fn aggregate_input_place(
    output: &Place,
    kind: &AggregateKind,
    index: usize,
    input_ty: TypeId,
) -> Option<Place> {
    match kind {
        AggregateKind::Enum { variant, .. } if index == 0 => Some(output.clone().with_projection(
            PlaceProjection::EnumPayload {
                variant: variant.clone(),
            },
            input_ty,
        )),
        AggregateKind::Struct { field_offsets, .. } => Some(output.clone().with_projection(
            PlaceProjection::Field {
                index,
                offset_bytes: *field_offsets.get(index)?,
            },
            input_ty,
        )),
        AggregateKind::Tuple { field_offsets } => Some(output.clone().with_projection(
            PlaceProjection::TupleField {
                index,
                offset_bytes: *field_offsets.get(index)?,
            },
            input_ty,
        )),
        _ => None,
    }
}
