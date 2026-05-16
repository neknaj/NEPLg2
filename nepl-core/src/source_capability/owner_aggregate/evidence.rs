use alloc::string::String;

use crate::qualified_name::split_leading_qualifier;
use crate::runtime_helpers::helper_base_name;
use crate::source_capability::scope::SourceCapabilityScope;

use super::context::OwnerAggregateEvidenceContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::source_capability) enum OwnerAggregateCapabilityEvidence {
    Constructor(String),
    FieldAccessor,
}

pub(in crate::source_capability) fn owner_aggregate_symbol_evidence(
    symbol: &str,
    scope: &SourceCapabilityScope,
    context: &OwnerAggregateEvidenceContext,
) -> Option<OwnerAggregateCapabilityEvidence> {
    if source_symbol_shadowed(symbol, scope) {
        return None;
    }
    if context.is_core_field_accessor_symbol(symbol) {
        return Some(OwnerAggregateCapabilityEvidence::FieldAccessor);
    }
    constructor_evidence_name(symbol, context).map(OwnerAggregateCapabilityEvidence::Constructor)
}

pub(in crate::source_capability) fn owner_aggregate_intrinsic_evidence(
    symbol: &str,
) -> Option<OwnerAggregateCapabilityEvidence> {
    match helper_base_name(symbol) {
        "get_field" | "get_field_ref" => Some(OwnerAggregateCapabilityEvidence::FieldAccessor),
        _ => None,
    }
}

fn constructor_evidence_name(
    symbol: &str,
    context: &OwnerAggregateEvidenceContext,
) -> Option<String> {
    if crate::qualified_name::member_tail(symbol) != symbol {
        return None;
    }
    let base = helper_base_name(symbol);
    if context.is_enum_variant(base) {
        return None;
    }
    base.as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_uppercase())
        .then(|| String::from(base))
}

fn source_symbol_shadowed(symbol: &str, scope: &SourceCapabilityScope) -> bool {
    split_leading_qualifier(symbol)
        .map(|(qualifier, _)| scope.shadows(qualifier))
        .unwrap_or_else(|| scope.shadows(symbol))
}
