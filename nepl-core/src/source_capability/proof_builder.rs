use crate::effects::{RawBodyMemoryOp, RawMemoryOp};
use crate::source_capability::owner_aggregate::OwnerAggregateCapabilityEvidence;
use crate::source_map::{
    CompilerMemoryType, SourceCapabilities, SourceCapabilitySpan, SourceCapabilityUseSite,
};
use crate::span::Span;

#[derive(Debug, Default)]
pub(in crate::source_capability) struct SourceCapabilityProof {
    pub(in crate::source_capability) capabilities: SourceCapabilities,
}

impl SourceCapabilityProof {
    pub(in crate::source_capability) fn into_source_capabilities(self) -> SourceCapabilities {
        self.capabilities
    }

    fn insert_use_site(&mut self, use_site: SourceCapabilityUseSite) {
        self.capabilities.insert_use_site(use_site);
    }

    fn site_span(span: Span) -> SourceCapabilitySpan {
        SourceCapabilitySpan::from_span(span)
    }

    pub(in crate::source_capability) fn insert_raw_memory_structural_boundary(
        &mut self,
        span: Span,
    ) {
        self.insert_use_site(SourceCapabilityUseSite::RawMemoryStructuralBoundary {
            span: Self::site_span(span),
        });
    }

    pub(in crate::source_capability) fn insert_raw_address_view_boundary(&mut self, span: Span) {
        self.insert_use_site(SourceCapabilityUseSite::RawAddressViewBoundary {
            span: Self::site_span(span),
        });
    }

    pub(in crate::source_capability) fn insert_raw_memory_operation_boundary(
        &mut self,
        operation: RawMemoryOp,
        span: Span,
    ) {
        self.insert_use_site(SourceCapabilityUseSite::RawMemoryOperationBoundary {
            operation,
            span: Self::site_span(span),
        });
    }

    pub(in crate::source_capability) fn insert_raw_body_memory_operation_boundary(
        &mut self,
        operation: RawBodyMemoryOp,
        span: Span,
    ) {
        self.insert_use_site(SourceCapabilityUseSite::RawBodyMemoryOperationBoundary {
            operation,
            span: Self::site_span(span),
        });
    }

    pub(in crate::source_capability) fn insert_owner_aggregate_evidence(
        &mut self,
        observed: Option<OwnerAggregateCapabilityEvidence>,
        span: Span,
    ) {
        match observed {
            Some(OwnerAggregateCapabilityEvidence::FieldAccessor) => {
                self.insert_use_site(SourceCapabilityUseSite::OwnerAggregateFieldBoundary {
                    span: Self::site_span(span),
                });
                self.insert_use_site(SourceCapabilityUseSite::CompilerMemoryFieldBoundary {
                    span: Self::site_span(span),
                });
            }
            Some(OwnerAggregateCapabilityEvidence::Constructor(name)) => {
                self.insert_use_site(SourceCapabilityUseSite::OwnerAggregateConstructorBoundary {
                    name,
                    span: Self::site_span(span),
                });
            }
            None => {}
        }
    }

    pub(in crate::source_capability) fn insert_compiler_memory_type_definition(
        &mut self,
        memory_type: CompilerMemoryType,
        span: Span,
    ) {
        self.insert_use_site(SourceCapabilityUseSite::CompilerMemoryTypeDefinition {
            memory_type,
            span: Self::site_span(span),
        });
    }
}
