extern crate alloc;

use alloc::vec::Vec;

use super::coverage::{ResourceCoverageCounts, ResourceCoverageDiagnostic};
use super::coverage_operation::ResourceCoveragePlaceOperation as CoveragePlaceOp;
use super::coverage_resource_place::{resource_alias_place_coverage, resource_place_coverage};
use super::model::{ResourceBlock, ResourceFunction, ResourceOp, ResourceTerminator};

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
        resource_place_coverage(
            function,
            CoveragePlaceOp::ReturnValue,
            place,
            *span,
            counts,
            diagnostics,
        );
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
                    CoveragePlaceOp::FunctionValueOutput,
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
                    CoveragePlaceOp::CallOutput,
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
                for arg in args {
                    resource_place_coverage(
                        function,
                        CoveragePlaceOp::CallArgument,
                        arg,
                        *span,
                        counts,
                        diagnostics,
                    );
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
                    CoveragePlaceOp::IndirectCallOutput,
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_place_coverage(
                    function,
                    CoveragePlaceOp::IndirectCallCallee,
                    callee,
                    *span,
                    counts,
                    diagnostics,
                );
                for arg in args {
                    resource_place_coverage(
                        function,
                        CoveragePlaceOp::IndirectCallArgument,
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
                    CoveragePlaceOp::RawMemoryOutput,
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
                for arg in args {
                    resource_place_coverage(
                        function,
                        CoveragePlaceOp::RawMemoryArgument,
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
                    CoveragePlaceOp::BranchOutput,
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_place_coverage(
                    function,
                    CoveragePlaceOp::BranchCondition,
                    condition,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_ops_coverage(function, then_ops, counts, diagnostics);
                resource_place_coverage(
                    function,
                    CoveragePlaceOp::BranchThenValue,
                    then_value,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_ops_coverage(function, else_ops, counts, diagnostics);
                resource_place_coverage(
                    function,
                    CoveragePlaceOp::BranchElseValue,
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
                    CoveragePlaceOp::LoopCondition,
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
                scrutinee_is_borrow_target,
                arms,
                span,
            } => {
                resource_place_coverage(
                    function,
                    CoveragePlaceOp::MatchOutput,
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
                if !*scrutinee_is_borrow_target {
                    resource_place_coverage(
                        function,
                        CoveragePlaceOp::MatchScrutinee,
                        scrutinee,
                        *span,
                        counts,
                        diagnostics,
                    );
                }
                for arm in arms {
                    if let Some(bind_local) = &arm.bind_local {
                        resource_place_coverage(
                            function,
                            CoveragePlaceOp::MatchBindLocal,
                            bind_local,
                            *span,
                            counts,
                            diagnostics,
                        );
                    }
                    resource_ops_coverage(function, &arm.ops, counts, diagnostics);
                    resource_place_coverage(
                        function,
                        CoveragePlaceOp::MatchArmValue,
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
                    CoveragePlaceOp::ExprOutput,
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
                    CoveragePlaceOp::DeclarePlace,
                    place,
                    *span,
                    counts,
                    diagnostics,
                );
                if let Some(initializer) = initializer {
                    resource_place_coverage(
                        function,
                        CoveragePlaceOp::DeclareInitializer,
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
                    CoveragePlaceOp::ReadSource,
                    source,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_place_coverage(
                    function,
                    CoveragePlaceOp::ReadOutput,
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
                    CoveragePlaceOp::MoveSource,
                    source,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_place_coverage(
                    function,
                    CoveragePlaceOp::MoveOutput,
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
                    CoveragePlaceOp::AssignTarget,
                    target,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_place_coverage(
                    function,
                    CoveragePlaceOp::AssignValue,
                    value,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::Borrow {
                source,
                output,
                synthetic,
                span,
                ..
            } => {
                if !*synthetic {
                    counts.borrows += 1;
                    resource_place_coverage(
                        function,
                        CoveragePlaceOp::BorrowSource,
                        source,
                        *span,
                        counts,
                        diagnostics,
                    );
                    resource_place_coverage(
                        function,
                        CoveragePlaceOp::BorrowOutput,
                        output,
                        *span,
                        counts,
                        diagnostics,
                    );
                }
            }
            ResourceOp::Drop { place, span } => {
                counts.drops += 1;
                resource_place_coverage(
                    function,
                    CoveragePlaceOp::DropPlace,
                    place,
                    *span,
                    counts,
                    diagnostics,
                );
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
                    CoveragePlaceOp::ConstructOutput,
                    output,
                    *span,
                    counts,
                    diagnostics,
                );
                for input in inputs {
                    resource_place_coverage(
                        function,
                        CoveragePlaceOp::ConstructInput,
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
                ..
            } => {
                resource_alias_place_coverage(
                    function,
                    CoveragePlaceOp::RawAddressAliasSource,
                    source,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_alias_place_coverage(
                    function,
                    CoveragePlaceOp::RawAddressAliasTarget,
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
                    CoveragePlaceOp::RawAddressViewSource,
                    source,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_alias_place_coverage(
                    function,
                    CoveragePlaceOp::RawAddressViewTarget,
                    target,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::StorageOrigin { target, span, .. } => {
                resource_alias_place_coverage(
                    function,
                    CoveragePlaceOp::StorageOriginTarget,
                    target,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::CollectionSlotLifecycle { target, span, .. } => {
                counts.collection_slot_lifecycle_ops += 1;
                resource_alias_place_coverage(
                    function,
                    CoveragePlaceOp::CollectionSlotLifecycleTarget,
                    target,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::CollectionStorageRelocate {
                old_storage,
                new_storage,
                span,
            } => {
                counts.collection_storage_relocates += 1;
                resource_alias_place_coverage(
                    function,
                    CoveragePlaceOp::CollectionStorageRelocateOld,
                    old_storage,
                    *span,
                    counts,
                    diagnostics,
                );
                resource_alias_place_coverage(
                    function,
                    CoveragePlaceOp::CollectionStorageRelocateNew,
                    new_storage,
                    *span,
                    counts,
                    diagnostics,
                );
            }
            ResourceOp::CallEffect { .. } | ResourceOp::EndScope { .. } => {}
        }
    }
}
