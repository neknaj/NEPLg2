use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{Module, PrefixExpr, StructDef};
use crate::effects::{
    raw_body_direct_callee_effects, raw_body_memory_operations, raw_memory_op_from_name,
    RawBodyDirectCallee, RawBodyMemoryOp, RawMemoryOp,
};
use crate::hir::HirBody;
use crate::source_capability::binding::SourceCapabilityBindingKind;
use crate::source_capability::compiler_memory_field::{
    compiler_memory_field_intrinsic_evidence, compiler_memory_field_symbol_evidence,
};
use crate::source_capability::memory_type_definition::compiler_memory_type_from_struct_def;
use crate::source_capability::owner_aggregate::{
    owner_aggregate_explicit_constructor_evidence, owner_aggregate_intrinsic_evidence,
    owner_aggregate_symbol_evidence, OwnerAggregateEvidenceContext,
};
use crate::source_capability::proof_builder::SourceCapabilityProof;
use crate::source_capability::raw_evidence_gate::raw_symbol_shadow_allows_evidence;
use crate::source_capability::raw_memory::{RawAddressViewEvidence, RawMemoryStructuralEvidence};
use crate::source_capability::raw_operation_proof::{
    RawOperationBoundaryContract, RawOperationFunctionEvidence,
};
use crate::source_capability::scope::SourceCapabilityScope;
use crate::source_capability::top_level_raw_calls::{
    apply_top_level_raw_call_evidence, RawOperationFunctionProof, TopLevelRawCallSite,
};
use crate::source_capability::walk::{walk_module_capability_evidence, SourceCapabilityObserver};
use crate::source_map::SourceCapabilities;
use crate::span::Span;

pub(crate) fn module_source_capabilities(module: &Module) -> SourceCapabilities {
    collect_source_capability_proof(module).into_source_capabilities()
}

#[derive(Debug)]
struct RawOperationFunctionFrame {
    name: String,
    span: Span,
    boundary_contract: RawOperationBoundaryContract,
    evidence: RawOperationFunctionEvidence,
    top_level_raw_calls: Vec<TopLevelRawCallSite>,
}

fn collect_source_capability_proof(module: &Module) -> SourceCapabilityProof {
    let owner_context = OwnerAggregateEvidenceContext::from_module(module);
    let mut collector = SourceCapabilityProofCollector {
        owner_context: &owner_context,
        proof: SourceCapabilityProof::default(),
        raw_operation_function_frames: Vec::new(),
        completed_raw_operation_function_frames: Vec::new(),
    };
    walk_module_capability_evidence(module, &mut collector);
    apply_top_level_raw_call_evidence(
        &collector.completed_raw_operation_function_frames,
        &mut collector.proof.capabilities,
    );
    collector.proof
}

struct SourceCapabilityProofCollector<'a> {
    owner_context: &'a OwnerAggregateEvidenceContext,
    proof: SourceCapabilityProof,
    raw_operation_function_frames: Vec<RawOperationFunctionFrame>,
    completed_raw_operation_function_frames: Vec<RawOperationFunctionProof>,
}

impl SourceCapabilityProofCollector<'_> {
    fn record_raw_operation_evidence(&mut self, operation: RawMemoryOp) {
        if let Some(frame) = self.raw_operation_function_frames.last_mut() {
            frame.evidence.insert_operation(operation);
        }
    }

    fn record_raw_body_operation_evidence(&mut self, operation: RawBodyMemoryOp) {
        if let Some(frame) = self.raw_operation_function_frames.last_mut() {
            frame.evidence.insert_raw_body_operation(operation);
        }
    }

    fn record_top_level_raw_call_evidence(
        &mut self,
        target: &str,
        operation: RawMemoryOp,
        span: Span,
    ) {
        if let Some(frame) = self.raw_operation_function_frames.last_mut() {
            frame.top_level_raw_calls.push(TopLevelRawCallSite {
                target: String::from(target),
                operation,
                span,
            });
        }
    }

    fn collect_symbol_evidence(
        &mut self,
        symbol: &str,
        span: Span,
        selector: Option<&str>,
        scope: &SourceCapabilityScope,
    ) {
        self.collect_raw_symbol_evidence(symbol, span, scope);
        self.collect_owner_aggregate_symbol_evidence(symbol, span, scope);
        self.collect_compiler_memory_field_symbol_evidence(symbol, selector, span, scope);
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
                if matches!(kind, SourceCapabilityBindingKind::TopLevelCallable) {
                    if let Some(operation) = raw_memory_op_from_name(symbol) {
                        self.record_top_level_raw_call_evidence(symbol, operation, span);
                    }
                }
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
            self.record_raw_operation_evidence(operation);
        }
    }

    fn collect_raw_body_evidence(&mut self, body: HirBody, span: Span) {
        for operation in raw_body_memory_operations(&body) {
            self.proof
                .insert_raw_body_memory_operation_boundary(operation, span);
            self.record_raw_body_operation_evidence(operation);
        }
        for callee in raw_body_direct_callee_effects(&body) {
            if let RawBodyDirectCallee::RawMemory { operation, .. } = callee {
                self.proof
                    .insert_raw_memory_operation_boundary(operation, span);
                self.record_raw_operation_evidence(operation);
            }
        }
    }

    fn finish_raw_operation_function_frame(&mut self, frame: RawOperationFunctionFrame) {
        self.completed_raw_operation_function_frames
            .push(RawOperationFunctionProof {
                name: frame.name,
                span: frame.span,
                boundary_contract: frame.boundary_contract,
                evidence: frame.evidence,
                top_level_raw_calls: frame.top_level_raw_calls,
            });
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

    fn collect_compiler_memory_field_symbol_evidence(
        &mut self,
        symbol: &str,
        selector: Option<&str>,
        span: Span,
        scope: &SourceCapabilityScope,
    ) {
        self.proof.insert_compiler_memory_field_evidence(
            compiler_memory_field_symbol_evidence(symbol, selector, scope, self.owner_context),
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
                boundary_contract: raw_memory_boundary_contract_from_function_name(name),
                evidence: RawOperationFunctionEvidence::default(),
                top_level_raw_calls: Vec::new(),
            });
    }

    fn observe_named_function_end(
        &mut self,
        _name: &str,
        _span: Span,
        _scope: &SourceCapabilityScope,
    ) {
        if let Some(frame) = self.raw_operation_function_frames.pop() {
            self.finish_raw_operation_function_frame(frame);
        }
    }

    fn observe_fn_alias_target(&mut self, symbol: &str, span: Span, scope: &SourceCapabilityScope) {
        self.collect_symbol_evidence(symbol, span, None, scope);
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
        selector: Option<&str>,
        scope: &SourceCapabilityScope,
    ) {
        self.collect_symbol_evidence(symbol, span, selector, scope);
    }

    fn observe_explicit_constructor_symbol(
        &mut self,
        symbol: &str,
        span: Span,
        scope: &SourceCapabilityScope,
    ) {
        self.collect_owner_aggregate_explicit_constructor_evidence(symbol, span, scope);
    }

    fn observe_intrinsic(
        &mut self,
        name: &str,
        args: &[PrefixExpr],
        span: Span,
        _scope: &SourceCapabilityScope,
    ) {
        self.collect_raw_builtin_evidence(name, span);
        self.proof
            .insert_owner_aggregate_evidence(owner_aggregate_intrinsic_evidence(name), span);
        self.proof.insert_compiler_memory_field_evidence(
            compiler_memory_field_intrinsic_evidence(name, args),
            span,
        );
    }

    fn observe_raw_body(&mut self, body: HirBody, span: Span) {
        self.collect_raw_body_evidence(body, span);
    }
}

fn raw_memory_boundary_contract_from_function_name(name: &str) -> RawOperationBoundaryContract {
    raw_memory_op_from_name(name)
        .map(RawOperationBoundaryContract::RawMemoryOperation)
        .unwrap_or(RawOperationBoundaryContract::None)
}
