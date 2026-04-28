use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{TypeId, TypeKind};

use super::traits::{
    format_trait_ref_name, infer_type_param_from_instantiated_pair, merge_inferred_instantiation,
    parse_trait_ref_name, trait_application_matches, TraitBoundRef, TraitInfo,
};
use super::{type_contains_unbound_var, BlockChecker, StackEntry};

macro_rules! trait_check_log {
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
    pub(super) fn is_concrete_type(&self, ty: TypeId) -> bool {
        !type_contains_unbound_var(self.ctx, ty)
    }

    pub(super) fn type_param_has_bound_ref(
        &self,
        ty: TypeId,
        trait_base_name: &str,
        trait_args: &[TypeId],
    ) -> bool {
        let matches_bound = |b: &TraitBoundRef| {
            trait_application_matches(
                self.ctx,
                trait_base_name,
                trait_args,
                &b.trait_base_name,
                &b.trait_args,
            )
        };
        let resolved = self.ctx.resolve_id(ty);
        if let Some(bounds) = self.type_param_bounds.get(&resolved) {
            return bounds.iter().any(matches_bound);
        }

        // 型変数が他の型変数へ束縛された場合、resolve 後の TypeId が
        // 直接 type_param_bounds に存在しないことがあるため、正規化後 ID でも照合する。
        if self.type_param_bounds.iter().any(|(tp, bounds)| {
            self.ctx.resolve_id(*tp) == resolved && bounds.iter().any(matches_bound)
        }) {
            return true;
        }

        // `.T` の明示型引数が同一スコープの別 TypeId として現れる経路があるため、
        // 型変数ラベルが一致する場合も同じ境界として扱う。
        let label = match self.ctx.get(resolved) {
            TypeKind::Var(v) => v.label.clone(),
            _ => None,
        };
        let Some(label) = label else {
            return false;
        };
        self.type_param_bounds.iter().any(|(tp, bounds)| {
            let same_label = match self.ctx.get(self.ctx.resolve_id(*tp)) {
                TypeKind::Var(v) => v.label.as_deref() == Some(label.as_str()),
                _ => false,
            };
            same_label && bounds.iter().any(matches_bound)
        })
    }

    pub(super) fn trait_bound_satisfied_by_ref(&self, bound: &TraitBoundRef, ty: TypeId) -> bool {
        if !self.is_concrete_type(ty) {
            return self.type_param_has_bound_ref(ty, &bound.trait_base_name, &bound.trait_args);
        }
        if crate::log::is_verbose() {
            trait_check_log!(
                "trait_bound_satisfied_by_ref: bound={} trait_self_ty={:?} ty={} ({:?})",
                bound.name,
                bound.trait_self_ty,
                self.ctx.type_to_string(ty),
                self.ctx.resolve_id(ty),
            );
            for imp in self.impls.iter().filter(|imp| {
                imp.trait_base_name
                    .as_deref()
                    .map(|base| {
                        trait_application_matches(
                            self.ctx,
                            &bound.trait_base_name,
                            &bound.trait_args,
                            base,
                            &imp.trait_args,
                        )
                    })
                    .unwrap_or(false)
            }) {
                trait_check_log!(
                    "  impl candidate target={} ({:?}) same_type={}",
                    self.ctx.type_to_string(imp.target_ty),
                    self.ctx.resolve_id(imp.target_ty),
                    self.ctx.same_type(imp.target_ty, ty),
                );
            }
        }
        self.impls.iter().any(|imp| {
            imp.trait_base_name
                .as_deref()
                .map(|base| {
                    trait_application_matches(
                        self.ctx,
                        &bound.trait_base_name,
                        &bound.trait_args,
                        base,
                        &imp.trait_args,
                    )
                })
                .unwrap_or(false)
                && self.ctx.type_pattern_matches(imp.target_ty, ty)
        })
    }

    pub(super) fn infer_unique_type_param_for_trait_ref(
        &self,
        trait_base_name: &str,
        trait_args: &[TypeId],
    ) -> Option<TypeId> {
        let mut matched: Option<TypeId> = None;
        for (tp, bounds) in &self.type_param_bounds {
            if !bounds.iter().any(|b| {
                trait_application_matches(
                    self.ctx,
                    trait_base_name,
                    trait_args,
                    &b.trait_base_name,
                    &b.trait_args,
                )
            }) {
                continue;
            }
            let resolved = self.ctx.resolve_id(*tp);
            match matched {
                None => matched = Some(resolved),
                Some(prev) if self.ctx.same_type(prev, resolved) => {}
                Some(_) => return None,
            }
        }
        matched
    }

    pub(super) fn infer_unique_type_param_for_trait(&self, trait_name: &str) -> Option<TypeId> {
        if let Some((base, args)) = parse_trait_ref_name(trait_name, self.ctx) {
            return self.infer_unique_type_param_for_trait_ref(&base, &args);
        }
        self.infer_unique_type_param_for_trait_ref(trait_name, &[])
    }

    pub(super) fn infer_trait_application_name(
        &self,
        trait_name: &str,
        trait_info: &TraitInfo,
        sig: TypeId,
        args: &[StackEntry],
        expected_ret: Option<TypeId>,
    ) -> String {
        let inferred = self.infer_trait_application_args(trait_info, sig, args, expected_ret);
        format_trait_ref_name(trait_name, &inferred, self.ctx)
    }

    pub(super) fn infer_trait_application_args(
        &self,
        trait_info: &TraitInfo,
        sig: TypeId,
        args: &[StackEntry],
        expected_ret: Option<TypeId>,
    ) -> Vec<TypeId> {
        if trait_info.type_params.is_empty() {
            return Vec::new();
        }
        let resolved_sig = self.ctx.resolve_id(sig);
        let TypeKind::Function { params, result, .. } = self.ctx.get(resolved_sig) else {
            return Vec::new();
        };
        let mut inferred = Vec::new();
        for tp in &trait_info.type_params {
            let label = match self.ctx.get(self.ctx.resolve_id(*tp)) {
                TypeKind::Var(v) => v.label.clone(),
                _ => None,
            };
            let mut found = None;
            for (param_ty, arg) in params.iter().zip(args.iter()) {
                found = merge_inferred_instantiation(
                    self.ctx,
                    found,
                    infer_type_param_from_instantiated_pair(
                        self.ctx,
                        *param_ty,
                        arg.ty,
                        *tp,
                        label.as_deref(),
                    ),
                );
            }
            if let Some(expected) = expected_ret {
                found = merge_inferred_instantiation(
                    self.ctx,
                    found,
                    infer_type_param_from_instantiated_pair(
                        self.ctx,
                        result,
                        expected,
                        *tp,
                        label.as_deref(),
                    ),
                );
            }
            inferred.push(found.unwrap_or(*tp));
        }
        inferred
    }
}
