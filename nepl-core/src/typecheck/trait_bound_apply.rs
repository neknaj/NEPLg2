use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::DiagnosticCode;
use crate::span::Span;
use crate::types::TypeId;

use super::traits::{format_trait_ref_name, infer_instantiated_type_arg, TraitBoundRef};
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
    pub(super) fn check_selected_function_trait_bounds(
        &mut self,
        callee_name: &str,
        binding_ty: TypeId,
        inst_ty: TypeId,
        type_param_bounds: &BTreeMap<TypeId, Vec<TraitBoundRef>>,
        type_arg_mapping: &BTreeMap<TypeId, TypeId>,
        span: Span,
    ) {
        if type_param_bounds.is_empty() {
            return;
        }
        for (tp, bounds) in type_param_bounds.iter() {
            let Some(raw_arg) = type_arg_mapping.get(tp) else {
                continue;
            };
            let resolved_arg = self.ctx.resolve_id(*raw_arg);
            for b in bounds {
                let substituted_trait_args = b
                    .trait_args
                    .iter()
                    .map(|arg| self.ctx.substitute(*arg, type_arg_mapping))
                    .collect::<Vec<_>>();
                let substituted_bound = TraitBoundRef {
                    name: format_trait_ref_name(
                        &b.trait_base_name,
                        &substituted_trait_args,
                        self.ctx,
                    ),
                    trait_base_name: b.trait_base_name.clone(),
                    trait_args: substituted_trait_args,
                    trait_self_ty: self.ctx.substitute(b.trait_self_ty, type_arg_mapping),
                };
                if crate::log::is_verbose() {
                    trait_bound_apply_log!(
                        "trait-bound debug: callee='{}' tp={} raw_arg={} resolved_arg={} bound={} current_bounds={}",
                        callee_name,
                        self.ctx.type_to_string(*tp),
                        self.ctx.type_to_string(*raw_arg),
                        self.ctx.type_to_string(resolved_arg),
                        substituted_bound.name,
                        self.type_param_bounds
                            .iter()
                            .map(|(bound_tp, bs)| {
                                format!(
                                    "{}:[{}]",
                                    self.ctx.type_to_string(*bound_tp),
                                    bs.iter()
                                        .map(|bb| bb.name.clone())
                                        .collect::<Vec<_>>()
                                        .join("|")
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
                if self.trait_bound_satisfied_by_ref(&substituted_bound, *raw_arg)
                    || self.trait_bound_satisfied_by_ref(&substituted_bound, resolved_arg)
                {
                    continue;
                }
                let inferred_arg = infer_instantiated_type_arg(self.ctx, binding_ty, inst_ty, *tp)
                    .unwrap_or(resolved_arg);
                if self.trait_bound_satisfied_by_ref(&substituted_bound, inferred_arg) {
                    continue;
                }
                if self.is_concrete_type(inferred_arg) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            format!(
                                "type does not satisfy trait bound '{}'",
                                substituted_bound.name
                            ),
                            span,
                        )
                        .with_code(DiagnosticCode::Type(
                            crate::diagnostic_codes::TypeDiagnosticCode::TraitBoundUnsatisfied,
                        )),
                    );
                } else {
                    self.pending_trait_bound_checks
                        .push((substituted_bound, inferred_arg, span));
                }
            }
        }
    }
}
