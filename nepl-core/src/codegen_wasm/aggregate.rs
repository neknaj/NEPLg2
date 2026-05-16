use crate::hir::{HirExpr, HirExprKind};
use crate::layout::{struct_field_layout_by_name, tuple_field_layout};
use crate::types::{TypeCtx, TypeId};

use super::string_data::StringDataLayout;

pub(super) fn aggregate_field_layout(
    ctx: &TypeCtx,
    base_ty: TypeId,
    field_expr: &HirExpr,
    strings: &StringDataLayout,
) -> Option<(TypeId, u32)> {
    match &field_expr.kind {
        HirExprKind::LiteralI32(index) if *index >= 0 => {
            tuple_field_layout(ctx, base_ty, *index as usize)
                .map(|field| (field.ty, field.offset as u32))
        }
        HirExprKind::LiteralStr(id) => {
            let field_name = strings.literal_value(*id)?;
            struct_field_layout_by_name(ctx, base_ty, field_name)
                .map(|field| (field.ty, field.offset as u32))
        }
        _ => None,
    }
}
