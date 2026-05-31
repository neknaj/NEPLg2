use crate::ast::{PrefixExpr, StructDef};
use crate::effects::{
    raw_body_direct_callee_effects, raw_body_memory_operations, raw_memory_op_from_name,
    RawBodyDirectCallee, RawBodyMemoryOp, RawMemoryOp,
};
use crate::hir::HirBody;
use crate::source_capability::binding::SourceCapabilityBindingKind;
use crate::source_capability::collection_slot::collect_collection_slot_boundary_evidence;
use crate::source_capability::compiler_memory_field::{
    compiler_memory_field_intrinsic_evidence, compiler_memory_field_symbol_evidence,
};
use crate::source_capability::fact::{
    compiler_memory_field_proof_fact, owner_aggregate_proof_fact,
};
use crate::source_capability::memory_type_definition::compiler_memory_type_from_struct_def;
use crate::source_capability::owner_aggregate::{
    owner_aggregate_explicit_constructor_evidence, owner_aggregate_intrinsic_evidence,
    owner_aggregate_symbol_evidence, OwnerAggregateEvidenceContext,
};
use crate::source_capability::private_cache::collect_private_cache_boundary_evidence;
use crate::source_capability::proof_builder::{SourceCapabilityProof, SourceCapabilityProofFact};
use crate::source_capability::raw_builtin_evidence::collect_raw_builtin_evidence;
use crate::source_capability::raw_evidence_gate::raw_symbol_shadow_allows_evidence;
use crate::source_capability::raw_operation_proof::RawOperationBoundaryContract;
use crate::source_capability::scope::SourceCapabilityScope;
use crate::span::Span;

pub(in crate::source_capability) enum SourceCapabilityProofEvent<'a> {
    Symbol {
        symbol: &'a str,
        span: Span,
        selector: Option<&'a str>,
        scope: &'a SourceCapabilityScope,
    },
    ExplicitConstructor {
        symbol: &'a str,
        span: Span,
        scope: &'a SourceCapabilityScope,
    },
    StructDefinition {
        def: &'a StructDef,
    },
    Intrinsic {
        name: &'a str,
        args: &'a [PrefixExpr],
        span: Span,
        name_span: Span,
        collection_slot_surface: CollectionSlotLifecycleSourceSurface,
    },
    RawBody {
        body: HirBody,
        span: Span,
    },
    PropagatedRawOperation {
        operation: RawMemoryOp,
        span: Span,
    },
}

pub(in crate::source_capability) trait SourceCapabilityProofSink {
    fn proof_mut(&mut self) -> &mut SourceCapabilityProof;
    fn owner_context(&self) -> &OwnerAggregateEvidenceContext;
    fn current_raw_operation_function_name(&self) -> Option<&str>;
    fn record_raw_operation_evidence(&mut self, operation: RawMemoryOp);
    fn record_raw_body_operation_evidence(&mut self, operation: RawBodyMemoryOp);
    fn record_top_level_raw_call_evidence(
        &mut self,
        target: &str,
        operation: RawMemoryOp,
        span: Span,
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::source_capability) enum CollectionSlotLifecycleSourceSurface {
    InternalCallable,
    PublicCallableSurface,
}

pub(in crate::source_capability) fn dispatch_source_capability_proof_event(
    sink: &mut impl SourceCapabilityProofSink,
    event: SourceCapabilityProofEvent<'_>,
) {
    match event {
        SourceCapabilityProofEvent::Symbol {
            symbol,
            span,
            selector,
            scope,
        } => {
            collect_raw_symbol_evidence(sink, symbol, span, scope);
            collect_owner_aggregate_symbol_evidence(sink, symbol, span, scope);
            collect_compiler_memory_field_symbol_evidence(sink, symbol, selector, span, scope);
        }
        SourceCapabilityProofEvent::ExplicitConstructor {
            symbol,
            span,
            scope,
        } => collect_owner_aggregate_explicit_constructor_evidence(sink, symbol, span, scope),
        SourceCapabilityProofEvent::StructDefinition { def } => {
            if let Some(memory_type) = compiler_memory_type_from_struct_def(def) {
                sink.proof_mut().insert_fact(
                    SourceCapabilityProofFact::CompilerMemoryTypeDefinition(memory_type),
                    def.name.span,
                );
            }
        }
        SourceCapabilityProofEvent::Intrinsic {
            name,
            args,
            span,
            name_span,
            collection_slot_surface,
        } => {
            collect_raw_builtin_evidence(sink, name, span);
            collect_private_cache_boundary_evidence(sink, name, name_span);
            match collection_slot_surface {
                CollectionSlotLifecycleSourceSurface::InternalCallable => {
                    collect_collection_slot_boundary_evidence(sink, name, name_span);
                }
                CollectionSlotLifecycleSourceSurface::PublicCallableSurface => {}
            }
            insert_proof_fact(
                sink,
                owner_aggregate_proof_fact(owner_aggregate_intrinsic_evidence(name)),
                span,
            );
            insert_proof_fact(
                sink,
                compiler_memory_field_proof_fact(compiler_memory_field_intrinsic_evidence(
                    name, args,
                )),
                span,
            );
        }
        SourceCapabilityProofEvent::RawBody { body, span } => {
            collect_raw_body_evidence(sink, body, span);
        }
        SourceCapabilityProofEvent::PropagatedRawOperation { operation, span } => {
            sink.proof_mut().insert_fact(
                SourceCapabilityProofFact::RawMemoryOperationBoundary(operation),
                span,
            );
        }
    }
}

pub(in crate::source_capability) fn raw_memory_boundary_contract_from_function_name(
    name: &str,
) -> RawOperationBoundaryContract {
    raw_memory_op_from_name(name)
        .map(RawOperationBoundaryContract::RawMemoryOperation)
        .unwrap_or(RawOperationBoundaryContract::None)
}

fn collect_raw_symbol_evidence(
    sink: &mut impl SourceCapabilityProofSink,
    symbol: &str,
    span: Span,
    scope: &SourceCapabilityScope,
) {
    if let Some(kind) = scope.shadow_kind_symbol_or_qualifier(symbol) {
        let current_function = sink.current_raw_operation_function_name();
        if !raw_symbol_shadow_allows_evidence(symbol, kind, current_function) {
            if matches!(kind, SourceCapabilityBindingKind::TopLevelCallable) {
                if let Some(operation) = raw_memory_op_from_name(symbol) {
                    sink.record_top_level_raw_call_evidence(symbol, operation, span);
                }
            }
            return;
        }
    }
    collect_raw_builtin_evidence(sink, symbol, span);
}

fn collect_raw_body_evidence(sink: &mut impl SourceCapabilityProofSink, body: HirBody, span: Span) {
    for operation in raw_body_memory_operations(&body) {
        sink.proof_mut().insert_fact(
            SourceCapabilityProofFact::RawBodyMemoryOperationBoundary(operation),
            span,
        );
        sink.record_raw_body_operation_evidence(operation);
    }
    for callee in raw_body_direct_callee_effects(&body) {
        if let RawBodyDirectCallee::RawMemory { operation, .. } = callee {
            sink.proof_mut().insert_fact(
                SourceCapabilityProofFact::RawMemoryOperationBoundary(operation),
                span,
            );
            sink.record_raw_operation_evidence(operation);
        }
    }
}

fn collect_owner_aggregate_symbol_evidence(
    sink: &mut impl SourceCapabilityProofSink,
    symbol: &str,
    span: Span,
    scope: &SourceCapabilityScope,
) {
    let observed = owner_aggregate_symbol_evidence(symbol, scope, sink.owner_context());
    insert_proof_fact(sink, owner_aggregate_proof_fact(observed), span);
}

fn collect_owner_aggregate_explicit_constructor_evidence(
    sink: &mut impl SourceCapabilityProofSink,
    symbol: &str,
    span: Span,
    scope: &SourceCapabilityScope,
) {
    let observed =
        owner_aggregate_explicit_constructor_evidence(symbol, scope, sink.owner_context());
    insert_proof_fact(sink, owner_aggregate_proof_fact(observed), span);
}

fn collect_compiler_memory_field_symbol_evidence(
    sink: &mut impl SourceCapabilityProofSink,
    symbol: &str,
    selector: Option<&str>,
    span: Span,
    scope: &SourceCapabilityScope,
) {
    let observed =
        compiler_memory_field_symbol_evidence(symbol, selector, scope, sink.owner_context());
    insert_proof_fact(sink, compiler_memory_field_proof_fact(observed), span);
}

fn insert_proof_fact(
    sink: &mut impl SourceCapabilityProofSink,
    fact: Option<SourceCapabilityProofFact>,
    span: Span,
) {
    if let Some(fact) = fact {
        sink.proof_mut().insert_fact(fact, span);
    }
}
