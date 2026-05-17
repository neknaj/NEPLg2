use alloc::string::String;

use crate::effects::{RawBodyMemoryOp, RawMemoryOp};
use crate::source_map::{
    CompilerMemoryField, CompilerMemoryType, SourceCapabilities, SourceCapabilitySpan,
    SourceCapabilityUseSite,
};
use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::source_capability) enum SourceCapabilityProofFact {
    RawMemoryStructuralBoundary,
    RawAddressViewBoundary,
    RawMemoryOperationBoundary(RawMemoryOp),
    RawBodyMemoryOperationBoundary(RawBodyMemoryOp),
    OwnerAggregateFieldBoundary,
    OwnerAggregateConstructorBoundary(String),
    CompilerMemoryFieldBoundary(CompilerMemoryField),
    CompilerMemoryTypeDefinition(CompilerMemoryType),
}

#[derive(Debug, Default)]
pub(in crate::source_capability) struct SourceCapabilityProof {
    capabilities: SourceCapabilities,
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

    pub(in crate::source_capability) fn insert_fact(
        &mut self,
        fact: SourceCapabilityProofFact,
        span: Span,
    ) {
        match fact {
            SourceCapabilityProofFact::RawMemoryStructuralBoundary => {
                self.insert_use_site(SourceCapabilityUseSite::RawMemoryStructuralBoundary {
                    span: Self::site_span(span),
                });
            }
            SourceCapabilityProofFact::RawAddressViewBoundary => {
                self.insert_use_site(SourceCapabilityUseSite::RawAddressViewBoundary {
                    span: Self::site_span(span),
                });
            }
            SourceCapabilityProofFact::RawMemoryOperationBoundary(operation) => {
                self.insert_use_site(SourceCapabilityUseSite::RawMemoryOperationBoundary {
                    operation,
                    span: Self::site_span(span),
                });
            }
            SourceCapabilityProofFact::RawBodyMemoryOperationBoundary(operation) => {
                self.insert_use_site(SourceCapabilityUseSite::RawBodyMemoryOperationBoundary {
                    operation,
                    span: Self::site_span(span),
                });
            }
            SourceCapabilityProofFact::OwnerAggregateFieldBoundary => {
                self.insert_use_site(SourceCapabilityUseSite::OwnerAggregateFieldBoundary {
                    span: Self::site_span(span),
                });
            }
            SourceCapabilityProofFact::OwnerAggregateConstructorBoundary(name) => {
                self.insert_use_site(SourceCapabilityUseSite::OwnerAggregateConstructorBoundary {
                    name,
                    span: Self::site_span(span),
                });
            }
            SourceCapabilityProofFact::CompilerMemoryFieldBoundary(field) => {
                self.insert_use_site(SourceCapabilityUseSite::CompilerMemoryFieldBoundary {
                    field,
                    span: Self::site_span(span),
                });
            }
            SourceCapabilityProofFact::CompilerMemoryTypeDefinition(memory_type) => {
                self.insert_use_site(SourceCapabilityUseSite::CompilerMemoryTypeDefinition {
                    memory_type,
                    span: Self::site_span(span),
                });
            }
        }
    }
}
