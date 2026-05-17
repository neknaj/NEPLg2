use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::effects::RawMemoryOp;
use crate::source_capability::raw_operation_proof::{
    RawOperationBoundaryContract, RawOperationFunctionEvidence,
};
use crate::span::Span;

#[derive(Debug, Clone)]
pub(in crate::source_capability) struct TopLevelRawCallSite {
    pub(in crate::source_capability) target: String,
    pub(in crate::source_capability) operation: RawMemoryOp,
    pub(in crate::source_capability) span: Span,
}

#[derive(Debug)]
pub(in crate::source_capability) struct RawOperationFunctionProof {
    pub(in crate::source_capability) name: String,
    pub(in crate::source_capability) span: Span,
    pub(in crate::source_capability) boundary_contract: RawOperationBoundaryContract,
    pub(in crate::source_capability) evidence: RawOperationFunctionEvidence,
    pub(in crate::source_capability) top_level_raw_calls: Vec<TopLevelRawCallSite>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::source_capability) struct PropagatedRawOperationEvidence {
    pub(in crate::source_capability) operation: RawMemoryOp,
    pub(in crate::source_capability) span: Span,
}

pub(in crate::source_capability) fn collect_top_level_raw_call_evidence(
    frames: &[RawOperationFunctionProof],
) -> Vec<PropagatedRawOperationEvidence> {
    let mut proven_functions: BTreeMap<String, BTreeSet<RawMemoryOp>> = BTreeMap::new();
    let mut evidence = Vec::new();

    for frame in frames {
        if !frame.evidence.has_direct_raw_evidence() {
            continue;
        }
        if let Some(operation) = frame.boundary_contract.operation() {
            proven_functions
                .entry(frame.name.clone())
                .or_default()
                .insert(operation);
            evidence.push(PropagatedRawOperationEvidence {
                operation,
                span: frame.span,
            });
        }
    }

    loop {
        let mut changed = false;
        for frame in frames {
            let frame_has_proven_raw_call = frame.top_level_raw_calls.iter().any(|call| {
                proven_functions
                    .get(&call.target)
                    .is_some_and(|operations| operations.contains(&call.operation))
            });
            if !frame_has_proven_raw_call {
                continue;
            }

            for call in &frame.top_level_raw_calls {
                if proven_functions
                    .get(&call.target)
                    .is_some_and(|operations| operations.contains(&call.operation))
                {
                    evidence.push(PropagatedRawOperationEvidence {
                        operation: call.operation,
                        span: call.span,
                    });
                }
            }

            if let Some(operation) = frame.boundary_contract.operation() {
                let operations = proven_functions.entry(frame.name.clone()).or_default();
                if operations.insert(operation) {
                    evidence.push(PropagatedRawOperationEvidence {
                        operation,
                        span: frame.span,
                    });
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    evidence
}
