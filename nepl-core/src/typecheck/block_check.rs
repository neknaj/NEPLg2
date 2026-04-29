use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec::Vec;

use crate::ast::{Block, PrefixExpr, PrefixItem, Stmt, Symbol};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::{DiagnosticCode, TypeDiagnosticCode};
use crate::hir::{HirBlock, HirExpr, HirExprKind, HirLine};
use crate::resolve::DefId;
use crate::types::{TypeId, TypeKind};

use super::binding_rules::{
    detect_field_accessor_fn, emit_shadow_warning, find_visible_nonshadow_same_signature_func,
    find_visible_same_signature_func, is_callable_binding, shadow_blocked_by_nonshadow,
};
use super::diagnostics::type_error;
use super::env::{Binding, BindingKind};
use super::syntax_helpers::gate_allows;
use super::traits::{format_trait_ref_name, TraitBoundRef};
use super::type_expr::type_from_expr;
use super::{check_function, BlockChecker, StackEntry};

fn block_check_dump_enabled() -> bool {
    #[cfg(target_os = "none")]
    {
        false
    }
    #[cfg(not(target_os = "none"))]
    {
        std::env::var("NEPL_DUMP_HIR").is_ok()
    }
}

macro_rules! block_check_log {
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

macro_rules! block_check_dump {
    ($($arg:tt)*) => {
        if block_check_dump_enabled() {
            block_check_log!($($arg)*);
        }
    };
}

impl<'a> BlockChecker<'a> {
    pub(super) fn check_block(
        &mut self,
        block: &Block,
        base_depth: usize,
        new_scope: bool,
        expected_last_ty: Option<TypeId>,
    ) -> Option<(HirBlock, Option<TypeId>)> {
        let old_effect = self.current_effect;

        if new_scope {
            self.env.push_scope();
        }

        // Hoist let (non-mut) and nested fn signatures
        for stmt in block.items.iter() {
            if let Stmt::Expr(PrefixExpr { items, .. })
            | Stmt::ExprSemi(PrefixExpr { items, .. }, _) = stmt
            {
                if let Some(PrefixItem::Symbol(Symbol::Let {
                    name,
                    mutable: false,
                    no_shadow,
                })) = items.first()
                {
                    if let Some(blocked) = shadow_blocked_by_nonshadow(self.env, &name.name) {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!("cannot shadow non-shadowable symbol '{}'", name.name),
                                name.span,
                            )
                            .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowViolation)),
                        );
                        self.diagnostics.push(
                            Diagnostic::error("non-shadowable declaration is here", blocked.span)
                                .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowViolation))
                                .with_secondary_label(name.span, Some("shadow attempt".into())),
                        );
                        continue;
                    }
                    if *no_shadow && self.env.lookup_any(&name.name).is_some() {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "noshadow declaration '{}' conflicts with existing symbol",
                                    name.name
                                ),
                                name.span,
                            )
                            .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowConflict)),
                        );
                        continue;
                    }
                    let ty = self.ctx.fresh_var(None);
                    emit_shadow_warning(
                        &mut self.diagnostics,
                        self.env,
                        &name.name,
                        name.span,
                        "let",
                    );
                    let _ = self.env.insert_local(Binding {
                        name: name.name.clone(),
                        ty,
                        mutable: false,
                        no_shadow: *no_shadow,
                        defined: false,
                        span: name.span,
                        kind: BindingKind::Var,
                    });
                    block_check_dump!("typecheck: hoisted binding {}", name.name);
                }
            } else if let Stmt::FnAlias(_) = stmt {
                // function alias is handled at top-level
            } else if let Stmt::FnDef(f) = stmt {
                if !f.type_params.is_empty() {
                    self.diagnostics.push(Diagnostic::error(
                        "nested generic functions are not supported yet",
                        f.name.span,
                    ));
                    continue;
                }
                let base_ty = type_from_expr(self.ctx, self.labels, &f.signature);
                let captures = self.collect_nested_fn_captures(f);
                let mut ty = base_ty;
                if let TypeKind::Function {
                    type_params,
                    params,
                    result,
                    effect,
                } = self.ctx.get(base_ty)
                {
                    if let Some(blocked) = shadow_blocked_by_nonshadow(self.env, &f.name.name) {
                        if is_callable_binding(blocked) {
                            if let Some(conflict) = find_visible_nonshadow_same_signature_func(
                                self.env,
                                self.import_resolution,
                                &f.name.name,
                                ty,
                                f.name.span,
                                self.ctx,
                            ) {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "cannot shadow non-shadowable function '{}' with same signature",
                                            f.name.name
                                        ),
                                        f.name.span,
                                    )
                                    .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowViolation)),
                                );
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "non-shadowable function declaration is here",
                                        conflict.span,
                                    )
                                    .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowViolation))
                                    .with_secondary_label(
                                        f.name.span,
                                        Some("shadow attempt".into()),
                                    ),
                                );
                                continue;
                            }
                            // 関数同名はオーバーロードとして扱う（異なるシグネチャは許可）。
                        } else {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    format!(
                                        "cannot shadow non-shadowable symbol '{}'",
                                        f.name.name
                                    ),
                                    f.name.span,
                                )
                                .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowViolation)),
                            );
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "non-shadowable declaration is here",
                                    blocked.span,
                                )
                                .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowViolation))
                                .with_secondary_label(f.name.span, Some("shadow attempt".into())),
                            );
                            continue;
                        }
                    }
                    if f.no_shadow
                        && (self
                            .env
                            .lookup_all_any_defined(&f.name.name)
                            .iter()
                            .any(|b| !is_callable_binding(b))
                            || find_visible_same_signature_func(
                                self.env,
                                self.import_resolution,
                                &f.name.name,
                                ty,
                                f.name.span,
                                self.ctx,
                            )
                            .is_some())
                    {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "noshadow declaration '{}' conflicts with existing symbol",
                                    f.name.name
                                ),
                                f.name.span,
                            )
                            .with_code(DiagnosticCode::Resolve(crate::diagnostic_codes::ResolveDiagnosticCode::ShadowNoShadowConflict)),
                        );
                        continue;
                    }
                    if !captures.is_empty() {
                        let mut lifted_params =
                            captures.iter().map(|(_, t)| *t).collect::<Vec<_>>();
                        lifted_params.extend(params.iter().copied());
                        ty = self
                            .ctx
                            .function(type_params.clone(), lifted_params, result, effect);
                    }
                    let has_non_callable_conflict = self
                        .env
                        .lookup_all_any_defined(&f.name.name)
                        .iter()
                        .any(|b| !is_callable_binding(b));
                    if has_non_callable_conflict {
                        emit_shadow_warning(
                            &mut self.diagnostics,
                            self.env,
                            &f.name.name,
                            f.name.span,
                            "fn",
                        );
                    }
                    let _ = self.env.insert_local(Binding {
                        name: f.name.name.clone(),
                        ty,
                        mutable: false,
                        no_shadow: f.no_shadow,
                        defined: true,
                        span: f.name.span,
                        kind: BindingKind::Func {
                            def_id: DefId::from_span(f.name.span),
                            symbol: f.name.name.clone(),
                            effect,
                            arity: f.params.len(),
                            builtin: None,
                            field_accessor: detect_field_accessor_fn(f),
                            type_param_bounds: BTreeMap::new(),
                            captures,
                        },
                    });
                }
            }
        }

        let mut lines = Vec::new();
        let mut stack: Vec<StackEntry> = Vec::new();
        for _ in 0..base_depth {
            stack.push(StackEntry {
                ty: self.ctx.unit(),
                expr: HirExpr {
                    ty: self.ctx.unit(),
                    kind: HirExprKind::Unit,
                    span: block.span,
                },
                type_args: Vec::new(),
                assign: None,
                auto_call: true,
            });
        }

        // Find the last expression statement index (it determines the block result)
        let last_expr_idx = block
            .items
            .iter()
            .rposition(|s| matches!(s, Stmt::Expr(_) | Stmt::ExprSemi(_, _)));

        let mut pending_if: Option<bool> = None;
        for (idx, stmt) in block.items.iter().enumerate() {
            if let Stmt::Directive(d) = stmt {
                if let Some(allowed) = gate_allows(d, self.target, self.profile) {
                    pending_if = Some(allowed);
                    continue;
                }
            }
            let allowed = pending_if.unwrap_or(true);
            pending_if = None;
            if !allowed {
                continue;
            }

            // Drop stray unit between lines: [X, ()] -> [X]
            if stack.len() == base_depth + 1 {
                if matches!(self.ctx.get(stack.last().unwrap().ty), TypeKind::Unit) {
                    stack.pop();
                }
            }

            match stmt {
                Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => {
                    let expected_stmt_ty = if Some(idx) == last_expr_idx {
                        expected_last_ty
                    } else {
                        None
                    };
                    match self.check_prefix(expr, base_depth, &mut stack, expected_stmt_ty) {
                        Some((typed, dropped_from_prefix)) => {
                            let is_last_expr = Some(idx) == last_expr_idx;
                            let mut drop_result = !is_last_expr;
                            if matches!(stmt, Stmt::ExprSemi(_, _)) {
                                // `;` explicitly discards the statement value even
                                // when it appears on the last line of a block.
                                drop_result = true;
                            }

                            if dropped_from_prefix {
                                self.diagnostics.push(type_error(
                                    TypeDiagnosticCode::StackExtraValues,
                                    "expression left extra values on the stack",
                                    typed.span,
                                ));
                            }

                            // If there was an explicit semicolon token, require that the
                            // statement left exactly one value on the stack; otherwise
                            // emit a diagnostic and recover.
                            if let Stmt::ExprSemi(_, semi_span) = stmt {
                                if stack.len() != base_depth + 1 {
                                    let sp = semi_span.unwrap_or(typed.span);
                                    self.diagnostics.push(type_error(
                                        TypeDiagnosticCode::StackExtraValues,
                                        "statement must leave exactly one value on the stack",
                                        sp,
                                    ));
                                    while stack.len() > base_depth {
                                        stack.pop();
                                    }
                                    drop_result = true;
                                }
                            }

                            if drop_result {
                                // Pop all values down to base_depth
                                while stack.len() > base_depth {
                                    let _ = stack.pop();
                                }
                            }

                            // Previously a fallback was here; lower-let is handled in `check_prefix`

                            lines.push(HirLine {
                                expr: typed,
                                drop_result,
                            });
                        }
                        None => {}
                    }
                }
                Stmt::Directive(_) => {}
                Stmt::FnAlias(_) => {}
                Stmt::FnDef(f) => {
                    // capture 型はホイスト時点では未確定になり得るため、
                    // 実チェック時に現在の環境から再計算する。
                    let captures = self.collect_nested_fn_captures(f);
                    let mut f_ty = type_from_expr(self.ctx, self.labels, &f.signature);
                    if let TypeKind::Function {
                        type_params,
                        params,
                        result,
                        effect,
                    } = self.ctx.get(f_ty)
                    {
                        if !captures.is_empty() {
                            let mut lifted_params =
                                captures.iter().map(|(_, t)| *t).collect::<Vec<_>>();
                            lifted_params.extend(params.iter().copied());
                            f_ty = self.ctx.function(
                                type_params.clone(),
                                lifted_params,
                                result,
                                effect,
                            );
                        }
                    } else {
                        self.diagnostics.push(Diagnostic::error(
                            "function signature must be a function type",
                            f.name.span,
                        ));
                        continue;
                    }
                    let mut nested_bounds = BTreeMap::new();
                    if let TypeKind::Function { type_params, .. } = self.ctx.get(f_ty) {
                        for (p_node, p_id) in f.type_params.iter().zip(type_params.iter()) {
                            self.labels.insert(p_node.name.name.clone(), *p_id);
                            if !p_node.bounds.is_empty() {
                                let mut bounds = Vec::new();
                                for b in &p_node.bounds {
                                    if let Some(info) = self.traits.get(&b.name.name) {
                                        if info.type_params.len() != b.args.len() {
                                            self.diagnostics.push(type_error(
                                                TypeDiagnosticCode::TraitTypeParamsUnsupported,
                                                format!(
                                                    "trait bound '{}' expects {} type arguments, found {}",
                                                    b.name.name,
                                                    info.type_params.len(),
                                                    b.args.len()
                                                ),
                                                b.name.span,
                                            ));
                                            continue;
                                        }
                                        let arg_tys: Vec<TypeId> = b
                                            .args
                                            .iter()
                                            .map(|arg| {
                                                type_from_expr(self.ctx, &mut self.labels, arg)
                                            })
                                            .collect();
                                        bounds.push(TraitBoundRef {
                                            name: format_trait_ref_name(
                                                &b.name.name,
                                                &arg_tys,
                                                self.ctx,
                                            ),
                                            trait_base_name: b.name.name.clone(),
                                            trait_args: arg_tys,
                                            trait_self_ty: info.self_ty,
                                        });
                                    }
                                }
                                if !bounds.is_empty() {
                                    nested_bounds.insert(*p_id, bounds);
                                }
                            }
                        }
                    }
                    let _ = self.env.update_local_function_binding(
                        self.ctx,
                        &f.name.name,
                        f.name.span,
                        f_ty,
                        captures.clone(),
                    );
                    match check_function(
                        f,
                        f_ty,
                        false,
                        self.target,
                        self.profile,
                        captures.as_slice(),
                        self.ctx,
                        self.env,
                        self.labels,
                        self.string_table,
                        self.enums,
                        self.structs,
                        self.instantiations,
                        nested_bounds,
                        self.traits,
                        self.impls,
                        self.generated_functions,
                        self.import_resolution,
                        self.source_map,
                    ) {
                        Ok(checked) => {
                            self.diagnostics.extend(checked.diagnostics);
                            self.generated_functions.push(checked.function);
                        }
                        Err(mut diags) => self.diagnostics.append(&mut diags),
                    }
                }
                Stmt::StructDef(_) => {}
                Stmt::EnumDef(_) => {}
                Stmt::Wasm(_) => {
                    self.diagnostics.push(Diagnostic::error(
                        "wasm block is only allowed as a function body",
                        block.span,
                    ));
                }
                Stmt::LlvmIr(_) => {
                    self.diagnostics.push(Diagnostic::error(
                        "llvm ir block is only allowed as a function body",
                        block.span,
                    ));
                }
                Stmt::Trait(_) | Stmt::Impl(_) => {}
            }
        }

        // Handle final stack depth. Prefer to be forgiving: if there are
        // extra values on the stack, drop them with a warning rather than
        // failing hard. This keeps `:`-blocks and `if` branch combinations
        // usable while preserving diagnostics for surprising code.
        let final_ty: TypeId;
        let value_ty: Option<TypeId>;
        if stack.len() == base_depth {
            let u = self.ctx.unit();
            final_ty = u;
            value_ty = Some(u);
        } else if stack.len() == base_depth + 1 {
            let t = stack.last().unwrap().ty;
            final_ty = t;
            value_ty = Some(t);
        } else if stack.len() > base_depth + 1 {
            // Too many values: report an error and pop extras for recovery.
            let extras = stack.len() - (base_depth + 1);
            for _ in 0..extras {
                // Pop and ignore the extra value(s).
                stack.pop();
            }
            self.diagnostics.push(Diagnostic::error(
                "block left extra values on the stack",
                block.span,
            ));
            if stack.len() == base_depth {
                let u = self.ctx.unit();
                final_ty = u;
                value_ty = Some(u);
            } else {
                let t = stack.last().unwrap().ty;
                final_ty = t;
                value_ty = Some(t);
            }
        } else {
            // Fewer than expected: this is a hard error.
            self.diagnostics.push(Diagnostic::error(
                "block leaves inconsistent stack state",
                block.span,
            ));
            final_ty = self.ctx.unit();
            value_ty = None;
        };

        if block_check_dump_enabled() {
            block_check_dump!(
                "NEPL_DUMP_HIR: block span={:?} lines={} final_ty={:?} value_ty={:?}",
                block.span,
                lines.len(),
                final_ty,
                value_ty
            );
            // Print env scopes and a compact preview of the HIR lines for diagnosis
            block_check_dump!("NEPL_DUMP_HIR: env scopes=\n{:?}", self.env.scopes);
            for (i, l) in lines.iter().enumerate() {
                block_check_dump!(
                    "NEPL_DUMP_HIR: line {} -> expr.kind = {:?}, ty={:?}, drop={}",
                    i,
                    l.expr.kind,
                    l.expr.ty,
                    l.drop_result
                );
            }
        }

        if new_scope {
            self.env.pop_scope();
        }

        self.current_effect = old_effect;

        Some((
            HirBlock {
                lines,
                ty: final_ty,
                span: block.span,
            },
            value_ty,
        ))
    }
}
