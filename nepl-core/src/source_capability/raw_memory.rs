use alloc::vec::Vec;

use crate::ast::Module;
use crate::effects::{
    raw_body_direct_callees, raw_body_memory_operations, raw_memory_op_from_name, RawBodyMemoryOp,
    RawMemoryOp,
};
use crate::hir::HirBody;
use crate::source_capability::scope::SourceCapabilityScope;
use crate::source_capability::walk::{walk_module_capability_evidence, SourceCapabilityObserver};

mod evidence;

use evidence::{RawMemoryBoundaryEvidence, RawMemoryEvidence};

pub(crate) fn module_has_raw_memory_boundary_evidence(module: &Module) -> bool {
    collect_module_raw_memory_evidence(module).structural_boundary
}

pub(crate) fn module_raw_memory_operation_evidence(module: &Module) -> Vec<RawMemoryOp> {
    collect_module_raw_memory_evidence(module)
        .operations
        .into_iter()
        .collect()
}

pub(crate) fn module_raw_body_memory_operation_evidence(module: &Module) -> Vec<RawBodyMemoryOp> {
    collect_module_raw_memory_evidence(module)
        .raw_body_operations
        .into_iter()
        .collect()
}

fn collect_module_raw_memory_evidence(module: &Module) -> RawMemoryEvidence {
    let mut collector = RawMemoryCollector::default();
    walk_module_capability_evidence(module, &mut collector);
    collector.evidence
}

#[derive(Default)]
struct RawMemoryCollector {
    evidence: RawMemoryEvidence,
    function_has_evidence: Vec<bool>,
}

impl RawMemoryCollector {
    fn record_evidence(&mut self) {
        for frame in &mut self.function_has_evidence {
            *frame = true;
        }
    }

    fn collect_symbol_evidence(&mut self, symbol: &str, scope: &SourceCapabilityScope) {
        if scope.shadows(symbol) {
            return;
        }
        self.collect_builtin_evidence(symbol);
    }

    fn collect_builtin_evidence(&mut self, symbol: &str) {
        let mut observed = false;
        if RawMemoryBoundaryEvidence::from_symbol(symbol).is_some() {
            self.evidence.structural_boundary = true;
            observed = true;
        }
        if let Some(operation) = raw_memory_op_from_name(symbol) {
            self.evidence.operations.insert(operation);
            observed = true;
        }
        if observed {
            self.record_evidence();
        }
    }

    fn collect_raw_body_evidence(&mut self, body: HirBody) {
        for operation in raw_body_memory_operations(&body) {
            self.evidence.raw_body_operations.insert(operation);
            self.record_evidence();
        }
        for callee in raw_body_direct_callees(&body) {
            if let Some(operation) = raw_memory_op_from_name(&callee) {
                self.evidence.operations.insert(operation);
                self.record_evidence();
            }
        }
    }
}

impl SourceCapabilityObserver for RawMemoryCollector {
    fn observe_named_function_start(&mut self, _name: &str, _scope: &SourceCapabilityScope) {
        self.function_has_evidence.push(false);
    }

    fn observe_named_function_end(&mut self, name: &str, _scope: &SourceCapabilityScope) {
        let has_body_evidence = self.function_has_evidence.pop().unwrap_or(false);
        if has_body_evidence {
            if let Some(operation) = raw_memory_op_from_name(name) {
                self.evidence.operations.insert(operation);
            }
        }
    }

    fn observe_fn_alias_target(&mut self, symbol: &str, scope: &SourceCapabilityScope) {
        self.collect_symbol_evidence(symbol, scope);
    }

    fn observe_call_head_symbol(&mut self, symbol: &str, scope: &SourceCapabilityScope) {
        self.collect_symbol_evidence(symbol, scope);
    }

    fn observe_intrinsic(&mut self, name: &str, _scope: &SourceCapabilityScope) {
        self.collect_builtin_evidence(name);
    }

    fn observe_raw_body(&mut self, body: HirBody) {
        self.collect_raw_body_evidence(body);
    }
}
