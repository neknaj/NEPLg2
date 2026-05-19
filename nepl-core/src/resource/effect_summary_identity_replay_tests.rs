use alloc::{string::ToString, vec, vec::Vec};

use super::effect_summary::RawPointerReturnSummary;
use super::effect_summary_identity::compute_raw_identity_return_summaries;
use super::model::{
    EffectOp, Place, ResourceBlock, ResourceBlockId, ResourceCallTarget, ResourceFunction,
    ResourceId, ResourceLocal, ResourceModule, ResourceOp, ResourceTerminator,
};
use crate::ast::Effect;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId};

#[test]
fn raw_identity_summary_replay_uses_typectx_for_copy_moves() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    let i32_ty = types.i32();
    types.register_copy_impl_target(i32_ty);
    let span = Span::dummy();
    let module = ResourceModule {
        functions: vec![callee_function(i32_ty, span), caller_function(i32_ty, span)],
        entry: None,
        string_literals: Vec::new(),
    };
    let pointer_summaries = Vec::<RawPointerReturnSummary>::new();

    let summaries =
        compute_raw_identity_return_summaries(&module, &pointer_summaries, Some(&types));
    let caller_summary = summaries
        .iter()
        .find(|summary| summary.function == "caller")
        .expect("caller summary should preserve copy-moved identity");

    assert_eq!(caller_summary.parameter_returns.len(), 1);
    assert_eq!(caller_summary.parameter_returns[0].parameter_index, 0);
    assert!(caller_summary.parameter_returns[0]
        .source_projections
        .is_empty());
    assert!(caller_summary.parameter_returns[0]
        .return_projections
        .is_empty());
    assert_eq!(caller_summary.parameter_returns[0].source_ty, i32_ty);
    assert_eq!(caller_summary.parameter_returns[0].return_ty, i32_ty);
}

fn callee_function(i32_ty: TypeId, span: Span) -> ResourceFunction {
    let source = Place::local("p".to_string(), i32_ty);
    ResourceFunction {
        name: "callee".to_string(),
        origin_name: "callee".to_string(),
        type_params: Vec::new(),
        params: vec![param("p", i32_ty, &source)],
        result: i32_ty,
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: Vec::new(),
            terminator: ResourceTerminator::Return {
                value: Some(source),
                span,
            },
            span,
        }],
        span,
    }
}

fn caller_function(i32_ty: TypeId, span: Span) -> ResourceFunction {
    let source = Place::local("p".to_string(), i32_ty);
    let call_output = Place::temporary(ResourceId(2), i32_ty);
    ResourceFunction {
        name: "caller".to_string(),
        origin_name: "caller".to_string(),
        type_params: Vec::new(),
        params: vec![param("p", i32_ty, &source)],
        result: i32_ty,
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: vec![
                ResourceOp::Move {
                    source: source.clone(),
                    output: Place::temporary(ResourceId(1), i32_ty),
                    span,
                },
                ResourceOp::Call {
                    output: call_output.clone(),
                    target: ResourceCallTarget::User {
                        name: "callee".to_string(),
                        type_args: Vec::new(),
                    },
                    args: vec![source],
                    effect: EffectOp::Pure,
                    span,
                },
            ],
            terminator: ResourceTerminator::Return {
                value: Some(call_output),
                span,
            },
            span,
        }],
        span,
    }
}

fn param(name: &str, ty: TypeId, place: &Place) -> ResourceLocal {
    ResourceLocal {
        name: name.to_string(),
        ty,
        mutable: false,
        place: place.clone(),
    }
}
