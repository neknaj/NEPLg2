use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{
    FnBody, FnDef, Module, PrefixExpr, PrefixItem, Stmt, StructDef, Symbol, TypeExpr, Visibility,
};
use crate::effects::{RawBodyMemoryOp, RawMemoryOp};
use crate::hir::HirBody;
use crate::qualified_name::member_tail;
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
        CollectionSlotLifecycleSurfaceAnalysis::public_exposed_lifecycle_functions(module);
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
    public_raw_pointer_roots: BTreeSet<String>,
    direct_lifecycle_functions: BTreeSet<String>,
    ordinary_call_edges: BTreeMap<String, BTreeSet<String>>,
    public_surface_edges: BTreeMap<String, BTreeSet<String>>,
    function_stack: Vec<String>,
}

impl CollectionSlotLifecycleSurfaceAnalysis {
    fn public_exposed_lifecycle_functions(module: &Module) -> BTreeSet<String> {
        let mut analysis = Self::default();
        analysis.collect_public_surface_edges(module);
        walk_module_capability_evidence(module, &mut analysis);
        analysis.direct_lifecycle_functions_exposed_from_public_roots()
    }

    fn collect_public_surface_edges(&mut self, module: &Module) {
        let top_level_callables = top_level_callables(module);
        for stmt in &module.root.items {
            self.collect_public_surface_edges_from_stmt(stmt, &top_level_callables);
        }
    }

    fn collect_public_surface_edges_from_stmt(
        &mut self,
        stmt: &Stmt,
        top_level_callables: &BTreeSet<String>,
    ) {
        match stmt {
            Stmt::FnDef(def) => self.collect_public_surface_edges_from_function(
                def,
                def.vis == Visibility::Pub,
                top_level_callables,
            ),
            Stmt::FnAlias(alias) => {
                self.public_surface_edges
                    .entry(alias.name.name.clone())
                    .or_default()
                    .insert(alias.target.name.clone());
                if alias.vis == Visibility::Pub {
                    self.public_roots.insert(alias.name.name.clone());
                }
            }
            Stmt::Impl(def) => {
                for method in &def.methods {
                    self.collect_public_surface_edges_from_function(
                        method,
                        method.vis == Visibility::Pub,
                        top_level_callables,
                    );
                }
            }
            Stmt::Directive(_)
            | Stmt::StructDef(_)
            | Stmt::EnumDef(_)
            | Stmt::Wasm(_)
            | Stmt::LlvmIr(_)
            | Stmt::Trait(_)
            | Stmt::Expr(_)
            | Stmt::ExprSemi(_, _) => {}
        }
    }

    fn collect_public_surface_edges_from_function(
        &mut self,
        def: &FnDef,
        exported: bool,
        top_level_callables: &BTreeSet<String>,
    ) {
        self.public_surface_edges
            .entry(def.name.name.clone())
            .or_default();
        if exported {
            self.public_roots.insert(def.name.name.clone());
            if type_expr_contains_mem_ptr(&def.signature) {
                self.public_raw_pointer_roots.insert(def.name.name.clone());
            }
        }
        if let Some(target) = transparent_forward_target(def, top_level_callables) {
            self.public_surface_edges
                .entry(def.name.name.clone())
                .or_default()
                .insert(target);
        }
    }

    fn direct_lifecycle_functions_exposed_from_public_roots(&self) -> BTreeSet<String> {
        let mut reachable =
            self.reachable_from_roots(&self.public_roots, &self.public_surface_edges);
        reachable.extend(
            self.reachable_from_roots(&self.public_raw_pointer_roots, &self.ordinary_call_edges),
        );
        self.direct_lifecycle_functions
            .iter()
            .filter(|name| reachable.contains(*name))
            .cloned()
            .collect()
    }

    fn reachable_from_roots(
        &self,
        roots: &BTreeSet<String>,
        edges: &BTreeMap<String, BTreeSet<String>>,
    ) -> BTreeSet<String> {
        let mut reachable = BTreeSet::new();
        let mut stack: Vec<String> = roots.iter().cloned().collect();
        while let Some(name) = stack.pop() {
            if !reachable.insert(name.clone()) {
                continue;
            }
            if let Some(callees) = edges.get(&name) {
                stack.extend(callees.iter().cloned());
            }
        }
        reachable
    }

    fn current_function_name(&self) -> Option<&str> {
        self.function_stack.last().map(String::as_str)
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
        self.ordinary_call_edges
            .entry(String::from(name))
            .or_default();
        self.public_surface_edges
            .entry(String::from(name))
            .or_default();
        self.function_stack.push(String::from(name));
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
            if let Some(current) = self.current_function_name() {
                self.ordinary_call_edges
                    .entry(String::from(current))
                    .or_default()
                    .insert(String::from(symbol));
            }
        }
    }

    fn observe_named_function_end(
        &mut self,
        _name: &str,
        _span: Span,
        _scope: &SourceCapabilityScope,
    ) {
        self.function_stack.pop();
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

fn top_level_callables(module: &Module) -> BTreeSet<String> {
    module
        .root
        .items
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::FnDef(def) => Some(def.name.name.clone()),
            Stmt::FnAlias(alias) => Some(alias.name.name.clone()),
            Stmt::Directive(_)
            | Stmt::StructDef(_)
            | Stmt::EnumDef(_)
            | Stmt::Wasm(_)
            | Stmt::LlvmIr(_)
            | Stmt::Trait(_)
            | Stmt::Impl(_)
            | Stmt::Expr(_)
            | Stmt::ExprSemi(_, _) => None,
        })
        .collect()
}

fn transparent_forward_target(
    def: &FnDef,
    top_level_callables: &BTreeSet<String>,
) -> Option<String> {
    let expr = single_body_expr(def)?;
    let mut items = expr.items.iter();
    let target = plain_identifier(items.next()?)?;
    if !top_level_callables.contains(target) {
        return None;
    }
    let mut forwarded_params = items;
    for param in &def.params {
        if plain_identifier(forwarded_params.next()?)? != param.name {
            return None;
        }
    }
    if forwarded_params.next().is_some() {
        return None;
    }
    Some(String::from(target))
}

fn single_body_expr(def: &FnDef) -> Option<&PrefixExpr> {
    let FnBody::Parsed(block) = &def.body else {
        return None;
    };
    let [stmt] = block.items.as_slice() else {
        return None;
    };
    match stmt {
        Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => Some(expr),
        Stmt::Directive(_)
        | Stmt::FnDef(_)
        | Stmt::FnAlias(_)
        | Stmt::StructDef(_)
        | Stmt::EnumDef(_)
        | Stmt::Wasm(_)
        | Stmt::LlvmIr(_)
        | Stmt::Trait(_)
        | Stmt::Impl(_) => None,
    }
}

fn plain_identifier(item: &PrefixItem) -> Option<&str> {
    let PrefixItem::Symbol(Symbol::Ident(ident, type_args, forced_value)) = item else {
        return None;
    };
    if *forced_value || !type_args.is_empty() {
        return None;
    }
    Some(ident.name.as_str())
}

fn type_expr_contains_mem_ptr(ty: &TypeExpr) -> bool {
    match ty.as_unspanned() {
        TypeExpr::Named(name) => member_tail(name) == "MemPtr",
        TypeExpr::Apply(base, args) => {
            type_expr_contains_mem_ptr(base) || args.iter().any(type_expr_contains_mem_ptr)
        }
        TypeExpr::Boxed(inner) | TypeExpr::Reference(inner, _) => type_expr_contains_mem_ptr(inner),
        TypeExpr::Tuple(items) => items.iter().any(type_expr_contains_mem_ptr),
        TypeExpr::Function { params, result, .. } => {
            params.iter().any(type_expr_contains_mem_ptr) || type_expr_contains_mem_ptr(result)
        }
        TypeExpr::Unit
        | TypeExpr::I32
        | TypeExpr::U8
        | TypeExpr::F32
        | TypeExpr::Bool
        | TypeExpr::Char
        | TypeExpr::Never
        | TypeExpr::Str
        | TypeExpr::Label(_) => false,
        TypeExpr::Spanned(_, _) => unreachable!("as_unspanned removes nested spans"),
    }
}
