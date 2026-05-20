use crate::effects::raw_memory_op_from_name;
use crate::source_capability::proof_builder::SourceCapabilityProofFact;
use crate::source_capability::raw_memory::{
    OwnerTokenConstructEvidence, RawAddressAliasEvidence, RawAddressViewEvidence,
    RawMemoryStructuralEvidence,
};
use crate::source_capability::rule::SourceCapabilityProofSink;
use crate::span::Span;

pub(in crate::source_capability) fn collect_raw_builtin_evidence(
    sink: &mut impl SourceCapabilityProofSink,
    symbol: &str,
    span: Span,
) {
    if RawMemoryStructuralEvidence::from_symbol(symbol).is_some() {
        sink.proof_mut()
            .insert_fact(SourceCapabilityProofFact::RawMemoryStructuralBoundary, span);
    }
    if RawAddressViewEvidence::from_symbol(symbol).is_some() {
        sink.proof_mut()
            .insert_fact(SourceCapabilityProofFact::RawAddressViewBoundary, span);
    }
    if RawAddressAliasEvidence::from_symbol(symbol).is_some() {
        sink.proof_mut()
            .insert_fact(SourceCapabilityProofFact::RawAddressAliasBoundary, span);
    }
    if OwnerTokenConstructEvidence::from_symbol(symbol).is_some() {
        sink.proof_mut()
            .insert_fact(SourceCapabilityProofFact::OwnerTokenConstructBoundary, span);
    }
    if let Some(operation) = raw_memory_op_from_name(symbol) {
        sink.proof_mut().insert_fact(
            SourceCapabilityProofFact::RawMemoryOperationBoundary(operation),
            span,
        );
        sink.record_raw_operation_evidence(operation);
    }
}
