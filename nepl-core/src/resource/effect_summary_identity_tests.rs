use super::effect_identity::RawIdentityOrigin;
use super::effect_summary::{RawIdentityReturnProjection, RawIdentityReturnSummary};
use super::effect_summary_identity::filter_raw_identity_return_summary;
use super::model::{
    PlaceProjection, RawMemoryOp, ResourceBlock, ResourceBlockId, ResourceFunction,
    ResourceTerminator,
};
use crate::ast::Effect;
use crate::span::Span;
use crate::types::{EnumVariantInfo, TypeCtx, TypeKind};
use alloc::string::ToString;
use alloc::vec;

#[test]
fn identity_summary_filter_rejects_impossible_recursive_return_projection() {
    let mut types = TypeCtx::new();
    let result_ty = types.register_named(
        "Result".to_string(),
        TypeKind::Enum {
            name: "Result".to_string(),
            type_params: vec![],
            variants: vec![EnumVariantInfo {
                name: "Ok".to_string(),
                payload: Some(types.i32()),
            }],
        },
    );
    let mut summary = RawIdentityReturnSummary {
        function: "f".to_string(),
        parameter_returns: vec![],
        internal_alloc_returns: vec![
            RawIdentityReturnProjection {
                projections: vec![PlaceProjection::EnumPayload {
                    variant: "Ok".to_string(),
                }],
                ty: types.i32(),
                return_span: Span::dummy(),
                origins: vec![RawIdentityOrigin::new(RawMemoryOp::Alloc, Span::dummy())],
            },
            RawIdentityReturnProjection {
                projections: vec![
                    PlaceProjection::EnumPayload {
                        variant: "Ok".to_string(),
                    },
                    PlaceProjection::EnumPayload {
                        variant: "Ok".to_string(),
                    },
                ],
                ty: types.i32(),
                return_span: Span::dummy(),
                origins: vec![RawIdentityOrigin::new(RawMemoryOp::Alloc, Span::dummy())],
            },
        ],
    };

    filter_raw_identity_return_summary(&mut summary, &empty_function("f", result_ty), Some(&types));

    assert_eq!(summary.internal_alloc_returns.len(), 1);
    assert_eq!(
        summary.internal_alloc_returns[0].projections,
        vec![PlaceProjection::EnumPayload {
            variant: "Ok".to_string()
        }]
    );
}

fn empty_function(name: &str, result: crate::types::TypeId) -> ResourceFunction {
    ResourceFunction {
        name: name.to_string(),
        origin_name: name.to_string(),
        type_params: vec![],
        params: vec![],
        result,
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: vec![],
            terminator: ResourceTerminator::Return {
                value: None,
                span: Span::dummy(),
            },
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    }
}
