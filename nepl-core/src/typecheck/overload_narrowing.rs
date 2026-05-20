use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::types::{TypeCtx, TypeKind};

use super::overload_candidate::{
    OverloadAmbiguityReason, OverloadCandidate, OverloadCandidateNarrowingStage,
};
use super::signature::{function_signature_string, type_contains_unbound_var};

macro_rules! overload_narrowing_log {
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

pub(super) fn narrow_overload_candidates<'b>(
    ctx: &TypeCtx,
    current_effect: Effect,
    name: &str,
    mut candidates: Vec<OverloadCandidate<'b>>,
) -> Result<OverloadCandidate<'b>, OverloadAmbiguityReason> {
    let mut last_narrowing_stage = OverloadCandidateNarrowingStage::InitialCandidates;
    if candidates.len() > 1 && matches!(current_effect, Effect::Pure) {
        let pure_only: Vec<OverloadCandidate<'_>> = candidates
            .iter()
            .filter(|c| {
                matches!(
                    ctx.get(c.binding.ty),
                    TypeKind::Function {
                        effect: Effect::Pure,
                        ..
                    }
                )
            })
            .cloned()
            .collect();
        if !pure_only.is_empty() {
            candidates = pure_only;
        }
        last_narrowing_stage = OverloadCandidateNarrowingStage::PreferPureFunction;
    }

    if candidates.len() > 1 {
        let mut sig_seen: BTreeSet<alloc::string::String> = BTreeSet::new();
        let mut dedup: Vec<OverloadCandidate<'_>> = Vec::new();
        for c in candidates {
            let sig = function_signature_string(ctx, c.binding.ty);
            if sig_seen.insert(sig) {
                dedup.push(c);
            }
        }
        candidates = dedup;
        last_narrowing_stage = OverloadCandidateNarrowingStage::SignatureDedup;
    }
    if candidates.len() > 1 {
        let ordinary: Vec<OverloadCandidate<'_>> = candidates
            .iter()
            .filter(|b| b.field_accessor.is_none())
            .cloned()
            .collect();
        if !ordinary.is_empty() {
            candidates = ordinary;
        }
        last_narrowing_stage = OverloadCandidateNarrowingStage::PreferOrdinaryFunction;
    }
    if candidates.len() > 1 {
        let concrete: Vec<OverloadCandidate<'_>> = candidates
            .iter()
            .filter(|b| !type_contains_unbound_var(ctx, b.binding.ty))
            .cloned()
            .collect();
        if !concrete.is_empty() {
            candidates = concrete;
        }
        last_narrowing_stage = OverloadCandidateNarrowingStage::PreferConcreteSignature;
    }
    if candidates.len() > 1 {
        let min_type_params = candidates
            .iter()
            .map(|b| b.type_param_count)
            .min()
            .unwrap_or(0);
        candidates = candidates
            .into_iter()
            .filter(|b| b.type_param_count == min_type_params)
            .collect();
        last_narrowing_stage = OverloadCandidateNarrowingStage::PreferFewerTypeParameters;
    }
    if candidates.len() > 1 {
        if crate::log::is_verbose() {
            for candidate in &candidates {
                overload_narrowing_log!(
                    "overload debug: specificity '{}' candidate {} score={}",
                    name,
                    function_signature_string(ctx, candidate.binding.ty),
                    candidate.instantiated_specificity
                );
            }
        }
        let max_specificity = candidates
            .iter()
            .map(|b| b.instantiated_specificity)
            .max()
            .unwrap_or(0);
        candidates = candidates
            .into_iter()
            .filter(|b| b.instantiated_specificity == max_specificity)
            .collect();
        last_narrowing_stage = OverloadCandidateNarrowingStage::PreferInstantiatedSpecificity;
    }
    if candidates.len() > 1 {
        let max_declared_specificity = candidates
            .iter()
            .map(|b| b.declared_specificity)
            .max()
            .unwrap_or(0);
        candidates = candidates
            .into_iter()
            .filter(|b| b.declared_specificity == max_declared_specificity)
            .collect();
        last_narrowing_stage = OverloadCandidateNarrowingStage::PreferDeclaredSpecificity;
    }
    if candidates.len() > 1 {
        return Err(OverloadAmbiguityReason::after_stage(
            last_narrowing_stage,
            candidates.len(),
        ));
    }

    Ok(candidates[0])
}
