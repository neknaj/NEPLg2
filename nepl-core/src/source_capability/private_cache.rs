use crate::effects::{private_cache_op_from_name, PrivateEffectRegion};
use crate::source_capability::proof_builder::SourceCapabilityProofFact;
use crate::source_capability::rule::SourceCapabilityProofSink;
use crate::span::Span;

pub(in crate::source_capability) fn collect_private_cache_boundary_evidence(
    sink: &mut impl SourceCapabilityProofSink,
    name: &str,
    span: Span,
) {
    if let Some(operation) = private_cache_op_from_name(name) {
        sink.proof_mut().insert_fact(
            SourceCapabilityProofFact::PrivateCacheBoundary {
                operation,
                region: PrivateEffectRegion::UnsealedIntrinsic,
            },
            span,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::{PrivateCacheOp, RawBodyMemoryOp, RawMemoryOp};
    use crate::source_capability::owner_aggregate::OwnerAggregateEvidenceContext;
    use crate::source_capability::proof_builder::SourceCapabilityProof;

    #[derive(Debug, Default)]
    struct TestProofSink {
        proof: SourceCapabilityProof,
        owner_context: OwnerAggregateEvidenceContext,
    }

    impl SourceCapabilityProofSink for TestProofSink {
        fn proof_mut(&mut self) -> &mut SourceCapabilityProof {
            &mut self.proof
        }

        fn owner_context(&self) -> &OwnerAggregateEvidenceContext {
            &self.owner_context
        }

        fn current_raw_operation_function_name(&self) -> Option<&str> {
            None
        }

        fn record_raw_operation_evidence(&mut self, _operation: RawMemoryOp) {}

        fn record_raw_body_operation_evidence(&mut self, _operation: RawBodyMemoryOp) {}

        fn record_top_level_raw_call_evidence(
            &mut self,
            _target: &str,
            _operation: RawMemoryOp,
            _span: Span,
        ) {
        }
    }

    #[test]
    fn private_cache_op_all_covers_intrinsic_name_classification() {
        for operation in PrivateCacheOp::ALL {
            assert_eq!(
                private_cache_op_from_name(operation.intrinsic_name()),
                Some(operation)
            );
        }
        assert_eq!(private_cache_op_from_name("memo_call"), None);
    }

    #[test]
    fn private_cache_boundary_evidence_is_operation_exact_for_all_private_cache_ops() {
        for operation in PrivateCacheOp::ALL {
            let span = Span::new(crate::span::FileId(operation as u32), 10, 20);
            let shifted = Span::new(crate::span::FileId(operation as u32), 11, 21);
            let mut sink = TestProofSink::default();

            collect_private_cache_boundary_evidence(
                &mut sink,
                operation.intrinsic_name(),
                span,
            );

            let capabilities = sink.proof.into_source_capabilities();
            assert!(capabilities.allows_private_cache_boundary_in_region_at(
                operation,
                PrivateEffectRegion::UnsealedIntrinsic,
                span
            ));
            for other in PrivateCacheOp::ALL {
                if other != operation {
                    assert!(
                        !capabilities.allows_private_cache_boundary_in_region_at(
                            other,
                            PrivateEffectRegion::UnsealedIntrinsic,
                            span
                        ),
                        "{operation} proof must not authorize {other}"
                    );
                }
            }
            assert!(!capabilities.allows_private_cache_boundary_in_region_at(
                operation,
                PrivateEffectRegion::UnsealedIntrinsic,
                shifted
            ));
        }
    }
}
