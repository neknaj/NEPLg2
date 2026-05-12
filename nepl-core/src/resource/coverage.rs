extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::diagnostic_codes::{
    DiagnosticCode, ResourceDiagnosticCode, ResourceLowerDiagnosticCode,
};
use crate::hir::HirModule;
use crate::span::Span;
use crate::types::TypeCtx;

use super::coverage_hir::hir_function_coverage;
use super::coverage_kind::ResourceCoverageKind;
use super::coverage_operation::ResourceCoveragePlaceOperation;
use super::coverage_resource::resource_function_coverage;
use super::model::{Place, ResourceModule};

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
        operation: ResourceCoveragePlaceOperation,
        place: Place,
        span: Span,
    },
}

impl ResourceCoverageDiagnostic {
    pub fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            ResourceCoverageDiagnostic::MissingFunction { .. }
            | ResourceCoverageDiagnostic::CountMismatch { .. }
            | ResourceCoverageDiagnostic::UnknownPlace { .. } => resource_lower_incomplete_code(),
        }
    }
}

fn resource_lower_incomplete_code() -> DiagnosticCode {
    DiagnosticCode::Resource(ResourceDiagnosticCode::Lower(
        ResourceLowerDiagnosticCode::Incomplete,
    ))
}

pub fn compare_hir_resource_lowering(
    module: &HirModule,
    resource: &ResourceModule,
) -> ResourceLoweringCoverage {
    let types = TypeCtx::new();
    compare_hir_resource_lowering_typed(module, resource, &types)
}

pub fn compare_hir_resource_lowering_typed(
    module: &HirModule,
    resource: &ResourceModule,
    types: &TypeCtx,
) -> ResourceLoweringCoverage {
    let mut resource_functions = BTreeMap::new();
    for function in &resource.functions {
        resource_functions.insert(function.name.as_str(), function);
    }

    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    for function in &module.functions {
        let hir = hir_function_coverage(function, module, types, &module.string_literals);
        let Some(resource_function) = resource_functions.get(function.name.as_str()) else {
            diagnostics.push(ResourceCoverageDiagnostic::MissingFunction {
                name: function.name.clone(),
                span: function.span,
            });
            continue;
        };
        let resource_counts =
            resource_function_coverage(&function.name, resource_function, &mut diagnostics);
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
