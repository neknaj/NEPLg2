use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::hir::{HirExpr, HirExprKind};
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

use super::diagnostics::type_error;
use super::syntax_helpers::parse_variant_name;
use super::{BlockChecker, StackEntry};

pub(super) enum ConstructorApplyResult {
    NotHandled,
    Handled(Option<StackEntry>),
}

macro_rules! constructor_apply_log {
    ($($arg:tt)*) => {{
        #[cfg(target_os = "none")]
        {
            let _ = core::format_args!($($arg)*);
        }
        #[cfg(not(target_os = "none"))]
        {
            std::eprintln!($($arg)*);
        }
    }};
}

impl<'a> BlockChecker<'a> {
    pub(super) fn apply_constructor_function(
        &mut self,
        name: &str,
        args: &[StackEntry],
        c_params: &[TypeId],
        resolved_args: &[TypeId],
        user_params: &[TypeId],
        c_result: TypeId,
        span: Span,
    ) -> ConstructorApplyResult {
        if let Some((enm, var)) = parse_variant_name(name) {
            if let Some(result) = self.apply_enum_constructor(
                enm,
                var,
                name,
                args,
                c_params,
                resolved_args,
                user_params,
                c_result,
                span,
            ) {
                return ConstructorApplyResult::Handled(result);
            }
        }
        if let Some(result) =
            self.apply_struct_constructor(name, args, c_params, resolved_args, span)
        {
            return ConstructorApplyResult::Handled(result);
        }
        ConstructorApplyResult::NotHandled
    }

    fn apply_enum_constructor(
        &mut self,
        enm: &str,
        var: &str,
        name: &str,
        args: &[StackEntry],
        c_params: &[TypeId],
        resolved_args: &[TypeId],
        user_params: &[TypeId],
        c_result: TypeId,
        span: Span,
    ) -> Option<Option<StackEntry>> {
        let enum_ty = self.enums.get(enm).and_then(|info| {
            info.variants
                .iter()
                .any(|v| v.name == var)
                .then_some(info.ty)
        })?;
        if crate::log::is_verbose() && enm == "Result" && var == "Ok" {
            constructor_apply_log!(
                "enum ctor debug: name={} resolved_args=[{}] user_params=[{}] arg_tys=[{}] c_result={}",
                name,
                resolved_args
                    .iter()
                    .map(|ty| self.ctx.type_to_string(*ty))
                    .collect::<Vec<_>>()
                    .join(", "),
                user_params
                    .iter()
                    .map(|ty| self.ctx.type_to_string(*ty))
                    .collect::<Vec<_>>()
                    .join(", "),
                args.iter()
                    .map(|arg| self.ctx.type_to_string(arg.ty))
                    .collect::<Vec<_>>()
                    .join(", "),
                self.ctx.type_to_string(c_result)
            );
        }
        if c_params.len() == 1 && args.len() != 1 {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::ArgumentArityMismatch,
                "constructor expects one argument",
                span,
            ));
            return Some(None);
        }
        if c_params.is_empty() && !args.is_empty() {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::ArgumentArityMismatch,
                "constructor takes no arguments",
                span,
            ));
            return Some(None);
        }
        let payload_expr = if c_params.len() == 1 {
            args.first().map(|a0| Box::new(a0.expr.clone()))
        } else {
            None
        };
        let type_args = resolved_args.to_vec();
        let applied_ty = if type_args.is_empty() {
            enum_ty
        } else {
            self.ctx.apply(enum_ty, type_args.clone())
        };
        Some(Some(StackEntry {
            ty: applied_ty,
            expr: HirExpr {
                ty: applied_ty,
                kind: HirExprKind::EnumConstruct {
                    name: enm.to_string(),
                    variant: var.to_string(),
                    type_args,
                    payload: payload_expr,
                },
                span,
            },
            type_args: Vec::new(),
            assign: None,
            auto_call: true,
        }))
    }

    fn apply_struct_constructor(
        &mut self,
        name: &str,
        args: &[StackEntry],
        c_params: &[TypeId],
        resolved_args: &[TypeId],
        span: Span,
    ) -> Option<Option<StackEntry>> {
        let Some(info) = self.structs.get(name) else {
            return None;
        };
        let struct_ty = info.ty;
        let fields = info.fields.clone();
        let field_names = info.field_names.clone();
        if args.len() != c_params.len() {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::ArgumentArityMismatch,
                "struct constructor arity mismatch",
                span,
            ));
            return Some(None);
        }
        let is_tag_unit_struct = fields.len() == 1
            && field_names.len() == 1
            && field_names[0] == "tag"
            && matches!(self.ctx.get(self.ctx.resolve_id(fields[0])), TypeKind::Unit);
        let type_args = resolved_args.to_vec();
        let applied_ty = if type_args.is_empty() {
            struct_ty
        } else {
            self.ctx.apply(struct_ty, type_args.clone())
        };
        let field_exprs = if is_tag_unit_struct && args.is_empty() {
            vec![HirExpr {
                ty: self.ctx.unit(),
                kind: HirExprKind::Unit,
                span,
            }]
        } else {
            args.iter().map(|a| a.expr.clone()).collect()
        };
        Some(Some(StackEntry {
            ty: applied_ty,
            expr: HirExpr {
                ty: applied_ty,
                kind: HirExprKind::StructConstruct {
                    name: name.to_string(),
                    type_args,
                    fields: field_exprs,
                },
                span,
            },
            type_args: Vec::new(),
            assign: None,
            auto_call: true,
        }))
    }
}
