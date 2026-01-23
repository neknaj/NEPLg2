#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::*;
use crate::builtins::BuiltinKind;
use crate::compiler::CompileTarget;
use crate::diagnostic::Diagnostic;
use crate::hir::*;
use crate::span::Span;
use crate::types::{EnumVariantInfo, TypeCtx, TypeId, TypeKind};

#[derive(Debug)]
pub struct TypeCheckResult {
    pub module: Option<HirModule>,
    pub diagnostics: Vec<Diagnostic>,
    pub types: TypeCtx,
}

#[derive(Debug, Clone)]
struct EnumInfo {
    ty: TypeId,
    variants: Vec<EnumVariantInfo>,
}

#[derive(Debug, Clone)]
struct StructInfo {
    ty: TypeId,
    fields: Vec<TypeId>,
}

pub fn typecheck(module: &crate::ast::Module, target: CompileTarget) -> TypeCheckResult {
    let mut ctx = TypeCtx::new();
    let mut label_env = LabelEnv::new();
    let mut env = Env::new();
    let mut diagnostics = Vec::new();
    let mut strings = StringTable::new();
    let mut enums: BTreeMap<String, EnumInfo> = BTreeMap::new();
    let mut structs: BTreeMap<String, StructInfo> = BTreeMap::new();

    let mut entry = None;
    let mut externs: Vec<HirExtern> = Vec::new();
    let mut pending_if: Option<bool> = None;
    for d in &module.directives {
        if let Directive::IfTarget { target: gate, .. } = d {
            pending_if = Some(target_allows(gate.as_str(), target));
            continue;
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if !allowed {
            continue;
        }
        if let Directive::Entry { name } = d {
            entry = Some(name.name.clone());
        } else if let Directive::Extern {
            module: m,
            name: n,
            func,
            signature,
            span,
        } = d
        {
            if matches!(target, CompileTarget::Wasm) && m == "wasi_snapshot_preview1" {
                diagnostics.push(Diagnostic::error(
                    "WASI import not allowed for wasm target (use #target wasi)",
                    *span,
                ));
                continue;
            }
            let ty = type_from_expr(&mut ctx, &mut label_env, signature);
            if let TypeKind::Function {
                params,
                result,
                effect,
            } = ctx.get(ty)
            {
                env.insert_global(Binding {
                    name: func.name.clone(),
                    ty,
                    mutable: false,
                    defined: true,
                    kind: BindingKind::Func {
                        effect,
                        arity: params.len(),
                        builtin: None,
                    },
                });
                externs.push(HirExtern {
                    module: m.clone(),
                    name: n.clone(),
                    local_name: func.name.clone(),
                    params,
                    result,
                    effect,
                    span: *span,
                });
            } else {
                diagnostics.push(Diagnostic::error(
                    "extern signature must be a function type",
                    *span,
                ));
            }
        }
    }

    // Register builtins (allocator / memory helpers) into the environment
    for b in crate::builtins::builtins(&mut ctx) {
        // insert into env
        env.insert_global(Binding {
            name: b.name.to_string(),
            ty: b.ty,
            mutable: false,
            defined: true,
            kind: BindingKind::Func {
                effect: b.effect,
                arity: func_arity(&ctx, b.ty),
                builtin: Some(b.kind),
            },
        });

        // add to externs so codegen imports them from the runtime module
        if let TypeKind::Function { params, result, effect } = ctx.get(b.ty) {
            externs.push(HirExtern {
                module: "nepl_alloc".to_string(),
                name: match b.name {
                    "alloc" => "alloc".to_string(),
                    "dealloc" => "dealloc".to_string(),
                    "realloc" => "realloc".to_string(),
                    "load_i32" => "load_i32".to_string(),
                    "store_i32" => "store_i32".to_string(),
                    _ => b.name.to_string(),
                },
                local_name: b.name.to_string(),
                params: params.clone(),
                result: result,
                effect: b.effect,
                span: crate::span::Span::dummy(),
            });
        }
    }

    // Collect top-level function signatures (hoist)
    // Also hoist struct/enum definitions
    let mut pending_if: Option<bool> = None;
    for item in &module.root.items {
        if let Stmt::Directive(Directive::IfTarget { target: gate, .. }) = item {
            pending_if = Some(target_allows(gate.as_str(), target));
            continue;
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if !allowed {
            continue;
        }
        match item {
            Stmt::EnumDef(e) => {
                if enums.contains_key(&e.name.name) {
                    diagnostics.push(Diagnostic::error(
                        "duplicate enum definition",
                        e.name.span,
                    ));
                    continue;
                }
                if env.lookup(&e.name.name).is_some() || structs.contains_key(&e.name.name) {
                    diagnostics.push(Diagnostic::error(
                        "name already used by another item",
                        e.name.span,
                    ));
                    continue;
                }
                let mut vars = Vec::new();
                for v in &e.variants {
                    let payload_ty = v
                        .payload
                        .as_ref()
                        .map(|p| type_from_expr(&mut ctx, &mut label_env, p));
                    vars.push(EnumVariantInfo {
                        name: v.name.name.clone(),
                        payload: payload_ty,
                    });
                }
                let ty = ctx.register_named(
                    e.name.name.clone(),
                    TypeKind::Enum {
                        name: e.name.name.clone(),
                        variants: vars.clone(),
                    },
                );
                enums.insert(
                    e.name.name.clone(),
                    EnumInfo {
                        ty,
                        variants: vars,
                    },
                );
            }
            Stmt::StructDef(s) => {
                if structs.contains_key(&s.name.name) {
                    diagnostics.push(Diagnostic::error(
                        "duplicate struct definition",
                        s.name.span,
                    ));
                    continue;
                }
                if env.lookup(&s.name.name).is_some() || enums.contains_key(&s.name.name) {
                    diagnostics.push(Diagnostic::error(
                        "name already used by another item",
                        s.name.span,
                    ));
                    continue;
                }
                let mut fs = Vec::new();
                for (_, ty_expr) in &s.fields {
                    fs.push(type_from_expr(&mut ctx, &mut label_env, ty_expr));
                }
                let ty = ctx.register_named(
                    s.name.name.clone(),
                    TypeKind::Struct {
                        name: s.name.name.clone(),
                        fields: fs.clone(),
                    },
                );
                structs.insert(
                    s.name.name.clone(),
                    StructInfo {
                        ty,
                        fields: fs,
                    },
                );
            }
            _ => {}
        }
    }

    // Constructors for enums/structs
    for (name, info) in enums.iter() {
        for (idx, var) in info.variants.iter().enumerate() {
            let params = var
                .payload
                .iter()
                .copied()
                .collect::<Vec<TypeId>>();
            let func_ty = ctx.function(params.clone(), info.ty, Effect::Pure);
            let vname = format!("{}::{}", name, var.name);
            env.insert_global(Binding {
                name: vname.clone(),
                ty: func_ty,
                mutable: false,
                defined: true,
                kind: BindingKind::Func {
                    effect: Effect::Pure,
                    arity: params.len(),
                    builtin: None,
                },
            });
        }
    }
    for (name, info) in structs.iter() {
        let func_ty = ctx.function(info.fields.clone(), info.ty, Effect::Pure);
        env.insert_global(Binding {
            name: name.clone(),
            ty: func_ty,
            mutable: false,
            defined: true,
            kind: BindingKind::Func {
                effect: Effect::Pure,
                arity: info.fields.len(),
                builtin: None,
            },
        });
    }

    let mut pending_if: Option<bool> = None;
    for item in &module.root.items {
        if let Stmt::Directive(Directive::IfTarget { target: gate, .. }) = item {
            pending_if = Some(target_allows(gate.as_str(), target));
            continue;
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if !allowed {
            continue;
        }
        if let Stmt::FnDef(f) = item {
            let ty = type_from_expr(&mut ctx, &mut label_env, &f.signature);
            if let TypeKind::Function {
                params,
                result,
                effect,
            } = ctx.get(ty)
            {
                if env.lookup(&f.name.name).is_some() {
                    diagnostics.push(Diagnostic::error(
                        "duplicate function definition",
                        f.name.span,
                    ));
                    continue;
                }
                if enums.contains_key(&f.name.name) || structs.contains_key(&f.name.name) {
                    diagnostics.push(Diagnostic::error(
                        "name already used by another item",
                        f.name.span,
                    ));
                    continue;
                }
                env.insert_global(Binding {
                    name: f.name.name.clone(),
                    ty,
                    mutable: false,
                    defined: true,
                    kind: BindingKind::Func {
                        effect,
                        arity: params.len(),
                        builtin: None,
                    },
                });
            } else {
                diagnostics.push(Diagnostic::error(
                    "function signature must be a function type",
                    f.name.span,
                ));
            }
        }
    }

    let mut functions = Vec::new();
    let mut pending_if = None;
    for item in &module.root.items {
        if let Stmt::Directive(Directive::IfTarget { target: gate, .. }) = item {
            pending_if = Some(target_allows(gate.as_str(), target));
            continue;
        }
        let allowed = pending_if.unwrap_or(true);
        pending_if = None;
        if !allowed {
            continue;
        }
        if let Stmt::FnDef(f) = item {
            match check_function(
                f,
                &mut ctx,
                &mut env,
                &mut label_env,
                &mut strings,
                &enums,
                &structs,
            ) {
                Ok(func) => functions.push(func),
                Err(mut diags) => diagnostics.append(&mut diags),
            }
        }
    }

    let has_error = diagnostics
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostic::Severity::Error));
    TypeCheckResult {
        module: if has_error {
            None
        } else {
            Some(HirModule {
                functions,
                entry,
                externs,
                string_literals: strings.into_vec(),
            })
        },
        diagnostics,
        types: ctx,
    }
}

// ---------------------------------------------------------------------
// Function checking
// ---------------------------------------------------------------------

fn check_function(
    f: &FnDef,
    ctx: &mut TypeCtx,
    env: &mut Env,
    labels: &mut LabelEnv,
    strings: &mut StringTable,
    enums: &BTreeMap<String, EnumInfo>,
    structs: &BTreeMap<String, StructInfo>,
) -> Result<HirFunction, Vec<Diagnostic>> {
    let mut diags = Vec::new();
    let sig_ty = type_from_expr(ctx, labels, &f.signature);
    let (params_ty, result_ty, effect) = match ctx.get(sig_ty) {
        TypeKind::Function {
            params,
            result,
            effect,
        } => (params, result, effect),
        _ => {
            diags.push(Diagnostic::error(
                "function signature must be a function type",
                f.name.span,
            ));
            return Err(diags);
        }
    };
    if params_ty.len() != f.params.len() {
        diags.push(Diagnostic::error(
            "parameter count mismatch with signature",
            f.name.span,
        ));
        return Err(diags);
    }

    env.push_scope();
    for (param, ty) in f.params.iter().zip(params_ty.iter()) {
        let _ = env.insert_local(Binding {
            name: param.name.clone(),
            ty: *ty,
            mutable: false,
            defined: true,
            kind: BindingKind::Var,
        });
    }

    let (body, diag_out) = {
        let mut checker = BlockChecker {
            ctx,
            env,
            labels,
            string_table: strings,
            diagnostics: Vec::new(),
            current_effect: effect,
            enums,
            structs,
        };

        let body_res = match &f.body {
            FnBody::Parsed(b) => match checker.check_block(b, 0, true) {
                Some((blk, _val)) => {
                    if checker.ctx.unify(blk.ty, result_ty).is_err() {
                        checker.diagnostics.push(Diagnostic::error(
                            "return type does not match signature",
                            f.name.span,
                        ));
                    }
                    HirBody::Block(blk)
                }
                None => {
                    return Err(checker.diagnostics);
                }
            },
            FnBody::Wasm(wb) => HirBody::Wasm(wb.clone()),
        };
        (body_res, checker.diagnostics)
    };

    env.pop_scope();
    if diag_out.is_empty() {
        Ok(HirFunction {
            name: f.name.name.clone(),
            params: f
                .params
                .iter()
                .zip(params_ty.iter())
                .map(|(p, ty)| HirParam {
                    name: p.name.clone(),
                    ty: *ty,
                    mutable: false,
                })
                .collect(),
            result: result_ty,
            effect,
            body,
            span: f.name.span,
        })
    } else {
        Err(diag_out)
    }
}

// ---------------------------------------------------------------------
// Block checker
// ---------------------------------------------------------------------

struct BlockChecker<'a> {
    ctx: &'a mut TypeCtx,
    env: &'a mut Env,
    labels: &'a mut LabelEnv,
    string_table: &'a mut StringTable,
    diagnostics: Vec<Diagnostic>,
    current_effect: Effect,
    enums: &'a BTreeMap<String, EnumInfo>,
    structs: &'a BTreeMap<String, StructInfo>,
}

impl<'a> BlockChecker<'a> {
    fn check_block(
        &mut self,
        block: &Block,
        base_depth: usize,
        new_scope: bool,
    ) -> Option<(HirBlock, Option<TypeId>)> {
        if new_scope {
            self.env.push_scope();
        }

        // Hoist let (non-mut) and nested fn signatures
        for stmt in &block.items {
            if let Stmt::Expr(PrefixExpr { items, .. }) = stmt {
                if let Some(PrefixItem::Symbol(Symbol::Let {
                    name,
                    mutable: false,
                })) = items.first()
                {
                    let ty = self.ctx.fresh_var(None);
                    let _ = self.env.insert_local(Binding {
                        name: name.name.clone(),
                        ty,
                        mutable: false,
                        defined: false,
                        kind: BindingKind::Var,
                    });
                }
            } else if let Stmt::FnDef(f) = stmt {
                let ty = type_from_expr(self.ctx, self.labels, &f.signature);
                if let TypeKind::Function { params, effect, .. } = self.ctx.get(ty) {
                    let _ = self.env.insert_local(Binding {
                        name: f.name.name.clone(),
                        ty,
                        mutable: false,
                        defined: true,
                        kind: BindingKind::Func {
                            effect,
                            arity: params.len(),
                            builtin: None,
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
                assign: None,
            });
        }

        for stmt in &block.items {
            // Drop stray unit between lines: [X, ()] -> [X]
            if stack.len() == base_depth + 1 {
                if matches!(self.ctx.get(stack.last().unwrap().ty), TypeKind::Unit) {
                    stack.pop();
                }
            }
            match stmt {
                Stmt::Expr(expr) => match self.check_prefix(expr, base_depth, &mut stack) {
                    Some((typed, dropped)) => {
                        lines.push(HirLine {
                            expr: typed,
                            drop_result: dropped,
                        });
                    }
                    None => {}
                },
                Stmt::Directive(_) => {}
                Stmt::FnDef(_) => {
                    // Nested function bodies are not type-checked here
                }
                Stmt::StructDef(_) => {}
                Stmt::EnumDef(_) => {}
                Stmt::Wasm(_) => {
                    self.diagnostics.push(Diagnostic::error(
                        "wasm block is only allowed as a function body",
                        block.span,
                    ));
                }
            }
        }

        // Handle final stack depth. Prefer to be forgiving: if there are
        // extra values on the stack, drop them with a warning rather than
        // failing hard. This keeps `:`-blocks and `if` branch combinations
        // usable while preserving diagnostics for surprising code.
        let mut final_ty: TypeId;
        let mut value_ty: Option<TypeId>;
        if stack.len() == base_depth {
            let u = self.ctx.unit();
            final_ty = u;
            value_ty = Some(u);
        } else if stack.len() == base_depth + 1 {
            let t = stack.last().unwrap().ty;
            final_ty = t;
            value_ty = Some(t);
        } else if stack.len() > base_depth + 1 {
            // Too many values: pop extras and emit a warning.
            let extras = stack.len() - (base_depth + 1);
            for _ in 0..extras {
                // Pop and ignore the extra value(s).
                stack.pop();
            }
            self.diagnostics.push(Diagnostic::warning(
                "block left extra values on the stack; dropping them",
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

        if new_scope {
            self.env.pop_scope();
        }

        if self.diagnostics.is_empty() {
            Some((
                HirBlock {
                    lines,
                    ty: final_ty,
                    span: block.span,
                },
                value_ty,
            ))
        } else {
            None
        }
    }

    fn check_prefix(
        &mut self,
        expr: &PrefixExpr,
        base_depth: usize,
        stack: &mut Vec<StackEntry>,
    ) -> Option<(HirExpr, bool)> {
        let mut dropped = false;
        let mut last_expr: Option<HirExpr> = None;
        let mut pipe_pending: Option<StackEntry> = None;
        let mut pending_ascription: Option<TypeId> = None;
        for item in &expr.items {
            match item {
                PrefixItem::Literal(lit, span) => {
                    let (ty, hir) = match lit {
                        Literal::Int(text) => {
                            let v = text.parse::<i32>().unwrap_or(0);
                            (self.ctx.i32(), HirExprKind::LiteralI32(v))
                        }
                        Literal::Float(text) => {
                            let v = text.parse::<f32>().unwrap_or(0.0);
                            (self.ctx.f32(), HirExprKind::LiteralF32(v))
                        }
                        Literal::Bool(b) => (self.ctx.bool(), HirExprKind::LiteralBool(*b)),
                        Literal::Str(s) => {
                            let id = self.string_table.intern(s.clone());
                            (self.ctx.str(), HirExprKind::LiteralStr(id))
                        }
                        Literal::Unit => (self.ctx.unit(), HirExprKind::Unit),
                    };
                    stack.push(StackEntry {
                        ty,
                        expr: HirExpr {
                            ty,
                            kind: hir,
                            span: *span,
                        },
                        assign: None,
                    });
                    if let Some(t) = pending_ascription.take() {
                        self.apply_ascription(stack, t, *span);
                    }
                    last_expr = Some(stack.last().unwrap().expr.clone());
                }
                PrefixItem::Symbol(sym) => match sym {
                    Symbol::Ident(id) => {
                        if let Some(binding) = self.env.lookup(&id.name) {
                            match &binding.kind {
                                BindingKind::Func {
                                    effect,
                                    arity,
                                    builtin,
                                } => {
                                    stack.push(StackEntry {
                                        ty: binding.ty,
                                        expr: HirExpr {
                                            ty: binding.ty,
                                            kind: HirExprKind::Var(id.name.clone()),
                                            span: id.span,
                                },
                                assign: None,
                            });
                            if let Some(t) = pending_ascription.take() {
                                if !matches!(self.ctx.get(binding.ty), TypeKind::Function { .. }) {
                                    self.apply_ascription(stack, t, id.span);
                                } else {
                                    pending_ascription = Some(t);
                                }
                            }
                            last_expr = Some(stack.last().unwrap().expr.clone());
                        }
                        BindingKind::Var => {
                            stack.push(StackEntry {
                                ty: binding.ty,
                                        expr: HirExpr {
                                            ty: binding.ty,
                                            kind: HirExprKind::Var(id.name.clone()),
                                            span: id.span,
                                },
                                assign: None,
                            });
                            if let Some(t) = pending_ascription.take() {
                                if !matches!(self.ctx.get(binding.ty), TypeKind::Function { .. }) {
                                    self.apply_ascription(stack, t, id.span);
                                } else {
                                    pending_ascription = Some(t);
                                }
                            }
                            last_expr = Some(stack.last().unwrap().expr.clone());
                        }
                            }
                        } else {
                            self.diagnostics
                                .push(Diagnostic::error("undefined identifier", id.span));
                        }
                    }
                    Symbol::Let { name, mutable } => {
                        let ty = if let Some(b) = self.env.lookup(&name.name) {
                            b.ty
                        } else {
                            let t = self.ctx.fresh_var(None);
                            let _ = self.env.insert_local(Binding {
                                name: name.name.clone(),
                                ty: t,
                                mutable: *mutable,
                                defined: false,
                                kind: BindingKind::Var,
                            });
                            t
                        };
                        let func_ty = self.ctx.function(vec![ty], self.ctx.unit(), Effect::Pure);
                        stack.push(StackEntry {
                            ty: func_ty,
                            expr: HirExpr {
                                ty: func_ty,
                                kind: HirExprKind::Var(name.name.clone()),
                                span: name.span,
                            },
                            assign: Some(AssignKind::Let),
                        });
                        if let Some(t) = pending_ascription.take() {
                            if !matches!(self.ctx.get(func_ty), TypeKind::Function { .. }) {
                                self.apply_ascription(stack, t, name.span);
                            } else {
                                pending_ascription = Some(t);
                            }
                        }
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                    Symbol::Set { name } => {
                        if let Some(binding) = self.env.lookup(&name.name) {
                            if !binding.mutable {
                                self.diagnostics.push(Diagnostic::error(
                                    "cannot set immutable variable",
                                    name.span,
                                ));
                            }
                            let func_ty = self.ctx.function(
                                vec![binding.ty],
                                self.ctx.unit(),
                                Effect::Impure,
                            );
                            stack.push(StackEntry {
                                ty: func_ty,
                                expr: HirExpr {
                                    ty: func_ty,
                                    kind: HirExprKind::Var(name.name.clone()),
                                    span: name.span,
                                },
                                assign: Some(AssignKind::Set),
                            });
                            if let Some(t) = pending_ascription.take() {
                                if !matches!(self.ctx.get(func_ty), TypeKind::Function { .. }) {
                                    self.apply_ascription(stack, t, name.span);
                                } else {
                                    pending_ascription = Some(t);
                                }
                            }
                            last_expr = Some(stack.last().unwrap().expr.clone());
                        } else {
                            self.diagnostics
                                .push(Diagnostic::error("undefined variable", name.span));
                        }
                    }
                    Symbol::If(sp) => {
                        let t_cond = self.ctx.bool();
                        let t_branch = self.ctx.fresh_var(None);
                        let func_ty = self.ctx.function(
                            vec![t_cond, t_branch, t_branch],
                            t_branch,
                            Effect::Pure,
                        );
                        stack.push(StackEntry {
                            ty: func_ty,
                            expr: HirExpr {
                                ty: func_ty,
                                kind: HirExprKind::Var("if".to_string()),
                                span: *sp,
                            },
                            assign: None,
                        });
                        if let Some(t) = pending_ascription.take() {
                            if !matches!(self.ctx.get(func_ty), TypeKind::Function { .. }) {
                                self.apply_ascription(stack, t, *sp);
                            } else {
                                pending_ascription = Some(t);
                            }
                        }
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                    Symbol::While(sp) => {
                        let t_cond = self.ctx.bool();
                        let func_ty = self.ctx.function(
                            vec![t_cond, self.ctx.unit()],
                            self.ctx.unit(),
                            Effect::Pure,
                        );
                        stack.push(StackEntry {
                            ty: func_ty,
                            expr: HirExpr {
                                ty: func_ty,
                                kind: HirExprKind::Var("while".to_string()),
                                span: *sp,
                            },
                            assign: None,
                        });
                        if let Some(t) = pending_ascription.take() {
                            if !matches!(self.ctx.get(func_ty), TypeKind::Function { .. }) {
                                self.apply_ascription(stack, t, *sp);
                            } else {
                                pending_ascription = Some(t);
                            }
                        }
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                },
                PrefixItem::TypeAnnotation(ty_expr, span) => {
                    let ty = type_from_expr(self.ctx, self.labels, ty_expr);
                    pending_ascription = Some(ty);
                    last_expr = Some(HirExpr {
                        ty,
                        kind: HirExprKind::Unit,
                        span: *span,
                    });
                }
                PrefixItem::Match(mexpr, sp) => {
                    if let Some((hexpr, ty)) = self.check_match_expr(mexpr) {
                        stack.push(StackEntry {
                            ty,
                            expr: hexpr,
                            assign: None,
                        });
                        if let Some(t) = pending_ascription.take() {
                            self.apply_ascription(stack, t, *sp);
                        }
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                }
                PrefixItem::Block(b, sp) => {
                    let (blk, val_ty) = self.check_block(b, stack.len(), true)?;
                    if let Some(ty) = val_ty {
                        stack.push(StackEntry {
                            ty,
                            expr: HirExpr {
                                ty,
                                kind: HirExprKind::Block(blk),
                                span: *sp,
                            },
                            assign: None,
                        });
                        if let Some(t) = pending_ascription.take() {
                            self.apply_ascription(stack, t, *sp);
                        }
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    } else {
                        last_expr = Some(HirExpr {
                            ty: self.ctx.unit(),
                            kind: HirExprKind::Block(blk),
                            span: *sp,
                        });
                    }
                }
                PrefixItem::Pipe(sp) => {
                    if pipe_pending.is_some() {
                        self.diagnostics.push(Diagnostic::error(
                            "pipe already pending; consecutive |> not allowed",
                            *sp,
                        ));
                        continue;
                    }
                    if stack.len() == base_depth {
                        self.diagnostics.push(Diagnostic::error(
                            "pipe requires a value on the stack",
                            *sp,
                        ));
                        continue;
                    }
                    pipe_pending = stack.pop();
                    last_expr = pipe_pending.as_ref().map(|se| se.expr.clone());
                }
                PrefixItem::Semi(sp) => {
                        if stack.len() >= base_depth + 1 {
                            // If there are extra values, drop them with a warning,
                            // then pop the single value that semicolon returns.
                            let extras = stack.len() - (base_depth + 1);
                            if extras > 0 {
                                for _ in 0..extras {
                                    stack.pop();
                                }
                                self.diagnostics.push(Diagnostic::warning(
                                    "semicolon encountered with extra values; dropping them",
                                    *sp,
                                ));
                            }
                            if let Some(se) = stack.pop() {
                                last_expr = Some(se.expr);
                            }
                            dropped = true;
                        } else {
                            self.diagnostics.push(Diagnostic::error(
                                "semicolon requires exactly one value on the stack",
                                *sp,
                            ));
                        }
                }
            }

            if !matches!(item, PrefixItem::Pipe(_) | PrefixItem::TypeAnnotation(_, _)) {
                if let Some(val) = pipe_pending.take() {
                    // The last pushed element should be a callable (function type)
                    if let Some(top) = stack.last() {
                        if matches!(self.ctx.get(top.ty), TypeKind::Function { .. }) {
                            stack.push(val);
                            last_expr = Some(stack.last().unwrap().expr.clone());
                        } else {
                            self.diagnostics.push(Diagnostic::error(
                                "pipe target must be a callable expression",
                                expr.span,
                            ));
                            stack.push(val);
                        }
                    } else {
                        self.diagnostics
                            .push(Diagnostic::error("pipe target missing", expr.span));
                        stack.push(val);
                    }
                }
            }

            self.reduce_calls(stack);

            if let Some(t) = pending_ascription {
                if stack.len() > base_depth {
                    if let Some(top) = stack.last() {
                        if !matches!(self.ctx.get(top.ty), TypeKind::Function { .. }) {
                            self.apply_ascription(stack, t, expr.span);
                            pending_ascription = None;
                        }
                    }
                }
            }
        }

        let result_expr = if stack.len() == base_depth + 1 {
            stack.last().unwrap().expr.clone()
        } else if let Some(e) = last_expr {
            e
        } else {
            HirExpr {
                ty: self.ctx.unit(),
                kind: HirExprKind::Unit,
                span: expr.span,
            }
        };

        if pipe_pending.is_some() {
            self.diagnostics
                .push(Diagnostic::error("pipe has no target", expr.span));
        }

        // Validate final stack depth. Be forgiving: if there are extra
        // values, pop them and emit a warning rather than a hard error.
        if stack.len() > base_depth + 1 {
            let extras = stack.len() - (base_depth + 1);
            for _ in 0..extras {
                stack.pop();
            }
            self.diagnostics.push(Diagnostic::warning(
                "expression leaves extra values on the stack; dropping them",
                expr.span,
            ));
        }

        Some((result_expr, dropped))
    }

    fn apply_ascription(&mut self, stack: &mut [StackEntry], target: TypeId, span: Span) {
        if let Some(top) = stack.last_mut() {
            if let Err(_) = self.ctx.unify(top.ty, target) {
                self.diagnostics
                    .push(Diagnostic::error("type annotation mismatch", span));
            } else {
                top.ty = target;
                top.expr.ty = target;
            }
        }
    }

    fn reduce_calls(&mut self, stack: &mut Vec<StackEntry>) {
        loop {
            let func_pos = match stack.iter().rposition(|e| match self.ctx.get(e.ty) {
                TypeKind::Function { .. } => true,
                _ => false,
            }) {
                Some(p) => p,
                None => break,
            };
            let func_ty = self.ctx.get(stack[func_pos].ty);
            let (params, result, effect) = match func_ty {
                TypeKind::Function {
                    params,
                    result,
                    effect,
                } => (params, result, effect),
                _ => break,
            };
            if stack.len() < func_pos + 1 + params.len() {
                break;
            }
            let mut args = Vec::new();
            for _ in 0..params.len() {
                args.push(stack.remove(func_pos + 1));
            }

            let applied = self.apply_function(stack.remove(func_pos), params, result, effect, args);
            if let Some(val) = applied {
                stack.insert(func_pos, val);
            } else {
                break;
            }
        }
    }

    fn check_match_expr(&mut self, m: &MatchExpr) -> Option<(HirExpr, TypeId)> {
        // evaluate scrutinee
        let mut tmp_stack = Vec::new();
        if let Some((scrut_expr, _)) = self.check_prefix(&m.scrutinee, 0, &mut tmp_stack) {
            let scrut_ty = scrut_expr.ty;
            let enum_info = match self.ctx.get(scrut_ty) {
                TypeKind::Enum { name, .. } => self.enums.get(&name),
                TypeKind::Named(name) => self.enums.get(&name),
                _ => None,
            };
            if enum_info.is_none() {
                self.diagnostics
                    .push(Diagnostic::error("match scrutinee must be an enum", m.span));
                return None;
            }
            let enum_info = enum_info.unwrap();
            let mut seen = alloc::collections::BTreeSet::new();
            let mut arms_hir = Vec::new();
            let mut result_ty: Option<TypeId> = None;
            for arm in &m.arms {
                if !seen.insert(arm.variant.name.clone()) {
                    self.diagnostics.push(Diagnostic::error(
                        "duplicate match arm",
                        arm.variant.span,
                    ));
                    continue;
                }
                let var_info = enum_info
                    .variants
                    .iter()
                    .find(|v| v.name == arm.variant.name);
                if var_info.is_none() {
                    self.diagnostics.push(Diagnostic::error(
                        "unknown enum variant in match",
                        arm.variant.span,
                    ));
                    continue;
                }
                let var_info = var_info.unwrap();
                self.env.push_scope();
                if let Some(bind) = &arm.bind {
                    if let Some(pty) = var_info.payload {
                        let _ = self.env.insert_local(Binding {
                            name: bind.name.clone(),
                            ty: pty,
                            mutable: false,
                            defined: true,
                            kind: BindingKind::Var,
                        });
                    } else {
                        self.diagnostics.push(Diagnostic::error(
                            "variant has no payload to bind",
                            bind.span,
                        ));
                    }
                }
                let (blk, val_ty) = self.check_block(&arm.body, 0, false)?;
                self.env.pop_scope();
                let body_ty = val_ty.unwrap_or(self.ctx.unit());
                if let Some(t) = result_ty {
                    if let Err(_) = self.ctx.unify(t, body_ty) {
                        self.diagnostics.push(Diagnostic::error(
                            "match arms have incompatible types",
                            arm.span,
                        ));
                    }
                } else {
                    result_ty = Some(body_ty);
                }
                arms_hir.push(HirMatchArm {
                    variant: arm.variant.name.clone(),
                    bind_local: arm.bind.as_ref().map(|b| b.name.clone()),
                    body: HirExpr {
                        ty: body_ty,
                        kind: HirExprKind::Block(blk),
                        span: arm.span,
                    },
                });
            }
            // exhaustiveness
            for v in &enum_info.variants {
                if !seen.contains(&v.name) {
                    self.diagnostics.push(Diagnostic::error(
                        "non-exhaustive match",
                        m.span,
                    ));
                    break;
                }
            }
            let rty = result_ty.unwrap_or(self.ctx.unit());
            return Some((
                HirExpr {
                    ty: rty,
                    kind: HirExprKind::Match {
                        scrutinee: Box::new(scrut_expr),
                        arms: arms_hir,
                    },
                    span: m.span,
                },
                rty,
            ));
        }
        None
    }

    fn apply_function(
        &mut self,
        func: StackEntry,
        params: Vec<TypeId>,
        result: TypeId,
        effect: Effect,
        args: Vec<StackEntry>,
    ) -> Option<StackEntry> {
        // Effect check
        if matches!(self.current_effect, Effect::Pure) && matches!(effect, Effect::Impure) {
            self.diagnostics.push(Diagnostic::error(
                "pure context cannot call impure function",
                func.expr.span,
            ));
            return None;
        }

        // Assignment operators
        if let Some(assign) = func.assign {
            if args.len() != 1 {
                self.diagnostics.push(Diagnostic::error(
                    "assignment expects one argument",
                    func.expr.span,
                ));
                return None;
            }
            let name = match &func.expr.kind {
                HirExprKind::Var(n) => n.clone(),
                _ => "_".to_string(),
            };
            if let Some(binding_vals) = self.env.lookup(&name).map(|b| (b.ty, b.mutable, b.defined))
            {
                let (b_ty, b_mut, b_defined) = binding_vals;
                if let Err(_) = self.ctx.unify(b_ty, args[0].ty) {
                    self.diagnostics.push(Diagnostic::error(
                        "type mismatch in assignment",
                        func.expr.span,
                    ));
                }
                match assign {
                    AssignKind::Let => {
                        if let Some(b) = self.env.lookup_mut(&name) {
                            b.defined = true;
                            b.ty = b_ty;
                        }
                        return Some(StackEntry {
                            ty: self.ctx.unit(),
                            expr: HirExpr {
                                ty: self.ctx.unit(),
                                kind: HirExprKind::Let {
                                    name: name.clone(),
                                    mutable: b_mut,
                                    value: Box::new(args[0].expr.clone()),
                                },
                                span: func.expr.span,
                            },
                            assign: None,
                        });
                    }
                    AssignKind::Set => {
                        if !b_defined {
                            self.diagnostics.push(Diagnostic::error(
                                "cannot set undefined variable",
                                func.expr.span,
                            ));
                        }
                        if !b_mut {
                            self.diagnostics
                                .push(Diagnostic::error("variable is not mutable", func.expr.span));
                        }
                        return Some(StackEntry {
                            ty: self.ctx.unit(),
                            expr: HirExpr {
                                ty: self.ctx.unit(),
                                kind: HirExprKind::Set {
                                    name: name.clone(),
                                    value: Box::new(args[0].expr.clone()),
                                },
                                span: func.expr.span,
                            },
                            assign: None,
                        });
                    }
                }
            } else {
                self.diagnostics.push(Diagnostic::error(
                    "assignment target not found",
                    func.expr.span,
                ));
                return None;
            }
        }

        // Special-cased symbols (if / while)
        match &func.expr.kind {
            HirExprKind::Var(name) if name == "if" => {
                if args.len() != 3 {
                    self.diagnostics.push(Diagnostic::error(
                        "if expects three arguments",
                        func.expr.span,
                    ));
                    return None;
                }
                if self.ctx.unify(args[0].ty, self.ctx.bool()).is_err() {
                    self.diagnostics.push(Diagnostic::error(
                        "if condition must be bool",
                        args[0].expr.span,
                    ));
                }
                let t = self.ctx.unify(args[1].ty, args[2].ty).unwrap_or(args[1].ty);
                return Some(StackEntry {
                    ty: t,
                    expr: HirExpr {
                        ty: t,
                        kind: HirExprKind::If {
                            cond: Box::new(args[0].expr.clone()),
                            then_branch: Box::new(args[1].expr.clone()),
                            else_branch: Box::new(args[2].expr.clone()),
                        },
                        span: func.expr.span,
                    },
                    assign: None,
                });
            }
            HirExprKind::Var(name) if name == "while" => {
                if args.len() != 2 {
                    self.diagnostics.push(Diagnostic::error(
                        "while expects two arguments",
                        func.expr.span,
                    ));
                    return None;
                }
                if self.ctx.unify(args[0].ty, self.ctx.bool()).is_err() {
                    self.diagnostics.push(Diagnostic::error(
                        "while condition must be bool",
                        args[0].expr.span,
                    ));
                }
                if self.ctx.unify(args[1].ty, self.ctx.unit()).is_err() {
                    self.diagnostics.push(Diagnostic::error(
                        "while body must be unit",
                        args[1].expr.span,
                    ));
                }
                return Some(StackEntry {
                    ty: self.ctx.unit(),
                    expr: HirExpr {
                        ty: self.ctx.unit(),
                        kind: HirExprKind::While {
                            cond: Box::new(args[0].expr.clone()),
                            body: Box::new(args[1].expr.clone()),
                        },
                        span: func.expr.span,
                    },
                    assign: None,
                });
            }
            HirExprKind::Var(name) if name == "let" || name == "set" => {
                // handled elsewhere
            }
            _ => {}
        }

        // General call or let/set
        if let HirExprKind::Var(name) = &func.expr.kind {
            if let Some(binding) = self.env.lookup(name) {
                match &binding.kind {
                    BindingKind::Var => {
                        self.diagnostics.push(Diagnostic::error(
                            "variable is not callable",
                            func.expr.span,
                        ));
                        return None;
                    }
                    BindingKind::Func { builtin, .. } => {
                        // Enum/struct constructors
                        if let Some((enm, var)) = parse_variant_name(name) {
                            if let Some(info) = self.enums.get(enm) {
                                if let Some(vinfo) =
                                    info.variants.iter().find(|v| v.name == var)
                                {
                                    // arity check
                                    if vinfo.payload.is_some() && args.len() != 1 {
                                        self.diagnostics.push(Diagnostic::error(
                                            "constructor expects one argument",
                                            func.expr.span,
                                        ));
                                        return None;
                                    }
                                    if vinfo.payload.is_none() && !args.is_empty() {
                                        self.diagnostics.push(Diagnostic::error(
                                            "constructor takes no arguments",
                                            func.expr.span,
                                        ));
                                        return None;
                                    }
                                    let payload_expr = if let Some(pty) = vinfo.payload {
                                        if let Some(a0) = args.first() {
                                            if let Err(_) = self.ctx.unify(a0.ty, pty) {
                                                self.diagnostics.push(Diagnostic::error(
                                                    "constructor payload type mismatch",
                                                    func.expr.span,
                                                ));
                                            }
                                            Some(Box::new(a0.expr.clone()))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };
                                    return Some(StackEntry {
                                        ty: info.ty,
                                        expr: HirExpr {
                                            ty: info.ty,
                                            kind: HirExprKind::EnumConstruct {
                                                name: enm.to_string(),
                                                variant: var.to_string(),
                                                payload: payload_expr,
                                            },
                                            span: func.expr.span,
                                        },
                                        assign: None,
                                    });
                                }
                            }
                        }
                        if self.structs.contains_key(name) {
                            let s = self.structs.get(name).unwrap();
                            if args.len() != s.fields.len() {
                                self.diagnostics.push(Diagnostic::error(
                                    "struct constructor arity mismatch",
                                    func.expr.span,
                                ));
                                return None;
                            }
                            for (arg, fty) in args.iter().zip(s.fields.iter()) {
                                if let Err(_) = self.ctx.unify(arg.ty, *fty) {
                                    self.diagnostics.push(Diagnostic::error(
                                        "struct field type mismatch",
                                        arg.expr.span,
                                    ));
                                }
                            }
                            return Some(StackEntry {
                                ty: s.ty,
                                expr: HirExpr {
                                    ty: s.ty,
                                    kind: HirExprKind::StructConstruct {
                                        name: name.clone(),
                                        fields: args.into_iter().map(|a| a.expr).collect(),
                                    },
                                    span: func.expr.span,
                                },
                                assign: None,
                            });
                        }
                        // General call (builtin or user)
                        for (arg, param_ty) in args.iter().zip(params.iter()) {
                            if let Err(_) = self.ctx.unify(arg.ty, *param_ty) {
                                self.diagnostics.push(Diagnostic::error(
                                    "argument type mismatch",
                                    arg.expr.span,
                                ));
                            }
                        }
                        let callee = if builtin.is_some() {
                            FuncRef::Builtin(name.clone())
                        } else {
                            FuncRef::User(name.clone())
                        };
                        return Some(StackEntry {
                            ty: result,
                            expr: HirExpr {
                                ty: result,
                                kind: HirExprKind::Call {
                                    callee,
                                    args: args.into_iter().map(|a| a.expr).collect(),
                                },
                                span: func.expr.span,
                            },
                            assign: None,
                        });
                    }
                }
            }
        }

        // Fallback: function value call
        Some(StackEntry {
            ty: result,
            expr: HirExpr {
                ty: result,
                kind: HirExprKind::Call {
                    callee: FuncRef::User(String::from("_unknown")),
                    args: args.into_iter().map(|a| a.expr).collect(),
                },
                span: func.expr.span,
            },
            assign: None,
        })
    }
}

// ---------------------------------------------------------------------
// Environment
// ---------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Binding {
    name: String,
    ty: TypeId,
    mutable: bool,
    defined: bool,
    kind: BindingKind,
}

#[derive(Debug, Clone)]
enum BindingKind {
    Var,
    Func {
        effect: Effect,
        arity: usize,
        builtin: Option<BuiltinKind>,
    },
}

#[derive(Debug)]
struct Env {
    scopes: Vec<Vec<Binding>>,
}

impl Env {
    fn new() -> Self {
        Self {
            scopes: vec![Vec::new()],
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert_global(&mut self, binding: Binding) {
        if let Some(scope) = self.scopes.first_mut() {
            scope.push(binding);
        }
    }

    fn insert_local(&mut self, binding: Binding) -> Result<(), ()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.iter().any(|b| b.name == binding.name) {
                return Err(());
            }
            scope.push(binding);
        }
        Ok(())
    }

    fn lookup(&self, name: &str) -> Option<&Binding> {
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope.iter().rev().find(|b| b.name == name) {
                return Some(b);
            }
        }
        None
    }

    fn lookup_mut(&mut self, name: &str) -> Option<&mut Binding> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(pos) = scope.iter().rposition(|b| b.name == name) {
                return scope.get_mut(pos);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------

type LabelEnv = BTreeMap<String, TypeId>;

#[derive(Debug)]
struct StringTable {
    map: BTreeMap<String, u32>,
    items: Vec<String>,
}

impl StringTable {
    fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            items: Vec::new(),
        }
    }

    fn intern(&mut self, s: String) -> u32 {
        if let Some(id) = self.map.get(&s) {
            *id
        } else {
            let id = self.items.len() as u32;
            self.items.push(s.clone());
            self.map.insert(s, id);
            id
        }
    }

    fn into_vec(self) -> Vec<String> {
        self.items
    }
}

fn type_from_expr(ctx: &mut TypeCtx, labels: &mut LabelEnv, t: &TypeExpr) -> TypeId {
    match t {
        TypeExpr::Unit => ctx.unit(),
        TypeExpr::I32 => ctx.i32(),
        TypeExpr::F32 => ctx.f32(),
        TypeExpr::Bool => ctx.bool(),
        TypeExpr::Str => ctx.str(),
        TypeExpr::Never => ctx.never(),
        TypeExpr::Named(name) => {
            if let Some(id) = ctx.lookup_named(name) {
                id
            } else {
                ctx.register_named(name.clone(), TypeKind::Named(name.clone()))
            }
        }
        TypeExpr::Label(label) => {
            if let Some(name) = label {
                if let Some(existing) = labels.get(name) {
                    *existing
                } else {
                    let id = ctx.fresh_var(Some(name.clone()));
                    labels.insert(name.clone(), id);
                    id
                }
            } else {
                ctx.fresh_var(None)
            }
        }
        TypeExpr::Function {
            params,
            result,
            effect,
        } => {
            let mut p = Vec::new();
            for ty in params {
                p.push(type_from_expr(ctx, labels, ty));
            }
            let r = type_from_expr(ctx, labels, result);
            ctx.function(p, r, *effect)
        }
    }
}

fn func_arity(ctx: &TypeCtx, ty: TypeId) -> usize {
    match ctx.get(ty) {
        TypeKind::Function { params, .. } => params.len(),
        _ => 0,
    }
}

fn parse_variant_name(name: &str) -> Option<(&str, &str)> {
    let mut parts = name.splitn(2, "::");
    let a = parts.next()?;
    let b = parts.next()?;
    Some((a, b))
}

fn target_allows(target: &str, active: CompileTarget) -> bool {
    match target {
        "wasm" => true,
        "wasi" => matches!(active, CompileTarget::Wasi),
        _ => false,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AssignKind {
    Let,
    Set,
}

#[derive(Debug, Clone)]
struct StackEntry {
    ty: TypeId,
    expr: HirExpr,
    assign: Option<AssignKind>,
}
