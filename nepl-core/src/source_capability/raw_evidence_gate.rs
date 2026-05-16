use crate::effects::raw_memory_op_from_name;
use crate::source_capability::raw_memory::{RawAddressViewEvidence, RawMemoryStructuralEvidence};
use crate::source_capability::scope::SourceCapabilityBindingKind;

pub(super) fn raw_symbol_shadow_allows_evidence(
    symbol: &str,
    kind: SourceCapabilityBindingKind,
    current_function: Option<&str>,
) -> bool {
    kind == SourceCapabilityBindingKind::TopLevelCallable
        && current_function.is_some_and(|name| name == symbol)
        && raw_symbol_has_source_evidence(symbol)
}

fn raw_symbol_has_source_evidence(symbol: &str) -> bool {
    RawMemoryStructuralEvidence::from_symbol(symbol).is_some()
        || RawAddressViewEvidence::from_symbol(symbol).is_some()
        || raw_memory_op_from_name(symbol).is_some()
}
