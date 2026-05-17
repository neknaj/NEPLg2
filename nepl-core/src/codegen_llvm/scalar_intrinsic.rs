extern crate alloc;

use alloc::format;

use crate::diagnostic_codes::{BackendDiagnosticCode, DiagnosticCode, LlvmDiagnosticCode};
use crate::hir::HirExpr;
use crate::intrinsic_kinds::{ScalarIntrinsicBackendOp, ScalarIntrinsicKind};
use crate::span::Span;
use crate::types::TypeCtx;

use super::{llvm_codegen_error, lower_hir_expr, LlTy, LlValue, LlvmCodegenError, LowerCtx};

fn hir_unsupported_error(message: impl Into<alloc::string::String>) -> LlvmCodegenError {
    llvm_codegen_error(
        message,
        Span::dummy(),
        DiagnosticCode::Backend(BackendDiagnosticCode::Llvm(
            LlvmDiagnosticCode::HirUnsupported,
        )),
    )
}

fn expected_input_ty(op: ScalarIntrinsicBackendOp) -> LlTy {
    match op {
        ScalarIntrinsicBackendOp::F32ToI32 | ScalarIntrinsicBackendOp::ReinterpretF32AsI32 => {
            LlTy::F32
        }
        ScalarIntrinsicBackendOp::I64Identity => LlTy::I64,
        ScalarIntrinsicBackendOp::I32ToF32
        | ScalarIntrinsicBackendOp::I32ToU8
        | ScalarIntrinsicBackendOp::U8ToI32
        | ScalarIntrinsicBackendOp::I32Identity
        | ScalarIntrinsicBackendOp::ReinterpretI32AsF32 => LlTy::I32,
    }
}

pub(super) fn lower_scalar_intrinsic(
    types: &TypeCtx,
    ctx: &mut LowerCtx<'_>,
    kind: ScalarIntrinsicKind,
    args: &[HirExpr],
) -> Result<Option<LlValue>, LlvmCodegenError> {
    let expected_count = kind.argument_count();
    if args.len() != expected_count {
        return Err(hir_unsupported_error(format!(
            "internal compiler error: intrinsic {} expects {} argument(s) in '{}'",
            kind.intrinsic_name(),
            expected_count,
            ctx.function_name
        )));
    }

    let Some(v) = lower_hir_expr(types, ctx, &args[0])? else {
        return Err(hir_unsupported_error(format!(
            "internal compiler error: intrinsic {} value must produce a value in '{}'",
            kind.intrinsic_name(),
            ctx.function_name
        )));
    };

    let op = kind.backend_op();
    let expected_ty = expected_input_ty(op);
    if v.ty != expected_ty {
        return Err(hir_unsupported_error(format!(
            "internal compiler error: intrinsic {} expects {} representation in '{}' (got {:?})",
            kind.intrinsic_name(),
            expected_ty.ir(),
            ctx.function_name,
            v.ty
        )));
    }

    match op {
        ScalarIntrinsicBackendOp::I32ToF32 => {
            let out = ctx.next_tmp();
            ctx.push_line(&format!("  {} = sitofp i32 {} to float", out, v.repr));
            Ok(Some(LlValue {
                ty: LlTy::F32,
                repr: out,
            }))
        }
        ScalarIntrinsicBackendOp::I32ToU8 | ScalarIntrinsicBackendOp::U8ToI32 => {
            let out = ctx.next_tmp();
            ctx.push_line(&format!("  {} = and i32 {}, 255", out, v.repr));
            Ok(Some(LlValue {
                ty: LlTy::I32,
                repr: out,
            }))
        }
        ScalarIntrinsicBackendOp::F32ToI32 => {
            let out = ctx.next_tmp();
            ctx.push_line(&format!("  {} = fptosi float {} to i32", out, v.repr));
            Ok(Some(LlValue {
                ty: LlTy::I32,
                repr: out,
            }))
        }
        ScalarIntrinsicBackendOp::I32Identity | ScalarIntrinsicBackendOp::I64Identity => {
            Ok(Some(v))
        }
        ScalarIntrinsicBackendOp::ReinterpretI32AsF32 => {
            let out = ctx.next_tmp();
            ctx.push_line(&format!("  {} = bitcast i32 {} to float", out, v.repr));
            Ok(Some(LlValue {
                ty: LlTy::F32,
                repr: out,
            }))
        }
        ScalarIntrinsicBackendOp::ReinterpretF32AsI32 => {
            let out = ctx.next_tmp();
            ctx.push_line(&format!("  {} = bitcast float {} to i32", out, v.repr));
            Ok(Some(LlValue {
                ty: LlTy::I32,
                repr: out,
            }))
        }
    }
}
