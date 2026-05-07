extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;

use super::coverage::{ResourceCoverageCounts, ResourceCoverageDiagnostic};
use super::model::{
    Place, PlaceProjection, PlaceRoot, ResourceBlock, ResourceFunction, ResourceOp,
    ResourceTerminator,
};

pub(super) fn resource_function_coverage(
    function: &str,
    resource_function: &ResourceFunction,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) -> ResourceCoverageCounts {
    let mut counts = ResourceCoverageCounts::default();
    for block in &resource_function.blocks {
        resource_block_coverage(function, block, &mut counts, diagnostics);
    }
    counts
}

fn resource_block_coverage(
    function: &str,
    block: &ResourceBlock,
    counts: &mut ResourceCoverageCounts,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) {
    resource_ops_coverage(function, &block.ops, counts, diagnostics);
    if let ResourceTerminator::Return {
        value: Some(place),
        span,
    } = &block.terminator
    {
        resource_place_coverage(function, "return", place, *span, counts, diagnostics);
    }
}

fn resource_ops_coverage(
    function: &str,
    ops: &[ResourceOp],
    counts: &mut ResourceCoverageCounts,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) {
    for op in ops {
        match op {
            ResourceOp::FunctionValue { output, span, .. } => {
                counts.function_values += 1;
                resource_place_coverage(
                    function,
                    "function_value.output",
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::Call {
                output, args, span, ..
            } => {
                counts.direct_calls += 1;
                resource_place_coverage(
                    function,
                    "call.output",
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
                for arg in args {
                    resource_place_coverage(function, "call.arg", arg, *span, counts, diagnostics);
                }
            }
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                span,
                ..
            } => {
                counts.indirect_calls += 1;
                resource_place_coverage(
                    function,
                    "indirect_call.output",
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_place_coverage(
                    function,
                    "indirect_call.callee",
                    callee,
                    *span,
                    counts,
                    diagnostics,
                );
                for arg in args {
                    resource_place_coverage(
                        function,
                        "indirect_call.arg",
                        arg,
                        *span,
                        counts,
                        diagnostics,
                    );
                }
            }
            ResourceOp::RawMemory {
                output, args, span, ..
            } => {
                counts.raw_memory_ops += 1;
                resource_place_coverage(
                    function,
                    "raw_memory.output",
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
                for arg in args {
                    resource_place_coverage(
                        function,
                        "raw_memory.arg",
                        arg,
                        *span,
                        counts,
                        diagnostics,
                    );
                }
            }
            ResourceOp::Branch {
                output,
                condition,
                condition_fact: _,
                then_ops,
                then_value,
                else_ops,
                else_value,
                span,
            } => {
                resource_place_coverage(
                    function,
                    "branch.output",
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_place_coverage(
                    function,
                    "branch.condition",
                    condition,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_ops_coverage(function, then_ops, counts, diagnostics);
                resource_place_coverage(
                    function,
                    "branch.then_value",
                    then_value,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_ops_coverage(function, else_ops, counts, diagnostics);
                resource_place_coverage(
                    function,
                    "branch.else_value",
                    else_value,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::Loop {
                condition_ops,
                condition,
                condition_fact: _,
                body_ops,
                span,
            } => {
                resource_ops_coverage(function, condition_ops, counts, diagnostics);
                resource_place_coverage(
                    function,
                    "loop.condition",
                    condition,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_ops_coverage(function, body_ops, counts, diagnostics);
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                span,
            } => {
                resource_place_coverage(
                    function,
                    "match.output",
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_place_coverage(
                    function,
                    "match.scrutinee",
                    scrutinee,
                    *span,
                    counts,
                    diagnostics,
                );
                for arm in arms {
                    if let Some(bind_local) = &arm.bind_local {
                        resource_place_coverage(
                            function,
                            "match.bind_local",
                            bind_local,
                            *span,
                            counts,
                            diagnostics,
                        );
                    }
                    resource_ops_coverage(function, &arm.ops, counts, diagnostics);
                    resource_place_coverage(
                        function,
                        "match.arm_value",
                        &arm.value,
                        *span,
                        counts,
                        diagnostics,
                    );
                }
            }
            ResourceOp::Expr { output, span, .. } => {
                resource_place_coverage(
                    function,
                    "expr.output",
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::DeclareLocal {
                place,
                initializer,
                span,
                ..
            } => {
                counts.declares += 1;
                resource_place_coverage(
                    function,
                    "declare.place",
                    place,
                    *span,
                    counts,
                    diagnostics,
                );
                if let Some(initializer) = initializer {
                    resource_place_coverage(
                        function,
                        "declare.initializer",
                        initializer,
                        *span,
                        counts,
                        diagnostics,
                    );
                }
            }
            ResourceOp::Read {
                source,
                output,
                span,
            } => {
                counts.reads += 1;
                resource_place_coverage(
                    function,
                    "read.source",
                    source,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_place_coverage(
                    function,
                    "read.output",
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::Move {
                source,
                output,
                span,
            } => {
                counts.moves += 1;
                resource_place_coverage(
                    function,
                    "move.source",
                    source,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_place_coverage(
                    function,
                    "move.output",
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::Assign {
                target,
                value,
                span,
            } => {
                counts.assigns += 1;
                resource_place_coverage(
                    function,
                    "assign.target",
                    target,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_place_coverage(
                    function,
                    "assign.value",
                    value,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::Borrow {
                source,
                output,
                span,
                ..
            } => {
                counts.borrows += 1;
                resource_place_coverage(
                    function,
                    "borrow.source",
                    source,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_place_coverage(
                    function,
                    "borrow.output",
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::Drop { place, span } => {
                counts.drops += 1;
                resource_place_coverage(function, "drop.place", place, *span, counts, diagnostics);
            }
            ResourceOp::Construct {
                output,
                inputs,
                span,
                ..
            } => {
                counts.constructs += 1;
                resource_place_coverage(
                    function,
                    "construct.output",
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
                for input in inputs {
                    resource_place_coverage(
                        function,
                        "construct.input",
                        input,
                        *span,
                        counts,
                        diagnostics,
                    );
                }
            }
            ResourceOp::RawAddressAlias {
                source,
                target,
                span,
            } => {
                resource_alias_place_coverage(
                    function,
                    "raw_address_alias.source",
                    source,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_alias_place_coverage(
                    function,
                    "raw_address_alias.target",
                    target,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::RawAddressView {
                source,
                target,
                kind: _,
                span,
            } => {
                resource_place_coverage(
                    function,
                    "raw_address_view.source",
                    source,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_alias_place_coverage(
                    function,
                    "raw_address_view.target",
                    target,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::StorageOrigin { target, span, .. } => {
                resource_alias_place_coverage(
                    function,
                    "storage_origin.target",
                    target,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::CallEffect { .. } | ResourceOp::EndScope { .. } => {}
        }
    }
}

fn resource_alias_place_coverage(
    function: &str,
    operation: &str,
    place: &Place,
    span: Span,
    counts: &mut ResourceCoverageCounts,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) {
    if matches!(place.root, PlaceRoot::Unknown) {
        counts.unknown_places += 1;
        diagnostics.push(ResourceCoverageDiagnostic::UnknownPlace {
            function: String::from(function),
            operation: String::from(operation),
            place: place.clone(),
            span,
        });
    }
}

fn resource_place_coverage(
    function: &str,
    operation: &str,
    place: &Place,
    span: Span,
    counts: &mut ResourceCoverageCounts,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) {
    counts.deref_projections += place
        .projections
        .iter()
        .filter(|projection| matches!(projection, PlaceProjection::Deref))
        .count();
    if matches!(place.root, PlaceRoot::Unknown) {
        counts.unknown_places += 1;
        diagnostics.push(ResourceCoverageDiagnostic::UnknownPlace {
            function: String::from(function),
            operation: String::from(operation),
            place: place.clone(),
            span,
        });
    }
}
