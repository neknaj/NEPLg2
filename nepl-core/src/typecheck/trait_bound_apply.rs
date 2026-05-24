use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec::Vec;

use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::span::Span;
use crate::types::TypeId;

use super::diagnostics::type_error;
use super::traits::{
    infer_instantiated_type_arg, BoundEnv, PendingTraitCheck, TraitApplication, TraitBound,
};
use super::BlockChecker;

macro_rules! trait_bound_apply_log {
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
    fn selected_function_trait_bound_resolution(
        &mut self,
        binding_ty: TypeId,
        inst_ty: TypeId,
        type_arg_mapping: &BTreeMap<TypeId, TypeId>,
        bound: &TraitBound,
        type_param: TypeId,
        raw_arg: TypeId,
    ) -> SelectedFunctionTraitBoundResolution {
        let resolved_arg = self.ctx.resolve_id(raw_arg);
        if self.trait_bound_satisfied(bound, raw_arg)
            || self.trait_bound_satisfied(bound, resolved_arg)
        {
            return SelectedFunctionTraitBoundResolution::Satisfied;
        }
        let inferred_arg = infer_instantiated_type_arg(self.ctx, binding_ty, inst_ty, type_param)
            .unwrap_or(resolved_arg);
        if self.trait_bound_satisfied(bound, inferred_arg) {
            return SelectedFunctionTraitBoundResolution::Satisfied;
        }
        if self.trait_bound_satisfied_by_impl_pattern(bound, inferred_arg) {
            return SelectedFunctionTraitBoundResolution::Satisfied;
        }
        if self.is_concrete_type(inferred_arg) {
            SelectedFunctionTraitBoundResolution::Unsatisfied
        } else {
            let target_ty = self.ctx.substitute(inferred_arg, type_arg_mapping);
            SelectedFunctionTraitBoundResolution::Pending { target_ty }
        }
    }

    pub(super) fn selected_function_trait_bounds_may_satisfy(
        &mut self,
        binding_ty: TypeId,
        inst_ty: TypeId,
        type_param_bounds: &BoundEnv,
        type_arg_mapping: &BTreeMap<TypeId, TypeId>,
    ) -> bool {
        if type_param_bounds.is_empty() {
            return true;
        }
        for (tp, bounds) in type_param_bounds.iter() {
            let tp_type_id = tp.type_id();
            let Some(raw_arg) = type_arg_mapping.get(&tp_type_id).copied() else {
                continue;
            };
            for b in bounds {
                let substituted_trait_args = b
                    .application
                    .args
                    .iter()
                    .map(|arg| self.ctx.substitute(*arg, type_arg_mapping))
                    .collect::<Vec<_>>();
                let substituted_bound = TraitBound {
                    application: TraitApplication {
                        trait_id: b.application.trait_id.clone(),
                        args: substituted_trait_args,
                    },
                    trait_self_ty: self.ctx.substitute(b.trait_self_ty, type_arg_mapping),
                };
                if !matches!(
                    self.selected_function_trait_bound_resolution(
                        binding_ty,
                        inst_ty,
                        type_arg_mapping,
                        &substituted_bound,
                        tp_type_id,
                        raw_arg,
                    ),
                    SelectedFunctionTraitBoundResolution::Satisfied
                ) {
                    return false;
                }
            }
        }
        true
    }

    pub(super) fn check_selected_function_trait_bounds(
        &mut self,
        callee_name: &str,
        binding_ty: TypeId,
        inst_ty: TypeId,
        type_param_bounds: &BoundEnv,
        type_arg_mapping: &BTreeMap<TypeId, TypeId>,
        span: Span,
    ) {
        if type_param_bounds.is_empty() {
            return;
        }
        for (tp, bounds) in type_param_bounds.iter() {
            let tp_type_id = tp.type_id();
            let Some(raw_arg) = type_arg_mapping.get(&tp_type_id) else {
                continue;
            };
            let resolved_arg = self.ctx.resolve_id(*raw_arg);
            for b in bounds {
                let substituted_trait_args = b
                    .application
                    .args
                    .iter()
                    .map(|arg| self.ctx.substitute(*arg, type_arg_mapping))
                    .collect::<Vec<_>>();
                let substituted_bound = TraitBound {
                    application: TraitApplication {
                        trait_id: b.application.trait_id.clone(),
                        args: substituted_trait_args,
                    },
                    trait_self_ty: self.ctx.substitute(b.trait_self_ty, type_arg_mapping),
                };
                if crate::log::is_verbose() {
                    trait_bound_apply_log!(
                        "trait-bound debug: callee='{}' tp={} raw_arg={} resolved_arg={} bound={} current_bounds={}",
                        callee_name,
                        self.ctx.type_to_string(tp_type_id),
                        self.ctx.type_to_string(*raw_arg),
                        self.ctx.type_to_string(resolved_arg),
                        substituted_bound.display_name(self.ctx),
                        self.type_param_bounds
                            .iter()
                            .map(|(bound_tp, bs)| {
                                format!(
                                    "{}:[{}]",
                                    self.ctx.type_to_string(bound_tp.type_id()),
                                    bs.iter()
                                        .map(|bb| bb.display_name(self.ctx))
                                        .collect::<Vec<_>>()
                                        .join("|")
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
                match self.selected_function_trait_bound_resolution(
                    binding_ty,
                    inst_ty,
                    type_arg_mapping,
                    &substituted_bound,
                    tp_type_id,
                    *raw_arg,
                ) {
                    SelectedFunctionTraitBoundResolution::Satisfied => {}
                    SelectedFunctionTraitBoundResolution::Unsatisfied => {
                        self.diagnostics.push(type_error(
                            TypeDiagnosticCode::TraitBoundUnsatisfied,
                            format!(
                                "type does not satisfy trait bound '{}'",
                                substituted_bound.display_name(self.ctx)
                            ),
                            span,
                        ));
                    }
                    SelectedFunctionTraitBoundResolution::Pending { target_ty } => {
                        self.pending_trait_bound_checks.push(PendingTraitCheck {
                            bound: substituted_bound,
                            target_ty,
                            span,
                        });
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectedFunctionTraitBoundResolution {
    Satisfied,
    Pending { target_ty: TypeId },
    Unsatisfied,
}
