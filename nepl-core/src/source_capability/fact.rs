use crate::source_capability::compiler_memory_field::CompilerMemoryFieldEvidence;
use crate::source_capability::owner_aggregate::OwnerAggregateCapabilityEvidence;
use crate::source_capability::proof_builder::SourceCapabilityProofFact;

pub(in crate::source_capability) fn owner_aggregate_proof_fact(
    observed: Option<OwnerAggregateCapabilityEvidence>,
) -> Option<SourceCapabilityProofFact> {
    match observed {
        Some(OwnerAggregateCapabilityEvidence::FieldAccessor) => {
            Some(SourceCapabilityProofFact::OwnerAggregateFieldBoundary)
        }
        Some(OwnerAggregateCapabilityEvidence::Constructor(name)) => {
            Some(SourceCapabilityProofFact::OwnerAggregateConstructorBoundary(name))
        }
        None => None,
    }
}

pub(in crate::source_capability) fn compiler_memory_field_proof_fact(
    observed: Option<CompilerMemoryFieldEvidence>,
) -> Option<SourceCapabilityProofFact> {
    match observed {
        Some(evidence) => Some(SourceCapabilityProofFact::CompilerMemoryFieldBoundary(
            evidence.field(),
        )),
        None => None,
    }
}
