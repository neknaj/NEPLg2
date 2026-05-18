use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::ast::{MatchArm, MatchExpr, MatchPattern, Visibility};
use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::hir::{HirExpr, HirExprKind, HirMatchArm, HirMatchBindMode, HirMatchPattern};
use crate::span::Span;
use crate::types::{EnumVariantInfo, TypeId, TypeKind};

use super::binding_rules::emit_shadow_warning;
use super::diagnostics::type_error;
use super::env::{Binding, BindingKind};
use super::syntax_helpers::{parse_i32_literal, variant_member_tail};
use super::{BlockChecker, ScalarMatchKind};

impl<'a> BlockChecker<'a> {
    pub(super) fn check_match_expr(&mut self, m: &MatchExpr) -> Option<(HirExpr, TypeId)> {
        // Infer the expected scrutinee type from the arm variant names.
        // e.g. `Result::Ok`, `Result::Err` → `Result<fresh_A, fresh_B>`
        // This allows disambiguation of overloaded scrutinee calls when the enum base
        // type is enough to select the right overload.
        let expected_scrut_ty = self.infer_expected_type_from_match_arms(&m.arms);
        let checkpoint = self.ctx.checkpoint();
        let diagnostics_len = self.diagnostics.len();
        let pending_trait_bound_checks_len = self.pending_trait_bound_checks.len();
        let mut tmp_stack = Vec::new();
        if let Some((scrut_expr, _)) = self.check_prefix(&m.scrutinee, 0, &mut tmp_stack, None) {
            if let Some(result) = self.check_match_expr_from_scrutinee(m, scrut_expr.clone()) {
                return Some(result);
            }
        }

        if expected_scrut_ty.is_some() {
            self.diagnostics.truncate(diagnostics_len);
            self.pending_trait_bound_checks
                .truncate(pending_trait_bound_checks_len);
            self.ctx.rollback(checkpoint);
            let mut tmp_stack = Vec::new();
            if let Some((scrut_expr, _)) =
                self.check_prefix(&m.scrutinee, 0, &mut tmp_stack, expected_scrut_ty)
            {
                return self.check_match_expr_from_scrutinee(m, scrut_expr);
            }
        }
        None
    }

    fn check_match_expr_from_scrutinee(
        &mut self,
        m: &MatchExpr,
        scrut_expr: HirExpr,
    ) -> Option<(HirExpr, TypeId)> {
        let scrut_ty = scrut_expr.ty;
        let resolved_ty = self.ctx.resolve(scrut_ty);
        if let Some((variants, bind_mode)) = self.match_enum_subject_for_type(resolved_ty) {
            return self.check_enum_match_expr(m, scrut_expr, variants, bind_mode);
        }
        let scalar = match self.ctx.get(resolved_ty) {
            TypeKind::I32 => Some(ScalarMatchKind::I32),
            TypeKind::U8 => Some(ScalarMatchKind::U8),
            TypeKind::Bool => Some(ScalarMatchKind::Bool),
            TypeKind::Char => Some(ScalarMatchKind::Char),
            _ => None,
        };
        if let Some(kind) = scalar {
            return self.check_scalar_match_expr(m, scrut_expr, kind);
        }
        self.diagnostics.push(type_error(
            TypeDiagnosticCode::MatchScrutineeNotEnum,
            "match scrutinee must be an enum, bool, char, i32, or u8",
            m.span,
        ));
        None
    }

    fn match_enum_subject_for_type(
        &mut self,
        resolved_ty: TypeId,
    ) -> Option<(Vec<EnumVariantInfo>, HirMatchBindMode)> {
        let reference_target = match self.ctx.get(resolved_ty) {
            TypeKind::Reference(target, is_mut) => Some((target, is_mut)),
            _ => None,
        };
        if let Some((target, is_mut)) = reference_target {
            let target = self.ctx.resolve(target);
            return self
                .match_enum_variants_for_type(target)
                .map(|variants| (variants, HirMatchBindMode::Borrowed { is_mut }));
        }
        self.match_enum_variants_for_type(resolved_ty)
            .map(|variants| (variants, HirMatchBindMode::Owned))
    }

    pub(super) fn match_enum_variants_for_type(
        &mut self,
        resolved_ty: TypeId,
    ) -> Option<Vec<EnumVariantInfo>> {
        match self.ctx.get(resolved_ty) {
            TypeKind::Enum { variants, .. } => Some(variants.clone()),
            TypeKind::Apply { base, args } => {
                let base_ty = self.ctx.resolve(base);
                match self.ctx.get(base_ty) {
                    TypeKind::Enum {
                        type_params,
                        variants,
                        ..
                    } => {
                        if type_params.len() != args.len() {
                            return None;
                        }
                        let type_params = type_params.clone();
                        let variants = variants.clone();
                        let args = args.clone();
                        let mut mapping = alloc::collections::BTreeMap::new();
                        for (tp, arg) in type_params.iter().zip(args.iter()) {
                            mapping.insert(*tp, *arg);
                        }
                        let mut new_vars = Vec::new();
                        for v in variants {
                            new_vars.push(EnumVariantInfo {
                                name: v.name.clone(),
                                payload: v.payload.map(|p| self.ctx.substitute(p, &mapping)),
                            });
                        }
                        Some(new_vars)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub(super) fn check_enum_match_expr(
        &mut self,
        m: &MatchExpr,
        scrut_expr: HirExpr,
        variants: Vec<EnumVariantInfo>,
        subject_bind_mode: HirMatchBindMode,
    ) -> Option<(HirExpr, TypeId)> {
        let mut seen = alloc::collections::BTreeSet::new();
        let mut saw_wildcard = false;
        let mut arms_hir = Vec::new();
        let mut result_ty: Option<TypeId> = None;
        for (idx, arm) in m.arms.iter().enumerate() {
            let (variant, bind) = match &arm.pattern {
                MatchPattern::Variant { name, bind } => (name, bind),
                MatchPattern::Wildcard { span } => {
                    if idx + 1 != m.arms.len() {
                        self.diagnostics.push(type_error(
                            TypeDiagnosticCode::MatchWildcardNotLast,
                            "wildcard match arm must be last",
                            *span,
                        ));
                    }
                    if saw_wildcard {
                        self.diagnostics.push(type_error(
                            TypeDiagnosticCode::MatchDuplicateArm,
                            "duplicate match arm",
                            *span,
                        ));
                        continue;
                    }
                    saw_wildcard = true;
                    self.env.push_scope();
                    let (blk, val_ty) = self.check_block(&arm.body, 0, false, None)?;
                    self.env.pop_scope();
                    let body_ty = val_ty.unwrap_or(self.ctx.unit());
                    self.check_match_arm_result_type(&mut result_ty, body_ty, arm.span);
                    arms_hir.push(HirMatchArm {
                        pattern: HirMatchPattern::Wildcard,
                        bind_local: None,
                        bind_ty: None,
                        bind_mode: None,
                        body: HirExpr {
                            ty: body_ty,
                            kind: HirExprKind::Block(blk),
                            span: arm.span,
                        },
                    });
                    continue;
                }
                _ => {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchPatternUnsupported,
                        "unsupported match pattern for enum scrutinee",
                        arm.span,
                    ));
                    continue;
                }
            };
            let arm_var_name = variant_member_tail(&variant.name);
            if !seen.insert(arm_var_name.to_string()) {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::MatchDuplicateArm,
                    "duplicate match arm",
                    variant.span,
                ));
                continue;
            }
            let var_info = variants.iter().find(|v| v.name == arm_var_name);
            if var_info.is_none() {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::MatchVariantUnknown,
                    alloc::format!("unknown enum variant '{}' in match", variant.name),
                    variant.span,
                ));
                continue;
            }
            let var_info = var_info.unwrap();
            let bind_ty = bind.as_ref().and_then(|_| {
                var_info.payload.map(|payload| match subject_bind_mode {
                    HirMatchBindMode::Owned => payload,
                    HirMatchBindMode::Borrowed { is_mut } => self.ctx.reference(payload, is_mut),
                })
            });
            let bind_mode = bind
                .as_ref()
                .and_then(|_| var_info.payload.map(|_| subject_bind_mode));
            self.env.push_scope();
            if let Some(bind) = bind {
                if let Some(pty) = bind_ty {
                    emit_shadow_warning(
                        &mut self.diagnostics,
                        self.env,
                        &bind.name,
                        bind.span,
                        "match binding",
                    );
                    let _ = self.env.insert_local(Binding {
                        name: bind.name.clone(),
                        ty: pty,
                        visibility: Visibility::Private,
                        mutable: false,
                        no_shadow: false,
                        defined: true,
                        span: bind.span,
                        kind: BindingKind::Var,
                    });
                } else {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchPayloadBindingInvalid,
                        "variant has no payload to bind",
                        bind.span,
                    ));
                }
            }
            let (blk, val_ty) = self.check_block(&arm.body, 0, false, None)?;
            self.env.pop_scope();
            let body_ty = val_ty.unwrap_or(self.ctx.unit());
            self.check_match_arm_result_type(&mut result_ty, body_ty, arm.span);
            arms_hir.push(HirMatchArm {
                pattern: HirMatchPattern::Variant(variant.name.clone()),
                bind_local: bind.as_ref().map(|b| b.name.clone()),
                bind_ty,
                bind_mode,
                body: HirExpr {
                    ty: body_ty,
                    kind: HirExprKind::Block(blk),
                    span: arm.span,
                },
            });
        }
        if !saw_wildcard {
            for v in variants {
                if !seen.contains(&v.name) {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchNonExhaustive,
                        "non-exhaustive match",
                        m.span,
                    ));
                    break;
                }
            }
        }
        let rty = result_ty.unwrap_or(self.ctx.unit());
        Some((
            HirExpr {
                ty: rty,
                kind: HirExprKind::Match {
                    scrutinee: Box::new(scrut_expr),
                    arms: arms_hir,
                },
                span: m.span,
            },
            rty,
        ))
    }

    pub(super) fn check_scalar_match_expr(
        &mut self,
        m: &MatchExpr,
        scrut_expr: HirExpr,
        kind: ScalarMatchKind,
    ) -> Option<(HirExpr, TypeId)> {
        let mut seen_i32 = alloc::collections::BTreeSet::new();
        let mut seen_true = false;
        let mut seen_false = false;
        let mut saw_wildcard = false;
        let mut arms_hir = Vec::new();
        let mut result_ty: Option<TypeId> = None;

        for (idx, arm) in m.arms.iter().enumerate() {
            let hir_pattern = match self.scalar_match_pattern(kind, arm, idx + 1 == m.arms.len()) {
                Some(p) => p,
                None => continue,
            };
            match &hir_pattern {
                HirMatchPattern::IntLiteral(v) => {
                    if !seen_i32.insert(*v) {
                        self.diagnostics.push(type_error(
                            TypeDiagnosticCode::MatchDuplicateArm,
                            "duplicate match arm",
                            arm.span,
                        ));
                        continue;
                    }
                }
                HirMatchPattern::BoolLiteral(true) => {
                    if seen_true {
                        self.diagnostics.push(type_error(
                            TypeDiagnosticCode::MatchDuplicateArm,
                            "duplicate match arm",
                            arm.span,
                        ));
                        continue;
                    }
                    seen_true = true;
                }
                HirMatchPattern::BoolLiteral(false) => {
                    if seen_false {
                        self.diagnostics.push(type_error(
                            TypeDiagnosticCode::MatchDuplicateArm,
                            "duplicate match arm",
                            arm.span,
                        ));
                        continue;
                    }
                    seen_false = true;
                }
                HirMatchPattern::Wildcard => {
                    if saw_wildcard {
                        self.diagnostics.push(type_error(
                            TypeDiagnosticCode::MatchDuplicateArm,
                            "duplicate match arm",
                            arm.span,
                        ));
                        continue;
                    }
                    saw_wildcard = true;
                }
                HirMatchPattern::Variant(_) => {}
            }

            let (blk, val_ty) = self.check_block(&arm.body, 0, false, None)?;
            let body_ty = val_ty.unwrap_or(self.ctx.unit());
            self.check_match_arm_result_type(&mut result_ty, body_ty, arm.span);
            arms_hir.push(HirMatchArm {
                pattern: hir_pattern,
                bind_local: None,
                bind_ty: None,
                bind_mode: None,
                body: HirExpr {
                    ty: body_ty,
                    kind: HirExprKind::Block(blk),
                    span: arm.span,
                },
            });
        }

        let exhaustive = match kind {
            ScalarMatchKind::Bool => saw_wildcard || (seen_true && seen_false),
            ScalarMatchKind::I32 | ScalarMatchKind::U8 | ScalarMatchKind::Char => saw_wildcard,
        };
        if !exhaustive {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::MatchNonExhaustive,
                "non-exhaustive match",
                m.span,
            ));
        }
        let rty = result_ty.unwrap_or(self.ctx.unit());
        Some((
            HirExpr {
                ty: rty,
                kind: HirExprKind::Match {
                    scrutinee: Box::new(scrut_expr),
                    arms: arms_hir,
                },
                span: m.span,
            },
            rty,
        ))
    }

    pub(super) fn scalar_match_pattern(
        &mut self,
        kind: ScalarMatchKind,
        arm: &MatchArm,
        is_last: bool,
    ) -> Option<HirMatchPattern> {
        match &arm.pattern {
            MatchPattern::IntLiteral { text, span } => {
                if kind == ScalarMatchKind::Bool {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchPatternUnsupported,
                        "integer literal match arm cannot match bool",
                        *span,
                    ));
                    return None;
                }
                if kind == ScalarMatchKind::Char {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchPatternUnsupported,
                        "integer literal match arm cannot match char",
                        *span,
                    ));
                    return None;
                }
                let Some(value) = parse_i32_literal(text) else {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchPatternUnsupported,
                        "invalid integer literal in match arm",
                        *span,
                    ));
                    return None;
                };
                if kind == ScalarMatchKind::U8 && !(0..=255).contains(&value) {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchPatternUnsupported,
                        "u8 match literal must be in 0..=255",
                        *span,
                    ));
                    return None;
                }
                Some(HirMatchPattern::IntLiteral(value))
            }
            MatchPattern::BoolLiteral { value, span } => {
                if kind != ScalarMatchKind::Bool {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchPatternUnsupported,
                        "bool literal match arm cannot match this scrutinee type",
                        *span,
                    ));
                    return None;
                }
                Some(HirMatchPattern::BoolLiteral(*value))
            }
            MatchPattern::CharLiteral { value, span } => {
                if kind == ScalarMatchKind::Bool {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchPatternUnsupported,
                        "char literal match arm cannot match this scrutinee type",
                        *span,
                    ));
                    return None;
                }
                if *value > i32::MAX as u32 {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchPatternUnsupported,
                        "char literal is outside current i32-backed codegen range",
                        *span,
                    ));
                    return None;
                }
                if kind == ScalarMatchKind::U8 && *value > 255 {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchPatternUnsupported,
                        "char literal match arm does not fit in u8",
                        *span,
                    ));
                    return None;
                }
                Some(HirMatchPattern::IntLiteral(*value as i32))
            }
            MatchPattern::Wildcard { span } => {
                if !is_last {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchWildcardNotLast,
                        "wildcard match arm must be last",
                        *span,
                    ));
                }
                Some(HirMatchPattern::Wildcard)
            }
            MatchPattern::Variant { name, bind } => {
                if bind.is_some() {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchPayloadBindingInvalid,
                        "literal match arms cannot bind payloads",
                        name.span,
                    ));
                    return None;
                }
                if kind == ScalarMatchKind::Bool {
                    match name.name.as_str() {
                        "true" => Some(HirMatchPattern::BoolLiteral(true)),
                        "false" => Some(HirMatchPattern::BoolLiteral(false)),
                        _ => {
                            self.diagnostics.push(type_error(
                                TypeDiagnosticCode::MatchPatternUnsupported,
                                "bool match arms must be true, false, or _",
                                name.span,
                            ));
                            None
                        }
                    }
                } else if kind == ScalarMatchKind::Char {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchPatternUnsupported,
                        "char match arms must be char literals or _",
                        name.span,
                    ));
                    None
                } else {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::MatchPatternUnsupported,
                        "integer match arms must be integer literals or _",
                        name.span,
                    ));
                    None
                }
            }
        }
    }

    pub(super) fn check_match_arm_result_type(
        &mut self,
        result_ty: &mut Option<TypeId>,
        body_ty: TypeId,
        span: Span,
    ) {
        if let Some(t) = *result_ty {
            if let Err(_) = self.ctx.unify(t, body_ty) {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::MatchArmsMismatch,
                    alloc::format!(
                        "match arms have incompatible types: {} and {}",
                        self.ctx.type_to_string(t),
                        self.ctx.type_to_string(body_ty)
                    ),
                    span,
                ));
            }
        } else {
            *result_ty = Some(body_ty);
        }
    }
}
