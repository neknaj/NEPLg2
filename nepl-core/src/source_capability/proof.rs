use alloc::collections::BTreeSet;
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
use crate::source_capability::raw_memory::{RawMemoryBoundaryEvidence, RawMemoryEvidence};
use crate::source_capability::scope::SourceCapabilityScope;
use crate::source_capability::walk::{walk_module_capability_evidence, SourceCapabilityObserver};
use crate::source_map::{CompilerMemoryType, SourceCapabilities, SourceCapability};

pub(crate) fn module_source_capabilities(module: &Module) -> SourceCapabilities {
    collect_source_capability_proof(module).into_source_capabilities()
}

#[derive(Debug, Default)]
struct OwnerAggregateProofEvidence {
    constructors: BTreeSet<String>,
    field_accessor: bool,
}

#[derive(Debug, Default)]
struct SourceCapabilityProof {
    raw_memory: RawMemoryEvidence,
    owner_aggregate: OwnerAggregateProofEvidence,
    compiler_memory_types: BTreeSet<CompilerMemoryType>,
}

impl SourceCapabilityProof {
    fn into_source_capabilities(self) -> SourceCapabilities {
        let mut capabilities = SourceCapabilities::none();
        if self.raw_memory.structural_boundary {
            capabilities.insert(SourceCapability::RawMemoryStructuralBoundary);
        }
        for operation in self.raw_memory.operations {
            capabilities.insert(SourceCapability::RawMemoryOperationBoundary(operation));
        }
        for operation in self.raw_memory.raw_body_operations {
            capabilities.insert(SourceCapability::RawBodyMemoryOperationBoundary(operation));
        }
        for name in self.owner_aggregate.constructors {
            capabilities.insert(SourceCapability::OwnerAggregateConstructorBoundary(name));
        }
        if self.owner_aggregate.field_accessor {
            capabilities.insert(SourceCapability::OwnerAggregateFieldBoundary);
        }
        for memory_type in self.compiler_memory_types {
            capabilities.insert(SourceCapability::CompilerMemoryTypeDefinition(memory_type));
        }
        capabilities
    }
}

fn collect_source_capability_proof(module: &Module) -> SourceCapabilityProof {
    let owner_context = OwnerAggregateEvidenceContext::from_module(module);
    let mut collector = SourceCapabilityProofCollector {
        owner_context: &owner_context,
        raw_memory: RawMemoryEvidence::default(),
        owner_aggregate: OwnerAggregateProofEvidence::default(),
        compiler_memory_types: BTreeSet::new(),
        function_has_raw_memory_evidence: Vec::new(),
    };
    walk_module_capability_evidence(module, &mut collector);
    SourceCapabilityProof {
        raw_memory: collector.raw_memory,
        owner_aggregate: collector.owner_aggregate,
        compiler_memory_types: collector.compiler_memory_types,
    }
}

struct SourceCapabilityProofCollector<'a> {
    owner_context: &'a OwnerAggregateEvidenceContext,
    raw_memory: RawMemoryEvidence,
    owner_aggregate: OwnerAggregateProofEvidence,
    compiler_memory_types: BTreeSet<CompilerMemoryType>,
    function_has_raw_memory_evidence: Vec<bool>,
}

impl SourceCapabilityProofCollector<'_> {
    fn record_raw_memory_evidence(&mut self) {
        if let Some(frame) = self.function_has_raw_memory_evidence.last_mut() {
            *frame = true;
        }
    }

    fn collect_symbol_evidence(&mut self, symbol: &str, scope: &SourceCapabilityScope) {
        self.collect_raw_symbol_evidence(symbol, scope);
        self.collect_owner_aggregate_symbol_evidence(symbol, scope);
    }

    fn collect_raw_symbol_evidence(&mut self, symbol: &str, scope: &SourceCapabilityScope) {
        if scope.shadows(symbol) {
            return;
        }
        self.collect_raw_builtin_evidence(symbol);
    }

    fn collect_raw_builtin_evidence(&mut self, symbol: &str) {
        let mut observed = false;
        if RawMemoryBoundaryEvidence::from_symbol(symbol).is_some() {
            self.raw_memory.structural_boundary = true;
            observed = true;
        }
        if let Some(operation) = raw_memory_op_from_name(symbol) {
            self.raw_memory.operations.insert(operation);
            observed = true;
        }
        if observed {
            self.record_raw_memory_evidence();
        }
    }

    fn collect_raw_body_evidence(&mut self, body: HirBody) {
        for operation in raw_body_memory_operations(&body) {
            self.raw_memory.raw_body_operations.insert(operation);
            self.record_raw_memory_evidence();
        }
        for callee in raw_body_direct_callees(&body) {
            if let Some(operation) = raw_memory_op_from_name(&callee) {
                self.raw_memory.operations.insert(operation);
                self.record_raw_memory_evidence();
            }
        }
    }

    fn collect_owner_aggregate_symbol_evidence(
        &mut self,
        symbol: &str,
        scope: &SourceCapabilityScope,
    ) {
        self.record_owner_aggregate_evidence(owner_aggregate_symbol_evidence(
            symbol,
            scope,
            self.owner_context,
        ));
    }

    fn collect_owner_aggregate_explicit_constructor_evidence(
        &mut self,
        symbol: &str,
        scope: &SourceCapabilityScope,
    ) {
        self.record_owner_aggregate_evidence(owner_aggregate_explicit_constructor_evidence(
            symbol,
            scope,
            self.owner_context,
        ));
    }

    fn record_owner_aggregate_evidence(
        &mut self,
        observed: Option<OwnerAggregateCapabilityEvidence>,
    ) {
        match observed {
            Some(OwnerAggregateCapabilityEvidence::FieldAccessor) => {
                self.owner_aggregate.field_accessor = true
            }
            Some(OwnerAggregateCapabilityEvidence::Constructor(name)) => {
                self.owner_aggregate.constructors.insert(name);
            }
            None => {}
        }
    }
}

impl SourceCapabilityObserver for SourceCapabilityProofCollector<'_> {
    fn observe_named_function_start(&mut self, _name: &str, _scope: &SourceCapabilityScope) {
        self.function_has_raw_memory_evidence.push(false);
    }

    fn observe_named_function_end(&mut self, name: &str, _scope: &SourceCapabilityScope) {
        let has_body_evidence = self.function_has_raw_memory_evidence.pop().unwrap_or(false);
        if has_body_evidence {
            if let Some(operation) = raw_memory_op_from_name(name) {
                self.raw_memory.operations.insert(operation);
            }
        }
    }

    fn observe_fn_alias_target(&mut self, symbol: &str, scope: &SourceCapabilityScope) {
        self.collect_symbol_evidence(symbol, scope);
    }

    fn observe_struct_definition(&mut self, def: &StructDef) {
        if let Some(memory_type) = compiler_memory_type_from_struct_def(def) {
            self.compiler_memory_types.insert(memory_type);
        }
    }

    fn observe_call_head_symbol(&mut self, symbol: &str, scope: &SourceCapabilityScope) {
        self.collect_symbol_evidence(symbol, scope);
    }

    fn observe_explicit_constructor_symbol(&mut self, symbol: &str, scope: &SourceCapabilityScope) {
        self.collect_owner_aggregate_explicit_constructor_evidence(symbol, scope);
    }

    fn observe_intrinsic(&mut self, name: &str, _scope: &SourceCapabilityScope) {
        self.collect_raw_builtin_evidence(name);
        self.record_owner_aggregate_evidence(owner_aggregate_intrinsic_evidence(name));
    }

    fn observe_raw_body(&mut self, body: HirBody) {
        self.collect_raw_body_evidence(body);
    }
}
