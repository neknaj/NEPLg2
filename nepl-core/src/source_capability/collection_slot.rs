use crate::resource_primitives::{CollectionSlotBorrowPrimitive, CollectionSlotLifecyclePrimitive};
use crate::source_capability::proof_builder::SourceCapabilityProofFact;
use crate::source_capability::rule::SourceCapabilityProofSink;
use crate::span::Span;

pub(in crate::source_capability) fn collect_collection_slot_boundary_evidence(
    sink: &mut impl SourceCapabilityProofSink,
    name: &str,
    span: Span,
) {
    if let Some(primitive) = CollectionSlotLifecyclePrimitive::from_intrinsic_name(name) {
        sink.proof_mut().insert_fact(
            SourceCapabilityProofFact::CollectionSlotLifecycleBoundary(primitive),
            span,
        );
    }
    if let Some(primitive) = CollectionSlotBorrowPrimitive::from_intrinsic_name(name) {
        sink.proof_mut().insert_fact(
            SourceCapabilityProofFact::CollectionSlotBorrowBoundary(primitive),
            span,
        );
    }
}
