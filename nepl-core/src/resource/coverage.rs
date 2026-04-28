extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::effects::{intrinsic_is_raw_memory_effect, raw_callee_is_raw_memory_effect};
use crate::hir::{FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirModule};

use super::model::{ResourceModule, ResourceOp};

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceCoverageDiagnostic {
    MissingFunction {
        name: String,
    },
    CountMismatch {
        function: String,
        kind: ResourceCoverageKind,
        hir: usize,
        resource: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceCoverageKind {
    DirectCall,
    IndirectCall,
    FunctionValue,
    RawMemory,
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
            });
            continue;
        };
        let mut resource_counts = ResourceCoverageCounts::default();
        for block in &resource_function.blocks {
            resource_ops_coverage(&block.ops, &mut resource_counts);
        }
        push_count_diagnostics(&function.name, &hir, &resource_counts, &mut diagnostics);
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
        | HirExprKind::Unit
        | HirExprKind::Var(_)
        | HirExprKind::Drop { .. } => {}
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
            if let Some(payload) = payload {
                hir_expr_coverage(payload, counts);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for field in fields {
                hir_expr_coverage(field, counts);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for item in items {
                hir_expr_coverage(item, counts);
            }
        }
        HirExprKind::Block(block) => hir_block_coverage(block, counts),
        HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
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
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            hir_expr_coverage(inner, counts);
        }
    }
}

fn callee_is_raw_memory(callee: &FuncRef) -> bool {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => raw_callee_is_raw_memory_effect(name),
        FuncRef::Trait { .. } => false,
    }
}

fn resource_ops_coverage(ops: &[ResourceOp], counts: &mut ResourceCoverageCounts) {
    for op in ops {
        match op {
            ResourceOp::FunctionValue { .. } => counts.function_values += 1,
            ResourceOp::Call { .. } => counts.direct_calls += 1,
            ResourceOp::IndirectCall { .. } => counts.indirect_calls += 1,
            ResourceOp::RawMemory { .. } => counts.raw_memory_ops += 1,
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                resource_ops_coverage(then_ops, counts);
                resource_ops_coverage(else_ops, counts);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                resource_ops_coverage(condition_ops, counts);
                resource_ops_coverage(body_ops, counts);
            }
            ResourceOp::Match { arms, .. } => {
                for arm in arms {
                    resource_ops_coverage(&arm.ops, counts);
                }
            }
            ResourceOp::Expr { .. }
            | ResourceOp::DeclareLocal { .. }
            | ResourceOp::Read { .. }
            | ResourceOp::Assign { .. }
            | ResourceOp::Borrow { .. }
            | ResourceOp::Move { .. }
            | ResourceOp::Drop { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::Construct { .. } => {}
        }
    }
}

fn push_count_diagnostics(
    function: &str,
    hir: &ResourceCoverageCounts,
    resource: &ResourceCoverageCounts,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) {
    push_count_diagnostic(
        function,
        ResourceCoverageKind::DirectCall,
        hir.direct_calls,
        resource.direct_calls,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::IndirectCall,
        hir.indirect_calls,
        resource.indirect_calls,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::FunctionValue,
        hir.function_values,
        resource.function_values,
        diagnostics,
    );
    push_count_diagnostic(
        function,
        ResourceCoverageKind::RawMemory,
        hir.raw_memory_ops,
        resource.raw_memory_ops,
        diagnostics,
    );
}

fn push_count_diagnostic(
    function: &str,
    kind: ResourceCoverageKind,
    hir: usize,
    resource: usize,
    diagnostics: &mut Vec<ResourceCoverageDiagnostic>,
) {
    if hir != resource {
        diagnostics.push(ResourceCoverageDiagnostic::CountMismatch {
            function: String::from(function),
            kind,
            hir,
            resource,
        });
    }
}
