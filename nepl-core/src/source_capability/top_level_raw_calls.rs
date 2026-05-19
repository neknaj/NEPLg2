use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::effects::RawMemoryOp;
use crate::source_capability::raw_operation_compat::raw_memory_operation_set_supports_boundary;
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
        if let Some(operation) = frame.boundary_contract.operation() {
            if !frame.evidence.supports_operation(operation) {
                continue;
            }
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
            let proven_raw_calls = proven_top_level_raw_calls(frame, &proven_functions);
            if proven_raw_calls.is_empty() {
                continue;
            }

            for call in &proven_raw_calls {
                evidence.push(PropagatedRawOperationEvidence {
                    operation: call.operation,
                    span: call.span,
                });
            }

            if let Some(operation) = frame.boundary_contract.operation() {
                if !proven_raw_calls_support_boundary(&proven_raw_calls, operation) {
                    continue;
                }
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

fn proven_top_level_raw_calls<'a>(
    frame: &'a RawOperationFunctionProof,
    proven_functions: &BTreeMap<String, BTreeSet<RawMemoryOp>>,
) -> Vec<&'a TopLevelRawCallSite> {
    frame
        .top_level_raw_calls
        .iter()
        .filter(|call| {
            proven_functions
                .get(&call.target)
                .is_some_and(|operations| operations.contains(&call.operation))
        })
        .collect()
}

fn proven_raw_calls_support_boundary(
    proven_raw_calls: &[&TopLevelRawCallSite],
    operation: RawMemoryOp,
) -> bool {
    raw_memory_operation_set_supports_boundary(
        proven_raw_calls.iter().map(|call| call.operation),
        operation,
    )
}
