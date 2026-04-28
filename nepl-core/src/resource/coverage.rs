extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::effects::{intrinsic_is_raw_memory_effect, raw_callee_is_raw_memory_effect};
use crate::hir::{FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirModule};
use crate::span::Span;

use super::model::{Place, PlaceProjection, PlaceRoot, ResourceBlock, ResourceModule, ResourceOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLoweringCoverage {
    pub functions: Vec<ResourceFunctionCoverage>,
    pub diagnostics: Vec<ResourceCoverageDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceFunctionCoverage {
    pub name: String,
    pub hir: ResourceCoverageCounts,
    pub resource: ResourceCoverageCounts,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceCoverageCounts {
    pub direct_calls: usize,
    pub indirect_calls: usize,
    pub function_values: usize,
    pub raw_memory_ops: usize,
    pub constructs: usize,
    pub declares: usize,
    pub reads: usize,
    pub moves: usize,
    pub assigns: usize,
    pub borrows: usize,
    pub drops: usize,
    pub deref_projections: usize,
    pub unknown_places: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceCoverageDiagnostic {
    MissingFunction {
        name: String,
        span: Span,
    },
    CountMismatch {
        function: String,
        kind: ResourceCoverageKind,
        hir: usize,
        resource: usize,
        span: Span,
    },
    UnknownPlace {
        function: String,
        operation: String,
        place: Place,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceCoverageKind {
    DirectCall,
    IndirectCall,
    FunctionValue,
    RawMemory,
    Construct,
    Declare,
    Read,
    Move,
    Assign,
    Borrow,
    Drop,
    DerefProjection,
    UnknownPlace,
}

pub fn compare_hir_resource_lowering(
    module: &HirModule,
    resource: &ResourceModule,
) -> ResourceLoweringCoverage {
    let mut resource_functions = BTreeMap::new();
    for function in &resource.functions {
        resource_functions.insert(function.name.as_str(), function);
    }

    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    for function in &module.functions {
        let hir = hir_body_coverage(&function.body);
        let Some(resource_function) = resource_functions.get(function.name.as_str()) else {
            diagnostics.push(ResourceCoverageDiagnostic::MissingFunction {
                name: function.name.clone(),
                span: function.span,
            });
            continue;
        };
        let mut resource_counts = ResourceCoverageCounts::default();
        for block in &resource_function.blocks {
            resource_block_coverage(
                &function.name,
                block,
                &mut resource_counts,
                &mut diagnostics,
            );
        }
        push_count_diagnostics(
            &function.name,
            function.span,
            &hir,
            &resource_counts,
            &mut diagnostics,
        );
        functions.push(ResourceFunctionCoverage {
            name: function.name.clone(),
            hir,
            resource: resource_counts,
        });
    }

    ResourceLoweringCoverage {
        functions,
        diagnostics,
    }
}

fn hir_body_coverage(body: &HirBody) -> ResourceCoverageCounts {
    let mut counts = ResourceCoverageCounts::default();
    if let HirBody::Block(block) = body {
        hir_block_coverage(block, &mut counts);
    }
    counts
}

fn hir_block_coverage(block: &HirBlock, counts: &mut ResourceCoverageCounts) {
    for line in &block.lines {
        hir_expr_coverage(&line.expr, counts);
    }
}

fn hir_expr_coverage(expr: &HirExpr, counts: &mut ResourceCoverageCounts) {
    match &expr.kind {
        HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit => {}
        HirExprKind::Var(_) => {
            counts.reads += 1;
        }
        HirExprKind::Drop { .. } => {
            counts.drops += 1;
        }
        HirExprKind::FnValue(_) => {
            counts.function_values += 1;
        }
        HirExprKind::Call { callee, args } => {
            counts.direct_calls += 1;
            if callee_is_raw_memory(callee) {
                counts.raw_memory_ops += 1;
            }
            for arg in args {
                hir_expr_coverage(arg, counts);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            counts.indirect_calls += 1;
            hir_expr_coverage(callee, counts);
            for arg in args {
                hir_expr_coverage(arg, counts);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            hir_expr_coverage(cond, counts);
            hir_expr_coverage(then_branch, counts);
            hir_expr_coverage(else_branch, counts);
        }
        HirExprKind::While { cond, body } => {
            hir_expr_coverage(cond, counts);
            hir_expr_coverage(body, counts);
        }
        HirExprKind::Match { scrutinee, arms } => {
            hir_expr_coverage(scrutinee, counts);
            for arm in arms {
                hir_expr_coverage(&arm.body, counts);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            counts.constructs += 1;
            if let Some(payload) = payload {
                hir_expr_coverage(payload, counts);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            counts.constructs += 1;
            for field in fields {
                hir_expr_coverage(field, counts);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            counts.constructs += 1;
            for item in items {
                hir_expr_coverage(item, counts);
            }
        }
        HirExprKind::Block(block) => hir_block_coverage(block, counts),
        HirExprKind::Let { value, .. } => {
            counts.declares += 1;
            hir_expr_coverage(value, counts);
        }
        HirExprKind::Set { value, .. } => {
            counts.assigns += 1;
            hir_expr_coverage(value, counts);
        }
        HirExprKind::Intrinsic { name, args, .. } => {
            if intrinsic_is_raw_memory_effect(name) {
                counts.raw_memory_ops += 1;
            }
            for arg in args {
                hir_expr_coverage(arg, counts);
            }
        }
        HirExprKind::AddrOf(inner) => {
            counts.borrows += 1;
            hir_place_expr_coverage(inner, counts);
        }
        HirExprKind::Deref(inner) => {
            counts.reads += 1;
            counts.deref_projections += 1;
            hir_place_expr_coverage(inner, counts);
        }
    }
}

fn hir_place_expr_coverage(expr: &HirExpr, counts: &mut ResourceCoverageCounts) {
    match &expr.kind {
        HirExprKind::Var(_) => {}
        HirExprKind::Deref(inner) => {
            counts.deref_projections += 1;
            hir_place_expr_coverage(inner, counts);
        }
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && !args.is_empty() => {
            hir_place_expr_coverage(&args[0], counts);
            for arg in args.iter().skip(1) {
                hir_expr_coverage(arg, counts);
            }
        }
        _ => hir_expr_coverage(expr, counts),
    }
}

fn callee_is_raw_memory(callee: &FuncRef) -> bool {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => raw_callee_is_raw_memory_effect(name),
        FuncRef::Trait { .. } => false,
    }
}

fn resource_block_coverage(
    function: &str,
    block: &ResourceBlock,
    counts: &mut ResourceCoverageCounts,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) {
    resource_ops_coverage(function, &block.ops, counts, diagnostics);
    if let super::model::ResourceTerminator::Return {
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
            ResourceOp::CallEffect { .. } => {}
        }
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

fn push_count_diagnostics(
    function: &str,
    span: Span,
    hir: &ResourceCoverageCounts,
    resource: &ResourceCoverageCounts,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) {
    push_count_diagnostic(
        function,
        ResourceCoverageKind::DirectCall,
        hir.direct_calls,
        resource.direct_calls,
        span,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::IndirectCall,
        hir.indirect_calls,
        resource.indirect_calls,
        span,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::FunctionValue,
        hir.function_values,
        resource.function_values,
        span,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::RawMemory,
        hir.raw_memory_ops,
        resource.raw_memory_ops,
        span,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::Construct,
        hir.constructs,
        resource.constructs,
        span,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::Declare,
        hir.declares,
        resource.declares,
        span,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::Read,
        hir.reads,
        resource.reads,
        span,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::Move,
        hir.moves,
        resource.moves,
        span,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::Assign,
        hir.assigns,
        resource.assigns,
        span,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::Borrow,
        hir.borrows,
        resource.borrows,
        span,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::Drop,
        hir.drops,
        resource.drops,
        span,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::DerefProjection,
        hir.deref_projections,
        resource.deref_projections,
        span,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::UnknownPlace,
        hir.unknown_places,
        resource.unknown_places,
        span,
        diagnostics,
    );
}

fn push_count_diagnostic(
    function: &str,
    kind: ResourceCoverageKind,
    hir: usize,
    resource: usize,
    span: Span,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) {
    if hir != resource {
        diagnostics.push(ResourceCoverageDiagnostic::CountMismatch {
            function: String::from(function),
            kind,
            hir,
            resource,
            span,
        });
    }
}
