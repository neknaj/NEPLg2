use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

mod context;
mod evidence;
mod field_imports;
use crate::ast::Module;
use crate::source_capability::scope::SourceCapabilityScope;
use crate::source_capability::walk::{walk_module_capability_evidence, SourceCapabilityObserver};

use self::context::OwnerAggregateEvidenceContext;
use self::evidence::{
    owner_aggregate_intrinsic_evidence, owner_aggregate_symbol_evidence,
    OwnerAggregateCapabilityEvidence,
};

#[derive(Debug, Default)]
struct OwnerAggregateEvidence {
    constructors: BTreeSet<String>,
    field_accessor: bool,
}

pub(crate) fn module_owner_aggregate_constructor_evidence(module: &Module) -> Vec<String> {
    let evidence = collect_module_owner_aggregate_evidence(module);
    evidence.constructors.into_iter().collect()
}

pub(crate) fn module_has_owner_aggregate_field_evidence(module: &Module) -> bool {
    collect_module_owner_aggregate_evidence(module).field_accessor
}

fn collect_module_owner_aggregate_evidence(module: &Module) -> OwnerAggregateEvidence {
    let context = OwnerAggregateEvidenceContext::from_module(module);
    let mut collector = OwnerAggregateCollector {
        context: &context,
        evidence: OwnerAggregateEvidence::default(),
    };
    walk_module_capability_evidence(module, &mut collector);
    collector.evidence
}

struct OwnerAggregateCollector<'a> {
    context: &'a OwnerAggregateEvidenceContext,
    evidence: OwnerAggregateEvidence,
}

impl OwnerAggregateCollector<'_> {
    fn record_evidence(&mut self, observed: Option<OwnerAggregateCapabilityEvidence>) {
        match observed {
            Some(OwnerAggregateCapabilityEvidence::FieldAccessor) => {
                self.evidence.field_accessor = true
            }
            Some(OwnerAggregateCapabilityEvidence::Constructor(name)) => {
                self.evidence.constructors.insert(name);
            }
            None => {}
        }
    }
}

impl SourceCapabilityObserver for OwnerAggregateCollector<'_> {
    fn observe_fn_alias_target(&mut self, symbol: &str, scope: &SourceCapabilityScope) {
        self.record_evidence(owner_aggregate_symbol_evidence(symbol, scope, self.context));
    }

    fn observe_call_head_symbol(&mut self, symbol: &str, scope: &SourceCapabilityScope) {
        self.record_evidence(owner_aggregate_symbol_evidence(symbol, scope, self.context));
    }

    fn observe_intrinsic(&mut self, name: &str, _scope: &SourceCapabilityScope) {
        self.record_evidence(owner_aggregate_intrinsic_evidence(name));
    }
}
