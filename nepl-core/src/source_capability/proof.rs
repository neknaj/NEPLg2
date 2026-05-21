use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{Module, PrefixExpr, Stmt, StructDef, Visibility};
use crate::effects::{RawBodyMemoryOp, RawMemoryOp};
use crate::hir::HirBody;
use crate::resource_primitives::CollectionSlotLifecyclePrimitive;
use crate::source_capability::binding::SourceCapabilityBindingKind;
use crate::source_capability::owner_aggregate::OwnerAggregateEvidenceContext;
use crate::source_capability::proof_builder::SourceCapabilityProof;
use crate::source_capability::raw_operation_proof::{
    RawOperationBoundaryContract, RawOperationFunctionEvidence,
};
use crate::source_capability::rule::{
    dispatch_source_capability_proof_event, raw_memory_boundary_contract_from_function_name,
    CollectionSlotLifecycleSourceSurface, SourceCapabilityProofEvent, SourceCapabilityProofSink,
};
use crate::source_capability::scope::SourceCapabilityScope;
use crate::source_capability::top_level_raw_calls::{
    collect_top_level_raw_call_evidence, RawOperationFunctionProof, TopLevelRawCallSite,
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
    collection_slot_surface: CollectionSlotLifecycleSourceSurface,
    evidence: RawOperationFunctionEvidence,
    top_level_raw_calls: Vec<TopLevelRawCallSite>,
}

fn collect_source_capability_proof(module: &Module) -> SourceCapabilityProof {
    let owner_context = OwnerAggregateEvidenceContext::from_module(module);
    let lifecycle_public_surface_functions =
        CollectionSlotLifecycleSurfaceAnalysis::public_reachable_lifecycle_functions(module);
    let mut collector = SourceCapabilityProofCollector {
        owner_context: &owner_context,
        lifecycle_public_surface_functions,
        proof: SourceCapabilityProof::default(),
        raw_operation_function_frames: Vec::new(),
        completed_raw_operation_function_frames: Vec::new(),
    };
    walk_module_capability_evidence(module, &mut collector);
    let propagated_raw_operations =
        collect_top_level_raw_call_evidence(&collector.completed_raw_operation_function_frames);
    for evidence in propagated_raw_operations {
        dispatch_source_capability_proof_event(
            &mut collector,
            SourceCapabilityProofEvent::PropagatedRawOperation {
                operation: evidence.operation,
                span: evidence.span,
            },
        );
    }
    collector.proof
}

struct SourceCapabilityProofCollector<'a> {
    owner_context: &'a OwnerAggregateEvidenceContext,
    lifecycle_public_surface_functions: BTreeSet<String>,
    proof: SourceCapabilityProof,
    raw_operation_function_frames: Vec<RawOperationFunctionFrame>,
    completed_raw_operation_function_frames: Vec<RawOperationFunctionProof>,
}

impl SourceCapabilityProofCollector<'_> {
    fn collection_slot_surface_for_function(
        &self,
        name: &str,
        _exported: bool,
    ) -> CollectionSlotLifecycleSourceSurface {
        if self.lifecycle_public_surface_functions.contains(name) {
            CollectionSlotLifecycleSourceSurface::PublicCallableSurface
        } else {
            CollectionSlotLifecycleSourceSurface::InternalCallable
        }
    }

    fn current_collection_slot_surface(&self) -> CollectionSlotLifecycleSourceSurface {
        self.raw_operation_function_frames
            .last()
            .map(|frame| frame.collection_slot_surface)
            .unwrap_or(CollectionSlotLifecycleSourceSurface::InternalCallable)
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
        exported: bool,
        _scope: &SourceCapabilityScope,
    ) {
        let collection_slot_surface = self.collection_slot_surface_for_function(name, exported);
        self.raw_operation_function_frames
            .push(RawOperationFunctionFrame {
                name: String::from(name),
                span,
                boundary_contract: raw_memory_boundary_contract_from_function_name(name),
                collection_slot_surface,
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
        name_span: Span,
        _scope: &SourceCapabilityScope,
    ) {
        dispatch_source_capability_proof_event(
            self,
            SourceCapabilityProofEvent::Intrinsic {
                name,
                args,
                span,
                name_span,
                collection_slot_surface: self.current_collection_slot_surface(),
            },
        );
    }

    fn observe_raw_body(&mut self, body: HirBody, span: Span) {
        dispatch_source_capability_proof_event(
            self,
            SourceCapabilityProofEvent::RawBody { body, span },
        );
    }
}

#[derive(Debug, Default)]
struct CollectionSlotLifecycleSurfaceAnalysis {
    public_roots: BTreeSet<String>,
    direct_lifecycle_functions: BTreeSet<String>,
    call_edges: BTreeMap<String, BTreeSet<String>>,
    function_stack: Vec<String>,
}

impl CollectionSlotLifecycleSurfaceAnalysis {
    fn public_reachable_lifecycle_functions(module: &Module) -> BTreeSet<String> {
        let mut analysis = Self::default();
        analysis.collect_alias_edges(module);
        walk_module_capability_evidence(module, &mut analysis);
        analysis.direct_lifecycle_functions_reachable_from_public_roots()
    }

    fn collect_alias_edges(&mut self, module: &Module) {
        for stmt in &module.root.items {
            let Stmt::FnAlias(alias) = stmt else {
                continue;
            };
            self.call_edges
                .entry(alias.name.name.clone())
                .or_default()
                .insert(alias.target.name.clone());
            if alias.vis == Visibility::Pub {
                self.public_roots.insert(alias.name.name.clone());
            }
        }
    }

    fn direct_lifecycle_functions_reachable_from_public_roots(&self) -> BTreeSet<String> {
        let mut reachable = BTreeSet::new();
        let mut stack: Vec<String> = self.public_roots.iter().cloned().collect();
        while let Some(name) = stack.pop() {
            if !reachable.insert(name.clone()) {
                continue;
            }
            if let Some(callees) = self.call_edges.get(&name) {
                stack.extend(callees.iter().cloned());
            }
        }
        self.direct_lifecycle_functions
            .iter()
            .filter(|name| reachable.contains(*name))
            .cloned()
            .collect()
    }

    fn current_function_name(&self) -> Option<&str> {
        self.function_stack.last().map(String::as_str)
    }

    fn insert_current_call_edge(&mut self, target: &str) {
        let Some(current) = self.function_stack.last().cloned() else {
            return;
        };
        self.call_edges
            .entry(current)
            .or_default()
            .insert(String::from(target));
    }
}

impl SourceCapabilityObserver for CollectionSlotLifecycleSurfaceAnalysis {
    fn observe_named_function_start(
        &mut self,
        name: &str,
        _span: Span,
        exported: bool,
        _scope: &SourceCapabilityScope,
    ) {
        if exported {
            self.public_roots.insert(String::from(name));
        }
        self.call_edges.entry(String::from(name)).or_default();
        self.function_stack.push(String::from(name));
    }

    fn observe_named_function_end(
        &mut self,
        _name: &str,
        _span: Span,
        _scope: &SourceCapabilityScope,
    ) {
        self.function_stack.pop();
    }

    fn observe_call_head_symbol(
        &mut self,
        symbol: &str,
        _span: Span,
        _selector: Option<&str>,
        scope: &SourceCapabilityScope,
    ) {
        if matches!(
            scope.shadow_kind_symbol_or_qualifier(symbol),
            Some(SourceCapabilityBindingKind::TopLevelCallable)
        ) {
            self.insert_current_call_edge(symbol);
        }
    }

    fn observe_intrinsic(
        &mut self,
        name: &str,
        _args: &[PrefixExpr],
        _span: Span,
        _name_span: Span,
        _scope: &SourceCapabilityScope,
    ) {
        if CollectionSlotLifecyclePrimitive::from_intrinsic_name(name).is_some() {
            if let Some(current) = self.current_function_name() {
                self.direct_lifecycle_functions
                    .insert(String::from(current));
            }
        }
    }
}
