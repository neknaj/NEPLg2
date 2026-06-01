use alloc::string::ToString;
use alloc::vec;

use super::lower_hir_module;
use crate::ast::Effect;
use crate::hir::{HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirLine, HirModule};
use crate::resource::model::{EffectOp, PrivateEffectRegion, ResourceOp};
use crate::resource::PrivateCacheOp;
use crate::span::{FileId, Span};
use crate::types::TypeCtx;

#[test]
fn private_cache_intrinsic_lowers_call_effect_at_expression_span_for_all_ops() {
    for operation in PrivateCacheOp::ALL {
        let mut types = TypeCtx::new();
        let unit_ty = types.unit();
        let intrinsic_span = Span::new(FileId(0), 10, 42);
        let expr = HirExpr {
            ty: unit_ty,
            kind: HirExprKind::Intrinsic {
                name: operation.intrinsic_name().to_string(),
                type_args: vec![],
                args: vec![],
            },
            span: intrinsic_span,
        };
        let module = HirModule {
            functions: vec![HirFunction {
                doc: None,
                name: "main".to_string(),
                origin_name: "main".to_string(),
                func_ty: types.function(vec![], vec![], unit_ty, Effect::Impure),
                params: vec![],
                result: unit_ty,
                effect: Effect::Impure,
                body: HirBody::Block(HirBlock {
                    lines: vec![HirLine {
                        expr,
                        drop_result: true,
                    }],
                    ty: unit_ty,
                    span: intrinsic_span,
                }),
                span: intrinsic_span,
            }],
            entry: Some("main".to_string()),
            externs: vec![],
            string_literals: vec![],
            traits: vec![],
            impls: vec![],
        };

        let resource = lower_hir_module(&module, &types);
        let call_effect = resource.functions[0].blocks[0]
            .ops
            .iter()
            .find_map(|op| match op {
                ResourceOp::CallEffect { effect, span } => Some((effect, *span)),
                _ => None,
            })
            .expect("private cache intrinsic must produce a Resource IR effect operation");

        assert_eq!(
            call_effect,
            (
                &EffectOp::PrivateCache {
                    operation,
                    region: PrivateEffectRegion::UnsealedIntrinsic,
                },
                intrinsic_span,
            )
        );
    }
}
