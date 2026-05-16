use crate::hir::{HirExpr, HirExprKind};
use crate::layout::{struct_field_layout_by_name, tuple_field_layout};
use crate::types::{TypeCtx, TypeId};

use super::LowerCtx;

pub(super) fn aggregate_field_layout(
    types: &TypeCtx,
    ctx: &LowerCtx<'_>,
    base_ty: TypeId,
    field_expr: &HirExpr,
) -> Option<(TypeId, i64)> {
    match &field_expr.kind {
        HirExprKind::LiteralI32(index) if *index >= 0 => {
            tuple_field_layout(types, base_ty, *index as usize)
                .map(|field| (field.ty, field.offset as i64))
        }
        HirExprKind::LiteralStr(id) => {
            let field_name = ctx.strings.get(*id as usize)?;
            struct_field_layout_by_name(types, base_ty, field_name.as_str())
                .map(|field| (field.ty, field.offset as i64))
        }
        _ => None,
    }
}
