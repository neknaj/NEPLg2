use alloc::vec::Vec;

use crate::types::{TypeId, TypeKind};

use super::signature::type_contains_unbound_var;
use super::traits::{type_param_has_trait_application_bound, TraitBound, TraitId, TraitInfo};
use super::type_argument_inference::{
    resolve_type_arguments_from_constraints, TypeArgumentConstraint, TypeArgumentResolution,
};
use super::type_expectation::TypeExpectation;
use super::{BlockChecker, StackEntry};

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

#[derive(Clone, Copy)]
enum TypeParamInferenceSource {
    Argument,
    ExpectedResult,
}

#[derive(Clone, Copy)]
struct TypeParamInferenceConstraint {
    source: TypeParamInferenceSource,
    original: TypeId,
    actual: TypeId,
}

impl TypeParamInferenceConstraint {
    fn argument(original: TypeId, actual: TypeId) -> Self {
        Self {
            source: TypeParamInferenceSource::Argument,
            original,
            actual,
        }
    }

    fn expected_result(original: TypeId, actual: TypeId) -> Self {
        Self {
            source: TypeParamInferenceSource::ExpectedResult,
            original,
            actual,
        }
    }

    fn type_argument_constraint(self) -> TypeArgumentConstraint {
        match self.source {
            TypeParamInferenceSource::Argument | TypeParamInferenceSource::ExpectedResult => {
                TypeArgumentConstraint::new(self.original, self.actual)
            }
        }
    }
}

impl<'a> BlockChecker<'a> {
    pub(super) fn is_concrete_type(&self, ty: TypeId) -> bool {
        !type_contains_unbound_var(self.ctx, ty)
    }

    pub(super) fn type_param_has_trait_application_bound(
        &self,
        ty: TypeId,
        trait_id: &TraitId,
        trait_args: &[TypeId],
    ) -> bool {
        type_param_has_trait_application_bound(
            self.ctx,
            &self.type_param_bounds,
            ty,
            trait_id,
            trait_args,
        )
    }

    pub(super) fn trait_bound_satisfied(&self, bound: &TraitBound, ty: TypeId) -> bool {
        if !self.is_concrete_type(ty) {
            return self.type_param_has_trait_application_bound(
                ty,
                &bound.application.trait_id,
                &bound.application.args,
            );
        }
        if crate::log::is_verbose() {
            trait_check_log!(
                "trait_bound_satisfied: bound={} trait_self_ty={:?} ty={} ({:?})",
                bound.display_name(self.ctx),
                bound.trait_self_ty,
                self.ctx.type_to_string(ty),
                self.ctx.resolve_id(ty),
            );
            for imp in self.impls.iter().filter(|imp| {
                imp.matches_trait_application(
                    self.ctx,
                    &bound.application.trait_id,
                    &bound.application.args,
                )
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
            imp.matches_trait_application(
                self.ctx,
                &bound.application.trait_id,
                &bound.application.args,
            ) && self.ctx.type_pattern_matches(imp.target_ty, ty)
        })
    }

    pub(super) fn infer_unique_type_param_for_trait_ref(
        &self,
        trait_id: &TraitId,
        trait_args: &[TypeId],
    ) -> Option<TypeId> {
        let mut matched: Option<TypeId> = None;
        for (tp, bounds) in self.type_param_bounds.iter() {
            if !bounds
                .iter()
                .any(|b| b.application.matches_parts(self.ctx, trait_id, trait_args))
            {
                continue;
            }
            let resolved = self.ctx.resolve_id(tp.type_id());
            match matched {
                None => matched = Some(resolved),
                Some(prev) if self.ctx.same_type(prev, resolved) => {}
                Some(_) => return None,
            }
        }
        matched
    }

    pub(super) fn resolve_trait_application_args(
        &self,
        trait_info: &TraitInfo,
        sig: TypeId,
        args: &[StackEntry],
        expected_ret: Option<TypeExpectation>,
    ) -> TypeArgumentResolution {
        if trait_info.type_params.is_empty() {
            return TypeArgumentResolution {
                resolved_args: Vec::new(),
                conflicts: Vec::new(),
            };
        }
        let resolved_sig = self.ctx.resolve_id(sig);
        let TypeKind::Function { params, result, .. } = self.ctx.get(resolved_sig) else {
            return TypeArgumentResolution {
                resolved_args: Vec::new(),
                conflicts: Vec::new(),
            };
        };
        let mut constraints = Vec::new();
        for (param_ty, arg) in params.iter().zip(args.iter()) {
            constraints.push(TypeParamInferenceConstraint::argument(*param_ty, arg.ty));
        }
        if let Some(expectation) = expected_ret {
            constraints.push(TypeParamInferenceConstraint::expected_result(
                result,
                expectation.target(),
            ));
        }
        let type_argument_constraints = constraints
            .iter()
            .copied()
            .map(TypeParamInferenceConstraint::type_argument_constraint)
            .collect::<Vec<_>>();
        resolve_type_arguments_from_constraints(
            self.ctx,
            &trait_info.type_params,
            trait_info.type_params.clone(),
            &type_argument_constraints,
        )
    }
}
