use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{Module, StructDef};
use crate::effects::{
    raw_body_direct_callees, raw_body_memory_operations, raw_memory_op_from_name,
};
use crate::hir::HirBody;
use crate::source_capability::memory_type_definition::compiler_memory_type_from_struct_def;
use crate::source_capability::owner_aggregate::{
    owner_aggregate_explicit_constructor_evidence, owner_aggregate_intrinsic_evidence,
    owner_aggregate_symbol_evidence, OwnerAggregateCapabilityEvidence,
    OwnerAggregateEvidenceContext,
};
use crate::source_capability::raw_evidence_gate::raw_symbol_shadow_allows_evidence;
use crate::source_capability::raw_memory::{RawAddressViewEvidence, RawMemoryStructuralEvidence};
use crate::source_capability::scope::SourceCapabilityScope;
use crate::source_capability::walk::{walk_module_capability_evidence, SourceCapabilityObserver};
use crate::source_map::{
    CompilerMemoryType, SourceCapabilities, SourceCapabilitySpan, SourceCapabilityUseSite,
};
use crate::span::Span;

pub(crate) fn module_source_capabilities(module: &Module) -> SourceCapabilities {
    collect_source_capability_proof(module).into_source_capabilities()
}

#[derive(Debug)]
struct RawOperationFunctionFrame {
    name: String,
    span: Span,
    has_raw_operation_evidence: bool,
}

#[derive(Debug, Default)]
struct SourceCapabilityProof {
    capabilities: SourceCapabilities,
}

impl SourceCapabilityProof {
    fn into_source_capabilities(self) -> SourceCapabilities {
        self.capabilities
    }

    fn insert_use_site(&mut self, use_site: SourceCapabilityUseSite) {
        self.capabilities.insert_use_site(use_site);
    }

    fn site_span(span: Span) -> SourceCapabilitySpan {
        SourceCapabilitySpan::from_span(span)
    }

    fn insert_raw_memory_structural_boundary(&mut self, span: Span) {
        self.insert_use_site(SourceCapabilityUseSite::RawMemoryStructuralBoundary {
            span: Self::site_span(span),
        });
    }

    fn insert_raw_address_view_boundary(&mut self, span: Span) {
        self.insert_use_site(SourceCapabilityUseSite::RawAddressViewBoundary {
            span: Self::site_span(span),
        });
    }

    fn insert_raw_memory_operation_boundary(
        &mut self,
        operation: crate::effects::RawMemoryOp,
        span: Span,
    ) {
        self.insert_use_site(SourceCapabilityUseSite::RawMemoryOperationBoundary {
            operation,
            span: Self::site_span(span),
        });
    }

    fn insert_raw_body_memory_operation_boundary(
        &mut self,
        operation: crate::effects::RawBodyMemoryOp,
        span: Span,
    ) {
        self.insert_use_site(SourceCapabilityUseSite::RawBodyMemoryOperationBoundary {
            operation,
            span: Self::site_span(span),
        });
    }

    fn insert_owner_aggregate_evidence(
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

    fn insert_compiler_memory_type_definition(
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

fn collect_source_capability_proof(module: &Module) -> SourceCapabilityProof {
    let owner_context = OwnerAggregateEvidenceContext::from_module(module);
    let mut collector = SourceCapabilityProofCollector {
        owner_context: &owner_context,
        proof: SourceCapabilityProof::default(),
        raw_operation_function_frames: Vec::new(),
    };
    walk_module_capability_evidence(module, &mut collector);
    collector.proof
}

struct SourceCapabilityProofCollector<'a> {
    owner_context: &'a OwnerAggregateEvidenceContext,
    proof: SourceCapabilityProof,
    raw_operation_function_frames: Vec<RawOperationFunctionFrame>,
}

impl SourceCapabilityProofCollector<'_> {
    fn record_raw_operation_evidence(&mut self) {
        if let Some(frame) = self.raw_operation_function_frames.last_mut() {
            frame.has_raw_operation_evidence = true;
        }
    }

    fn collect_symbol_evidence(&mut self, symbol: &str, span: Span, scope: &SourceCapabilityScope) {
        self.collect_raw_symbol_evidence(symbol, span, scope);
        self.collect_owner_aggregate_symbol_evidence(symbol, span, scope);
    }

    fn collect_raw_symbol_evidence(
        &mut self,
        symbol: &str,
        span: Span,
        scope: &SourceCapabilityScope,
    ) {
        if let Some(kind) = scope.shadow_kind_symbol_or_qualifier(symbol) {
            let current_function = self
                .raw_operation_function_frames
                .last()
                .map(|frame| frame.name.as_str());
            if !raw_symbol_shadow_allows_evidence(symbol, kind, current_function) {
                return;
            }
        }
        self.collect_raw_builtin_evidence(symbol, span);
    }

    fn collect_raw_builtin_evidence(&mut self, symbol: &str, span: Span) {
        if RawMemoryStructuralEvidence::from_symbol(symbol).is_some() {
            self.proof.insert_raw_memory_structural_boundary(span);
        }
        if RawAddressViewEvidence::from_symbol(symbol).is_some() {
            self.proof.insert_raw_address_view_boundary(span);
        }
        if let Some(operation) = raw_memory_op_from_name(symbol) {
            self.proof
                .insert_raw_memory_operation_boundary(operation, span);
            self.record_raw_operation_evidence();
        }
    }

    fn collect_raw_body_evidence(&mut self, body: HirBody, span: Span) {
        for operation in raw_body_memory_operations(&body) {
            self.proof
                .insert_raw_body_memory_operation_boundary(operation, span);
            self.record_raw_operation_evidence();
        }
        for callee in raw_body_direct_callees(&body) {
            if let Some(operation) = raw_memory_op_from_name(&callee) {
                self.proof
                    .insert_raw_memory_operation_boundary(operation, span);
                self.record_raw_operation_evidence();
            }
        }
    }

    fn collect_owner_aggregate_symbol_evidence(
        &mut self,
        symbol: &str,
        span: Span,
        scope: &SourceCapabilityScope,
    ) {
        self.proof.insert_owner_aggregate_evidence(
            owner_aggregate_symbol_evidence(symbol, scope, self.owner_context),
            span,
        );
    }

    fn collect_owner_aggregate_explicit_constructor_evidence(
        &mut self,
        symbol: &str,
        span: Span,
        scope: &SourceCapabilityScope,
    ) {
        self.proof.insert_owner_aggregate_evidence(
            owner_aggregate_explicit_constructor_evidence(symbol, scope, self.owner_context),
            span,
        );
    }
}

impl SourceCapabilityObserver for SourceCapabilityProofCollector<'_> {
    fn observe_named_function_start(
        &mut self,
        name: &str,
        span: Span,
        _scope: &SourceCapabilityScope,
    ) {
        self.raw_operation_function_frames
            .push(RawOperationFunctionFrame {
                name: String::from(name),
                span,
                has_raw_operation_evidence: false,
            });
    }

    fn observe_named_function_end(
        &mut self,
        name: &str,
        _span: Span,
        _scope: &SourceCapabilityScope,
    ) {
        if let Some(frame) = self.raw_operation_function_frames.pop() {
            if !frame.has_raw_operation_evidence {
                return;
            }
            if let Some(operation) = raw_memory_op_from_name(name) {
                self.proof
                    .insert_raw_memory_operation_boundary(operation, frame.span);
            }
        }
    }

    fn observe_fn_alias_target(&mut self, symbol: &str, span: Span, scope: &SourceCapabilityScope) {
        self.collect_symbol_evidence(symbol, span, scope);
    }

    fn observe_struct_definition(&mut self, def: &StructDef) {
        if let Some(memory_type) = compiler_memory_type_from_struct_def(def) {
            self.proof
                .insert_compiler_memory_type_definition(memory_type, def.name.span);
        }
    }

    fn observe_call_head_symbol(
        &mut self,
        symbol: &str,
        span: Span,
        scope: &SourceCapabilityScope,
    ) {
        self.collect_symbol_evidence(symbol, span, scope);
    }

    fn observe_explicit_constructor_symbol(
        &mut self,
        symbol: &str,
        span: Span,
        scope: &SourceCapabilityScope,
    ) {
        self.collect_owner_aggregate_explicit_constructor_evidence(symbol, span, scope);
    }

    fn observe_intrinsic(&mut self, name: &str, span: Span, _scope: &SourceCapabilityScope) {
        self.collect_raw_builtin_evidence(name, span);
        self.proof
            .insert_owner_aggregate_evidence(owner_aggregate_intrinsic_evidence(name), span);
    }

    fn observe_raw_body(&mut self, body: HirBody, span: Span) {
        self.collect_raw_body_evidence(body, span);
    }
}
