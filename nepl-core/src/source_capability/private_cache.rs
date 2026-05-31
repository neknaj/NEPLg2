use crate::effects::PrivateCacheOp;
use crate::source_capability::proof_builder::SourceCapabilityProofFact;
use crate::source_capability::rule::SourceCapabilityProofSink;
use crate::span::Span;

pub(in crate::source_capability) fn collect_private_cache_boundary_evidence(
    sink: &mut impl SourceCapabilityProofSink,
    name: &str,
    span: Span,
) {
    if let Some(operation) = private_cache_operation_from_intrinsic_name(name) {
        sink.proof_mut().insert_fact(
            SourceCapabilityProofFact::PrivateCacheBoundary(operation),
            span,
        );
    }
}

fn private_cache_operation_from_intrinsic_name(name: &str) -> Option<PrivateCacheOp> {
    match name {
        "private_cache_create" => Some(PrivateCacheOp::Create),
        "private_cache_lookup" => Some(PrivateCacheOp::Lookup),
        "private_cache_insert" => Some(PrivateCacheOp::Insert),
        "private_cache_drop" => Some(PrivateCacheOp::Drop),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_cache_intrinsic_names_map_to_typed_operations() {
        assert_eq!(
            private_cache_operation_from_intrinsic_name("private_cache_create"),
            Some(PrivateCacheOp::Create)
        );
        assert_eq!(
            private_cache_operation_from_intrinsic_name("private_cache_lookup"),
            Some(PrivateCacheOp::Lookup)
        );
        assert_eq!(
            private_cache_operation_from_intrinsic_name("private_cache_insert"),
            Some(PrivateCacheOp::Insert)
        );
        assert_eq!(
            private_cache_operation_from_intrinsic_name("private_cache_drop"),
            Some(PrivateCacheOp::Drop)
        );
        assert_eq!(
            private_cache_operation_from_intrinsic_name("memo_call"),
            None
        );
    }
}
