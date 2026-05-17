use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{Module, PrefixExpr, StructDef};
use crate::effects::{RawBodyMemoryOp, RawMemoryOp};
use crate::hir::HirBody;
use crate::source_capability::owner_aggregate::OwnerAggregateEvidenceContext;
use crate::source_capability::proof_builder::SourceCapabilityProof;
use crate::source_capability::raw_operation_proof::{
    RawOperationBoundaryContract, RawOperationFunctionEvidence,
};
use crate::source_capability::rule::{
    dispatch_source_capability_proof_event, raw_memory_boundary_contract_from_function_name,
    SourceCapabilityProofEvent, SourceCapabilityProofSink,
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
        &mut collector.proof,
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
}

impl SourceCapabilityProofSink for SourceCapabilityProofCollector<'_> {
    fn proof_mut(&mut self) -> &mut SourceCapabilityProof {
        &mut self.proof
    }

    fn owner_context(&self) -> &OwnerAggregateEvidenceContext {
        self.owner_context
    }

    fn current_raw_operation_function_name(&self) -> Option<&str> {
        self.raw_operation_function_frames
            .last()
            .map(|frame| frame.name.as_str())
    }

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
        dispatch_source_capability_proof_event(
            self,
            SourceCapabilityProofEvent::Symbol {
                symbol,
                span,
                selector: None,
                scope,
            },
        );
    }

    fn observe_struct_definition(&mut self, def: &StructDef) {
        dispatch_source_capability_proof_event(
            self,
            SourceCapabilityProofEvent::StructDefinition { def },
        );
    }

    fn observe_call_head_symbol(
        &mut self,
        symbol: &str,
        span: Span,
        selector: Option<&str>,
        scope: &SourceCapabilityScope,
    ) {
        dispatch_source_capability_proof_event(
            self,
            SourceCapabilityProofEvent::Symbol {
                symbol,
                span,
                selector,
                scope,
            },
        );
    }

    fn observe_explicit_constructor_symbol(
        &mut self,
        symbol: &str,
        span: Span,
        scope: &SourceCapabilityScope,
    ) {
        dispatch_source_capability_proof_event(
            self,
            SourceCapabilityProofEvent::ExplicitConstructor {
                symbol,
                span,
                scope,
            },
        );
    }

    fn observe_intrinsic(
        &mut self,
        name: &str,
        args: &[PrefixExpr],
        span: Span,
        _scope: &SourceCapabilityScope,
    ) {
        dispatch_source_capability_proof_event(
            self,
            SourceCapabilityProofEvent::Intrinsic { name, args, span },
        );
    }

    fn observe_raw_body(&mut self, body: HirBody, span: Span) {
        dispatch_source_capability_proof_event(
            self,
            SourceCapabilityProofEvent::RawBody { body, span },
        );
    }
}
