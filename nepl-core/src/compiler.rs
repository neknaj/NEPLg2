use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast;
use crate::codegen_wasm;
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::{
    BackendDiagnosticCode, DiagnosticCode, LoaderDiagnosticCode, ResolveDiagnosticCode,
    WasmDiagnosticCode,
};
use crate::effects::RawMemoryOp;
use crate::error::CoreError;
use crate::lexer;
use crate::monomorphize;
use crate::parser;
use crate::passes;
use crate::source_map::SourceMap;
use crate::span::FileId;
use crate::span::Span;
use crate::typecheck;
use wasmparser::{Imports, Parser, Payload, TypeRef, Validator};

/// コンパイル対象プラットフォーム。
///
/// - `Wasm`: 素の wasm 実行環境を想定
/// - `Wasi`: WASI 実行環境を想定（`wasm` の上位互換として扱う）
/// - `Llvm`: LLVM IR 出力向けのネイティブ経路（`nepl-cli` 側で処理）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileTarget {
    Wasm,
    Wasi,
    Wasix,
    Llvm,
}

impl CompileTarget {
    pub fn allows(&self, gate: &str) -> bool {
        crate::target_gate::target_gate_allows_expr(gate, *self)
    }
}

pub fn target_gate_allows_expr(expr: &str, active: CompileTarget) -> bool {
    crate::target_gate::target_gate_allows_expr(expr, active)
}

/// ビルドプロファイル。
///
/// 条件付きコンパイル（`#if[profile=...]`）の判定に使用する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    Debug,
    Release,
}

impl BuildProfile {
    pub fn detect() -> Self {
        if cfg!(debug_assertions) {
            BuildProfile::Debug
        } else {
            BuildProfile::Release
        }
    }
}

/// コンパイル実行オプション。
#[derive(Debug, Clone, Copy)]
pub struct CompileOptions {
    /// Explicit target override (e.g., CLI flag). If None, #target or default is used.
    pub target: Option<CompileTarget>,
    /// Emit verbose compiler logs for debugging.
    pub verbose: bool,
    /// Explicit profile override for conditional compilation.
    pub profile: Option<BuildProfile>,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            target: None,
            verbose: false,
            profile: None,
        }
    }
}

/// コンパイル成果物の構成オプション。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompilationArtifactOptions {
    /// WAT 出力に付与する関数・ローカル・型コメントを生成する。
    pub include_wat_comments: bool,
}

impl Default for CompilationArtifactOptions {
    fn default() -> Self {
        Self {
            include_wat_comments: true,
        }
    }
}

/// コンパイル成果物。
#[derive(Debug, Clone)]
pub struct CompilationArtifact {
    pub wasm: Vec<u8>,
    /// WAT 向けの補助情報（関数・ローカル変数・型）。
    /// 先頭コメントとして付与することを想定し、プレーンテキストで保持する。
    pub wat_comments: String,
}

/// 解析済みモジュールを最終成果物へ変換する。
///
/// この関数はコンパイルパイプラインの中核であり、以下の段階を順番に実行する。
/// 1. target/profile の確定
/// 2. typecheck
/// 3. Resource IR 静的検査用の source HIR monomorphize
/// 4. Resource IR 静的検査
/// 5. codegen 用 drop 挿入と monomorphize
/// 6. wasm 生成と妥当性検証
pub fn compile_module(
    module: ast::Module,
    options: CompileOptions,
) -> Result<CompilationArtifact, CoreError> {
    compile_module_with_artifact_options(module, options, CompilationArtifactOptions::default())
}

pub fn compile_module_with_source_map(
    module: ast::Module,
    source_map: Option<&SourceMap>,
    options: CompileOptions,
) -> Result<CompilationArtifact, CoreError> {
    compile_module_with_source_map_and_artifact_options(
        module,
        source_map,
        options,
        CompilationArtifactOptions::default(),
    )
}

pub fn compile_module_with_artifact_options(
    module: ast::Module,
    options: CompileOptions,
    artifact_options: CompilationArtifactOptions,
) -> Result<CompilationArtifact, CoreError> {
    compile_module_with_source_map_and_artifact_options(module, None, options, artifact_options)
}

pub fn compile_module_with_source_map_and_artifact_options(
    module: ast::Module,
    source_map: Option<&SourceMap>,
    options: CompileOptions,
    artifact_options: CompilationArtifactOptions,
) -> Result<CompilationArtifact, CoreError> {
    crate::log::set_verbose(options.verbose);
    let target = resolve_target(&module, options)?;
    if matches!(target, CompileTarget::Llvm) {
        let mut diags = Vec::new();
        diags.push(Diagnostic::error_with_code(
            DiagnosticCode::Backend(BackendDiagnosticCode::TargetRequiresCli),
            "llvm target is CLI-only and is not handled by the wasm backend; use nepl-cli LLVM pipeline",
            Span::dummy(),
        ));
        return Err(CoreError::from_diagnostics(diags));
    }
    let profile = options.profile.unwrap_or(BuildProfile::detect());
    let prepared =
        prepare_module_for_codegen_with_source_map(&module, target, profile, source_map)?;
    let pre_codegen_diags =
        passes::codegen_precheck::precheck_wasm_codegen(&prepared.types, &prepared.hir_module);
    if pre_codegen_diags
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostic::Severity::Error))
    {
        let mut diagnostics = prepared.diagnostics;
        diagnostics.extend(pre_codegen_diags);
        return Err(CoreError::from_diagnostics(diagnostics));
    }

    emit_wasm(
        &prepared.types,
        &prepared.hir_module,
        prepared.diagnostics,
        artifact_options.include_wat_comments,
    )
}

/// 解析済みモジュールを診断目的で検証する。
///
/// `--check` のような確認用途では成果物を生成しないため、
/// artifact emission へは進まない。ただし静的安全性の authority は artifact pipeline
/// と同一でなければならないため、target/profile precheck、typecheck、monomorphize、
/// Resource IR static gate、drop elaboration bridge までを共有 prepare phase で実行する。
/// これにより、深いが正当な HIR を codegen へ渡して native stack overflow させず、
/// memory/resource safety diagnostic だけを取りこぼさない。
pub fn check_module(module: ast::Module, options: CompileOptions) -> Result<(), CoreError> {
    check_module_with_source_map(module, None, options)
}

pub fn check_module_with_source_map(
    module: ast::Module,
    source_map: Option<&SourceMap>,
    options: CompileOptions,
) -> Result<(), CoreError> {
    crate::log::set_verbose(options.verbose);
    let target = resolve_target(&module, options)?;
    let profile = options.profile.unwrap_or(BuildProfile::detect());
    prepare_module_for_codegen_with_source_map(&module, target, profile, source_map)?;
    Ok(())
}

/// ソーステキストから wasm を生成する。
///
/// lexer/parser の診断がある場合は早期にエラーを返し、
/// その後の段階は `compile_module` に委譲する。
pub fn compile_wasm(
    file_id: FileId,
    source: &str,
    options: CompileOptions,
) -> Result<CompilationArtifact, CoreError> {
    let lex = lexer::lex(file_id, source);
    let parse = parser::parse_tokens(file_id, lex);
    let module = match parse.module {
        Some(m) => m,
        None => return Err(CoreError::from_diagnostics(parse.diagnostics)),
    };
    if parse
        .diagnostics
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostic::Severity::Error))
    {
        return Err(CoreError::from_diagnostics(parse.diagnostics));
    }

    match compile_module(module, options) {
        Ok(artifact) => Ok(artifact),
        Err(CoreError::Diagnostics(mut ds)) => {
            let mut diags = parse.diagnostics;
            diags.append(&mut ds);
            Err(CoreError::from_diagnostics(diags))
        }
        Err(e) => Err(e),
    }
}

struct TypedProgram {
    types: crate::types::TypeCtx,
    module: crate::hir::HirModule,
    diagnostics: Vec<Diagnostic>,
}

pub struct PreparedProgram {
    pub types: crate::types::TypeCtx,
    pub hir_module: crate::hir::HirModule,
    pub resource_drop_elaboration_plan: crate::resource::ResourceDropElaborationPlan,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct PreparedLlvmProgram {
    pub program: PreparedProgram,
    pub reachable_set: BTreeSet<String>,
    pub resolved_entries: BTreeMap<String, String>,
}

fn run_typecheck(
    module: &ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
    source_map: Option<&SourceMap>,
) -> Result<TypedProgram, CoreError> {
    let tc = typecheck::typecheck(module, target, profile, source_map);
    match tc.module {
        Some(m) => Ok(TypedProgram {
            types: tc.types,
            module: m,
            diagnostics: tc.diagnostics,
        }),
        None => Err(CoreError::from_diagnostics(tc.diagnostics)),
    }
}

fn extend_unresolved_trait_call_diagnostics(
    diagnostics: &mut Vec<Diagnostic>,
    unresolved_trait_calls: Vec<monomorphize::UnresolvedTraitCall>,
) {
    diagnostics.extend(unresolved_trait_calls.into_iter().map(|call| {
        Diagnostic::error_with_code(
            DiagnosticCode::Backend(BackendDiagnosticCode::TraitCallUnresolved),
            format!(
                "unresolved trait call remained after monomorphize: {}",
                call.description
            ),
            call.span,
        )
    }));
}

fn run_resource_static_check(
    hir_module: &crate::hir::HirModule,
    types: &crate::types::TypeCtx,
    diagnostics: &mut Vec<Diagnostic>,
    source_map: Option<&SourceMap>,
) -> Result<crate::resource::ResourceDropElaborationPlan, CoreError> {
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    run_resource_shadow_check(hir_module, types);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_shadow_check", stage_start);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let resource = crate::resource::lower_hir_module(hir_module, types);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_lowering", stage_start);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let lowering_coverage =
        crate::resource::compare_hir_resource_lowering_typed(hir_module, &resource, types);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_lowering_coverage", stage_start);
    run_resource_lowering_coverage_gate(&lowering_coverage, diagnostics)?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let initialized_moves = crate::resource::check_resource_initialized_moves(&resource, types);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_initialized_moves", stage_start);
    run_resource_cell_gate(&initialized_moves, diagnostics)?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let drop_elaboration_plan = run_resource_drop_elaboration_plan_gate(
        crate::resource::compute_resource_drop_elaboration_plan(&resource, &initialized_moves),
        diagnostics,
    )?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_drop_elaboration_plan", stage_start);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let borrow_lifetimes = crate::resource::check_resource_borrow_lifetimes(&resource, types);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_borrow_lifetimes", stage_start);
    run_resource_borrow_lifetime_gate(&borrow_lifetimes, diagnostics)?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let effect_boundaries =
        crate::resource::check_resource_effect_boundaries_typed(&resource, types);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_effect_boundaries", stage_start);
    run_resource_effect_boundary_gate(&effect_boundaries, diagnostics, source_map)?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let owner_obligations = crate::resource::check_resource_owner_obligations(&resource, types);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_owner_obligations", stage_start);
    run_resource_owner_obligation_gate(&owner_obligations, diagnostics)?;

    Ok(drop_elaboration_plan)
}

fn run_resource_drop_elaboration_plan_gate(
    plan: Result<
        crate::resource::ResourceDropElaborationPlan,
        Vec<crate::resource::ResourceDropElaborationPlanError>,
    >,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<crate::resource::ResourceDropElaborationPlan, CoreError> {
    let errors = match plan {
        Ok(plan) => return Ok(plan),
        Err(errors) => errors,
    };
    diagnostics.extend(
        errors
            .iter()
            .map(resource_drop_elaboration_plan_error_to_error),
    );
    Err(CoreError::from_diagnostics(diagnostics.clone()))
}

fn resource_drop_elaboration_plan_error_to_error(
    error: &crate::resource::ResourceDropElaborationPlanError,
) -> Diagnostic {
    match error {
        crate::resource::ResourceDropElaborationPlanError::DuplicateFunctionCheck { function } => {
            Diagnostic::error_with_code(
                error.diagnostic_code(),
                format!(
                    "resource drop elaboration found duplicate initialized-state check for function '{}'",
                    function
                ),
                Span::dummy(),
            )
        }
        crate::resource::ResourceDropElaborationPlanError::MissingFunctionCheck { function } => {
            Diagnostic::error_with_code(
                error.diagnostic_code(),
                format!(
                    "resource drop elaboration is missing initialized-state check for function '{}'",
                    function
                ),
                Span::dummy(),
            )
        }
        crate::resource::ResourceDropElaborationPlanError::MissingResourceFunction { function } => {
            Diagnostic::error_with_code(
                error.diagnostic_code(),
                format!(
                    "resource drop elaboration check references missing function '{}'",
                    function
                ),
                Span::dummy(),
            )
        }
        crate::resource::ResourceDropElaborationPlanError::InvalidDropPointPath {
            function,
            path,
            span,
            error: path_error,
        } => Diagnostic::error_with_code(
            error.diagnostic_code(),
            format!(
                "resource drop elaboration point in function '{}' does not resolve to its required insertion point: {:?} (path {:?})",
                function, path_error, path
            ),
            *span,
        ),
        crate::resource::ResourceDropElaborationPlanError::DropPlaceOutsideEndScope {
            function,
            path,
            place,
            span,
        } => Diagnostic::error_with_code(
            error.diagnostic_code(),
            format!(
                "resource drop elaboration point in function '{}' references place {:?} outside its EndScope locals (path {:?})",
                function, place, path
            ),
            *span,
        ),
        crate::resource::ResourceDropElaborationPlanError::DropPlaceDoesNotMatchAssignmentTarget {
            function,
            path,
            place,
            target,
            span,
        } => Diagnostic::error_with_code(
            error.diagnostic_code(),
            format!(
                "resource drop elaboration point in function '{}' references overwrite place {:?}, but assignment target is {:?} (path {:?})",
                function, place, target, path
            ),
            *span,
        ),
        crate::resource::ResourceDropElaborationPlanError::MissingDropBinding {
            function,
            path,
            place,
            span,
        } => Diagnostic::error_with_code(
            error.diagnostic_code(),
            format!(
                "resource drop elaboration point in function '{}' references place {:?} without a source binding name (path {:?})",
                function, place, path
            ),
            *span,
        ),
    }
}

fn run_resource_drop_elaboration_hir_bridge_gate(
    hir_module: &crate::hir::HirModule,
    plan: &crate::resource::ResourceDropElaborationPlan,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), CoreError> {
    let Err(errors) =
        crate::resource::validate_resource_drop_elaboration_hir_bridge(hir_module, plan)
    else {
        return Ok(());
    };
    diagnostics.extend(
        errors
            .iter()
            .map(resource_drop_elaboration_hir_bridge_error_to_error),
    );
    Err(CoreError::from_diagnostics(diagnostics.clone()))
}

fn resource_drop_elaboration_hir_bridge_error_to_error(
    error: &crate::resource::ResourceDropElaborationHirBridgeError,
) -> Diagnostic {
    match error {
        crate::resource::ResourceDropElaborationHirBridgeError::MissingSourceFunction {
            function,
            origin_name,
        } => Diagnostic::error_with_code(
            error.diagnostic_code(),
            format!(
                "resource drop elaboration function '{}' with origin '{}' has no source HIR function",
                function, origin_name
            ),
            Span::dummy(),
        ),
        crate::resource::ResourceDropElaborationHirBridgeError::MissingSourceBinding {
            function,
            origin_name,
            source_name,
            span,
        } => Diagnostic::error_with_code(
            error.diagnostic_code(),
            format!(
                "resource drop elaboration function '{}' with origin '{}' references source binding '{}' that is not available at the HIR insertion point",
                function, origin_name, source_name
            ),
            *span,
        ),
    }
}

fn run_resource_lowering_coverage_gate(
    coverage: &crate::resource::ResourceLoweringCoverage,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), CoreError> {
    let mut coverage_errors = Vec::new();
    for diagnostic in &coverage.diagnostics {
        coverage_errors.push(resource_coverage_diagnostic_to_error(diagnostic));
    }
    if coverage_errors.is_empty() {
        return Ok(());
    }
    diagnostics.extend(coverage_errors);
    Err(CoreError::from_diagnostics(diagnostics.clone()))
}

fn resource_coverage_diagnostic_to_error(
    diagnostic: &crate::resource::ResourceCoverageDiagnostic,
) -> Diagnostic {
    match diagnostic {
        crate::resource::ResourceCoverageDiagnostic::MissingFunction { name, span } => {
            Diagnostic::error_with_code(
                diagnostic.diagnostic_code(),
                format!("resource ir lowering did not produce function '{}'", name),
                *span,
            )
        }
        crate::resource::ResourceCoverageDiagnostic::CountMismatch {
            function,
            kind,
            hir,
            resource,
            span,
        } => Diagnostic::error_with_code(
            diagnostic.diagnostic_code(),
            format!(
                "resource ir lowering lost {:?} coverage in function '{}' (HIR={}, ResourceIR={})",
                kind, function, hir, resource
            ),
            *span,
        ),
        crate::resource::ResourceCoverageDiagnostic::UnknownPlace {
            function,
            operation,
            place,
            span,
        } => Diagnostic::error_with_code(
            diagnostic.diagnostic_code(),
            format!(
                "resource ir lowering produced unknown place for {} in function '{}': {:?}",
                operation.as_str(),
                function,
                place
            ),
            *span,
        ),
    }
}

fn run_resource_cell_gate(
    report: &crate::resource::ResourceCheckReport,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), CoreError> {
    let mut cell_errors = Vec::new();
    for diagnostic in &report.diagnostics {
        cell_errors.push(resource_cell_diagnostic_to_error(diagnostic));
    }
    if cell_errors.is_empty() {
        return Ok(());
    }
    diagnostics.extend(cell_errors);
    Err(CoreError::from_diagnostics(diagnostics.clone()))
}

fn resource_cell_diagnostic_to_error(
    diagnostic: &crate::resource::ResourceCheckDiagnostic,
) -> Diagnostic {
    match diagnostic {
        crate::resource::ResourceCheckDiagnostic::CellUnavailable {
            function,
            operation,
            place,
            state,
            span,
        } => Diagnostic::error_with_code(
            diagnostic.diagnostic_code(),
            format!(
                "resource ir cell state violation in function '{}': {:?} on {:?} found {:?}",
                function, operation, place, state
            ),
            *span,
        ),
    }
}

fn run_resource_owner_obligation_gate(
    report: &crate::resource::ResourceOwnerCheckReport,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), CoreError> {
    let mut owner_errors = Vec::new();
    for diagnostic in &report.diagnostics {
        owner_errors.push(resource_owner_diagnostic_to_error(diagnostic));
    }
    if owner_errors.is_empty() {
        return Ok(());
    }
    diagnostics.extend(owner_errors);
    Err(CoreError::from_diagnostics(diagnostics.clone()))
}

fn resource_owner_diagnostic_to_error(
    diagnostic: &crate::resource::ResourceOwnerDiagnostic,
) -> Diagnostic {
    match diagnostic {
        crate::resource::ResourceOwnerDiagnostic::OwnerUnavailable {
            function,
            operation,
            place,
            state,
            span,
        } => Diagnostic::error_with_code(
            diagnostic.diagnostic_code(),
            format!(
                "resource ir owner obligation violation in function '{}': {:?} on {:?} found {:?}",
                function, operation, place, state
            ),
            *span,
        ),
        crate::resource::ResourceOwnerDiagnostic::OwnerLeaked {
            function,
            place,
            storage,
            span,
        } => Diagnostic::error_with_code(
            diagnostic.diagnostic_code(),
            format!(
                "resource ir owner obligation leak in function '{}': {:?} still owns {:?}",
                function, place, storage
            ),
            *span,
        ),
        crate::resource::ResourceOwnerDiagnostic::OwnerMaybeLeaked {
            function,
            place,
            span,
        } => Diagnostic::error_with_code(
            diagnostic.diagnostic_code(),
            format!(
                "resource ir owner obligation may leak in function '{}': {:?}",
                function, place
            ),
            *span,
        ),
    }
}

fn run_resource_borrow_lifetime_gate(
    report: &crate::resource::ResourceBorrowCheckReport,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), CoreError> {
    let mut borrow_errors = Vec::new();
    for diagnostic in &report.diagnostics {
        borrow_errors.push(resource_borrow_diagnostic_to_error(diagnostic));
    }
    if borrow_errors.is_empty() {
        return Ok(());
    }
    diagnostics.extend(borrow_errors);
    Err(CoreError::from_diagnostics(diagnostics.clone()))
}

fn resource_borrow_diagnostic_to_error(
    diagnostic: &crate::resource::ResourceBorrowDiagnostic,
) -> Diagnostic {
    match diagnostic {
        crate::resource::ResourceBorrowDiagnostic::BorrowConflict {
            function,
            operation,
            place,
            active,
            span,
        } => Diagnostic::error_with_code(
            diagnostic.diagnostic_code(),
            resource_borrow_conflict_message(function, *operation, place, active),
            *span,
        ),
    }
}

fn resource_borrow_conflict_message(
    function: &str,
    operation: crate::resource::ResourceBorrowOperation,
    place: &crate::resource::Place,
    active: &crate::resource::BorrowState,
) -> String {
    match operation {
        crate::resource::ResourceBorrowOperation::ReturnValue => format!(
            "resource ir borrow lifetime violation in function '{}': returning {:?} escapes active borrow {:?}",
            function, place, active
        ),
        _ => format!(
            "resource ir borrow conflict in function '{}': {:?} on {:?} conflicts with active borrow {:?}",
            function, operation, place, active
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic_codes::EffectDiagnosticCode;
    use crate::resource::{
        BorrowState, CellState, OwnerState, Place, RawAddressAliasKind, RawAddressViewKind,
        RawMemoryOp, ResourceBorrowDiagnostic, ResourceBorrowOperation, ResourceCheckDiagnostic,
        ResourceCheckOperation, ResourceEffectBoundaryDiagnostic, ResourceEffectCallKind,
        ResourceId, ResourceOwnerDiagnostic, ResourceOwnerOperation, StorageId,
    };
    use crate::source_map::{
        SourceCapabilities, SourceCapabilitySpan, SourceCapabilityUseSite, SourceMap,
    };
    use alloc::boxed::Box;

    fn use_site_capabilities(use_site: SourceCapabilityUseSite) -> SourceCapabilities {
        let mut capabilities = SourceCapabilities::none();
        capabilities.insert_use_site(use_site);
        capabilities
    }

    fn raw_memory_structural_capabilities(span: Span) -> SourceCapabilities {
        use_site_capabilities(SourceCapabilityUseSite::RawMemoryStructuralBoundary {
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    fn raw_memory_operation_capabilities(operation: RawMemoryOp, span: Span) -> SourceCapabilities {
        use_site_capabilities(SourceCapabilityUseSite::RawMemoryOperationBoundary {
            operation,
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    fn raw_address_view_capabilities(span: Span) -> SourceCapabilities {
        use_site_capabilities(SourceCapabilityUseSite::RawAddressViewBoundary {
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    fn raw_address_alias_capabilities(span: Span) -> SourceCapabilities {
        use_site_capabilities(SourceCapabilityUseSite::RawAddressAliasBoundary {
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    fn owner_token_construct_capabilities(span: Span) -> SourceCapabilities {
        use_site_capabilities(SourceCapabilityUseSite::OwnerTokenConstructBoundary {
            span: SourceCapabilitySpan::from_span(span),
        })
    }

    #[test]
    fn resource_cell_gate_maps_cell_diagnostics_to_cell_code() {
        let types = crate::types::TypeCtx::new();
        for operation in [
            ResourceCheckOperation::RawMemoryLoadCell,
            ResourceCheckOperation::RawMemoryDeallocCell,
            ResourceCheckOperation::Read,
            ResourceCheckOperation::ReturnValue,
        ] {
            let place = Place::temporary(ResourceId(0), types.i32());
            let diagnostic = ResourceCheckDiagnostic::CellUnavailable {
                function: String::from("main"),
                operation,
                place,
                state: CellState::Moved,
                span: Span::dummy(),
            };

            let error = resource_cell_diagnostic_to_error(&diagnostic);

            assert_eq!(
                error.code,
                DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Cell(
                    crate::diagnostic_codes::ResourceCellDiagnosticCode::Moved,
                ))
            );
            assert!(error.message.contains("resource ir cell state violation"));
        }
    }

    #[test]
    fn resource_owner_gate_maps_owner_diagnostics_to_owner_code() {
        let types = crate::types::TypeCtx::new();
        let place = Place::temporary(ResourceId(0), types.i32());
        let diagnostic = ResourceOwnerDiagnostic::OwnerUnavailable {
            function: String::from("main"),
            operation: ResourceOwnerOperation::Dealloc,
            place,
            state: OwnerState::Freed,
            span: Span::dummy(),
        };

        let error = resource_owner_diagnostic_to_error(&diagnostic);

        assert_eq!(
            error.code,
            DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Owner(
                crate::diagnostic_codes::ResourceOwnerDiagnosticCode::DoubleFree,
            ))
        );
        assert!(error
            .message
            .contains("resource ir owner obligation violation"));
    }

    #[test]
    fn resource_owner_gate_maps_leaks_to_owner_code() {
        let types = crate::types::TypeCtx::new();
        let place = Place::temporary(ResourceId(0), types.i32());
        let diagnostic = ResourceOwnerDiagnostic::OwnerLeaked {
            function: String::from("main"),
            place,
            storage: StorageId(0),
            span: Span::dummy(),
        };

        let error = resource_owner_diagnostic_to_error(&diagnostic);

        assert_eq!(
            error.code,
            DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Owner(
                crate::diagnostic_codes::ResourceOwnerDiagnosticCode::Leak,
            ))
        );
        assert!(error.message.contains("resource ir owner obligation leak"));
    }

    #[test]
    fn resource_owner_gate_maps_no_free_obligation_to_owner_code() {
        let types = crate::types::TypeCtx::new();
        let place = Place::temporary(ResourceId(0), types.i32());
        let diagnostic = ResourceOwnerDiagnostic::OwnerUnavailable {
            function: String::from("main"),
            operation: ResourceOwnerOperation::Dealloc,
            place,
            state: OwnerState::NoFreeObligation,
            span: Span::dummy(),
        };

        let error = resource_owner_diagnostic_to_error(&diagnostic);

        assert_eq!(
            error.code,
            DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Owner(
                crate::diagnostic_codes::ResourceOwnerDiagnosticCode::NoFreeObligation,
            ))
        );
        assert!(error
            .message
            .contains("resource ir owner obligation violation"));
    }

    #[test]
    fn resource_owner_gate_maps_reserved_owner_to_reserved_code() {
        let types = crate::types::TypeCtx::new();
        let place = Place::temporary(ResourceId(0), types.i32());
        let diagnostic = ResourceOwnerDiagnostic::OwnerUnavailable {
            function: String::from("main"),
            operation: ResourceOwnerOperation::CallArgument,
            place,
            state: OwnerState::Reserved { storage: None },
            span: Span::dummy(),
        };

        let error = resource_owner_diagnostic_to_error(&diagnostic);

        assert_eq!(
            error.code,
            DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Owner(
                crate::diagnostic_codes::ResourceOwnerDiagnosticCode::Reserved,
            ))
        );
        assert!(error
            .message
            .contains("resource ir owner obligation violation"));
    }

    #[test]
    fn resource_borrow_gate_maps_return_escape_to_borrow_return_code() {
        let types = crate::types::TypeCtx::new();
        let place = Place::temporary(ResourceId(0), types.i32());
        let diagnostic = ResourceBorrowDiagnostic::BorrowConflict {
            function: String::from("main"),
            operation: ResourceBorrowOperation::ReturnValue,
            place,
            active: BorrowState::Shared { count: 1 },
            span: Span::dummy(),
        };

        let error = resource_borrow_diagnostic_to_error(&diagnostic);

        assert_eq!(
            error.code,
            DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Borrow(
                crate::diagnostic_codes::ResourceBorrowDiagnosticCode::ReturnEscape,
            ))
        );
        assert!(error
            .message
            .contains("resource ir borrow lifetime violation"));
    }

    #[test]
    fn resource_borrow_gate_maps_non_return_conflicts_to_borrow_codes() {
        let types = crate::types::TypeCtx::new();
        let place = Place::temporary(ResourceId(0), types.i32());
        for (operation, active, expected) in [
            (
                ResourceBorrowOperation::Read,
                BorrowState::Unique {
                    source: Box::new(place.clone()),
                },
                DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Borrow(
                    crate::diagnostic_codes::ResourceBorrowDiagnosticCode::UseDuringUnique,
                )),
            ),
            (
                ResourceBorrowOperation::Move,
                BorrowState::Shared { count: 1 },
                DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Borrow(
                    crate::diagnostic_codes::ResourceBorrowDiagnosticCode::MoveFromShared,
                )),
            ),
            (
                ResourceBorrowOperation::Assign,
                BorrowState::Shared { count: 1 },
                DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Borrow(
                    crate::diagnostic_codes::ResourceBorrowDiagnosticCode::AssignDuringShared,
                )),
            ),
            (
                ResourceBorrowOperation::Drop,
                BorrowState::Unique {
                    source: Box::new(place.clone()),
                },
                DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Borrow(
                    crate::diagnostic_codes::ResourceBorrowDiagnosticCode::DropDuringUnique,
                )),
            ),
            (
                ResourceBorrowOperation::SharedBorrow,
                BorrowState::Unique {
                    source: Box::new(place.clone()),
                },
                DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Borrow(
                    crate::diagnostic_codes::ResourceBorrowDiagnosticCode::BorrowDuringUnique,
                )),
            ),
            (
                ResourceBorrowOperation::UniqueBorrow,
                BorrowState::Shared { count: 1 },
                DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Borrow(
                    crate::diagnostic_codes::ResourceBorrowDiagnosticCode::UniqueDuringShared,
                )),
            ),
        ] {
            let diagnostic = ResourceBorrowDiagnostic::BorrowConflict {
                function: String::from("main"),
                operation,
                place: place.clone(),
                active,
                span: Span::dummy(),
            };

            let error = resource_borrow_diagnostic_to_error(&diagnostic);

            assert_eq!(error.code, expected);
            assert!(error.message.contains("resource ir borrow conflict"));
        }
    }

    #[test]
    fn resource_effect_gate_maps_raw_identity_escape_to_resource_raw_code() {
        let types = crate::types::TypeCtx::new();
        let place = Place::temporary(ResourceId(0), types.i32());
        let diagnostic = ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function: String::from("leak_raw"),
            operation: RawMemoryOp::Alloc,
            place,
            origin_span: Span::dummy(),
            span: Span::dummy(),
        };

        let error = resource_effect_boundary_diagnostic_to_error(&diagnostic);

        assert_eq!(
            error.code,
            DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Raw(
                crate::diagnostic_codes::ResourceRawDiagnosticCode::IdentityEscape,
            ))
        );
        assert!(error.message.contains("returns raw address identity"));
    }

    #[test]
    fn resource_effect_gate_allows_raw_identity_escape_inside_raw_boundary() {
        let types = crate::types::TypeCtx::new();
        let mut source_map = SourceMap::new();
        let raw_file = source_map.add("stdlib/core/mem/allocator.nepl", String::new());
        let span = Span::new(raw_file, 0, 1);
        source_map.set_capabilities(raw_file, raw_memory_structural_capabilities(span));
        let diagnostic = ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function: String::from("alloc_raw"),
            operation: RawMemoryOp::Alloc,
            place: Place::temporary(ResourceId(0), types.i32()),
            origin_span: span,
            span,
        };

        assert!(resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
            &diagnostic,
            Some(&source_map),
        ));
    }

    #[test]
    fn resource_effect_gate_allows_raw_alloc_identity_with_alloc_capability() {
        let types = crate::types::TypeCtx::new();
        let mut source_map = SourceMap::new();
        let alloc_file = source_map.add("stdlib/core/mem/allocator.nepl", String::new());
        let store_file = source_map.add("stdlib/core/mem/store.nepl", String::new());
        let alloc_span = Span::new(alloc_file, 0, 1);
        let store_span = Span::new(store_file, 0, 1);
        source_map.set_capabilities(
            alloc_file,
            raw_memory_operation_capabilities(RawMemoryOp::Alloc, alloc_span),
        );
        source_map.set_capabilities(
            store_file,
            raw_memory_operation_capabilities(RawMemoryOp::Store, store_span),
        );
        let place = Place::temporary(ResourceId(0), types.i32());
        let alloc_escape = ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function: String::from("alloc_raw"),
            operation: RawMemoryOp::Alloc,
            place: place.clone(),
            origin_span: alloc_span,
            span: alloc_span,
        };
        let store_escape = ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function: String::from("alloc_raw"),
            operation: RawMemoryOp::Alloc,
            place,
            origin_span: store_span,
            span: store_span,
        };

        assert!(resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
            &alloc_escape,
            Some(&source_map),
        ));
        assert!(
            !resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
                &store_escape,
                Some(&source_map),
            )
        );
    }

    #[test]
    fn resource_effect_gate_requires_raw_identity_origin_span_capability() {
        let types = crate::types::TypeCtx::new();
        let mut source_map = SourceMap::new();
        let file = source_map.add("stdlib/core/mem/allocator.nepl", String::new());
        let unrelated_alloc_span = Span::new(file, 40, 45);
        let raw_identity_origin_span = Span::new(file, 10, 15);
        let return_span = Span::new(file, 0, 60);
        source_map.set_capabilities(
            file,
            raw_memory_operation_capabilities(RawMemoryOp::Alloc, unrelated_alloc_span),
        );
        let diagnostic = ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function: String::from("alloc_raw"),
            operation: RawMemoryOp::Alloc,
            place: Place::temporary(ResourceId(0), types.i32()),
            origin_span: raw_identity_origin_span,
            span: return_span,
        };

        assert!(
            !resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
                &diagnostic,
                Some(&source_map),
            )
        );

        source_map.set_capabilities(
            file,
            raw_memory_operation_capabilities(RawMemoryOp::Alloc, raw_identity_origin_span),
        );

        assert!(resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
            &diagnostic,
            Some(&source_map),
        ));
    }

    #[test]
    fn resource_effect_gate_maps_impure_indirect_call_to_effect_code() {
        let diagnostic = ResourceEffectBoundaryDiagnostic::ImpureCallInPureFunction {
            function: String::from("main"),
            call: ResourceEffectCallKind::Indirect,
            span: Span::dummy(),
        };

        let error = resource_effect_boundary_diagnostic_to_error(&diagnostic);

        assert_eq!(
            error.code,
            DiagnosticCode::Effect(EffectDiagnosticCode::PureCallsImpure)
        );
        assert!(error.message.contains("impure function value"));
    }

    #[test]
    fn resource_effect_gate_maps_unsafe_memory_to_effect_code() {
        let diagnostic = ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction {
            function: String::from("store_raw"),
            operation: RawMemoryOp::Store,
            span: Span::dummy(),
        };

        let error = resource_effect_boundary_diagnostic_to_error(&diagnostic);

        assert_eq!(
            error.code,
            DiagnosticCode::Effect(EffectDiagnosticCode::PureCallsImpure)
        );
        assert!(error.message.contains("unsafe memory operation 'store'"));
    }

    #[test]
    fn resource_effect_gate_maps_raw_memory_outside_boundary_to_resource_raw_code() {
        let diagnostic = ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary {
            function: String::from("store_raw"),
            operation: RawMemoryOp::Store,
            span: Span::dummy(),
        };

        let error = resource_effect_boundary_diagnostic_to_error(&diagnostic);

        assert_eq!(
            error.code,
            DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Raw(
                crate::diagnostic_codes::ResourceRawDiagnosticCode::MemoryOutsideBoundary,
            ))
        );
        assert!(error.message.contains("raw memory operation 'store'"));
        assert!(error.message.contains("outside raw-memory boundary"));
    }

    #[test]
    fn resource_effect_gate_maps_raw_address_view_outside_boundary_to_resource_raw_code() {
        let diagnostic = ResourceEffectBoundaryDiagnostic::RawAddressViewOutsideBoundary {
            function: String::from("offset_ptr"),
            kind: RawAddressViewKind::MemPtrOffset,
            span: Span::dummy(),
        };

        let error = resource_effect_boundary_diagnostic_to_error(&diagnostic);

        assert_eq!(
            error.code,
            DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Raw(
                crate::diagnostic_codes::ResourceRawDiagnosticCode::MemoryOutsideBoundary,
            ))
        );
        assert!(error.message.contains("raw address view 'mem_ptr_offset'"));
        assert!(error.message.contains("outside raw-memory boundary"));
    }

    #[test]
    fn resource_effect_gate_maps_raw_address_alias_outside_boundary_to_resource_raw_code() {
        let diagnostic = ResourceEffectBoundaryDiagnostic::RawAddressAliasOutsideBoundary {
            function: String::from("mem_ptr_wrap"),
            kind: RawAddressAliasKind::InternalHelper,
            span: Span::dummy(),
        };

        let error = resource_effect_boundary_diagnostic_to_error(&diagnostic);

        assert_eq!(
            error.code,
            DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Raw(
                crate::diagnostic_codes::ResourceRawDiagnosticCode::MemoryOutsideBoundary,
            ))
        );
        assert!(error
            .message
            .contains("raw address alias 'internal_helper'"));
        assert!(error.message.contains("outside raw-memory boundary"));
    }

    #[test]
    fn resource_effect_gate_allows_raw_memory_inside_raw_boundary() {
        let mut source_map = SourceMap::new();
        let raw_file = source_map.add("stdlib/core/mem/raw.nepl", String::new());
        let span = Span::new(raw_file, 0, 1);
        source_map.set_capabilities(
            raw_file,
            raw_memory_operation_capabilities(RawMemoryOp::Store, span),
        );
        let diagnostic = ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary {
            function: String::from("store_raw"),
            operation: RawMemoryOp::Store,
            span,
        };

        assert!(resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
            &diagnostic,
            Some(&source_map),
        ));
    }

    #[test]
    fn resource_effect_gate_allows_raw_address_view_inside_raw_boundary() {
        let mut source_map = SourceMap::new();
        let raw_file = source_map.add("stdlib/core/mem/pointer/view.nepl", String::new());
        let span = Span::new(raw_file, 0, 1);
        source_map.set_capabilities(raw_file, raw_address_view_capabilities(span));
        let diagnostic = ResourceEffectBoundaryDiagnostic::RawAddressViewOutsideBoundary {
            function: String::from("mem_ptr_add"),
            kind: RawAddressViewKind::MemPtrOffset,
            span,
        };

        assert!(resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
            &diagnostic,
            Some(&source_map),
        ));
    }

    #[test]
    fn resource_effect_gate_allows_raw_address_alias_inside_raw_boundary() {
        let mut source_map = SourceMap::new();
        let raw_file = source_map.add("stdlib/core/mem/internal.nepl", String::new());
        let span = Span::new(raw_file, 0, 1);
        source_map.set_capabilities(raw_file, raw_address_alias_capabilities(span));
        let diagnostic = ResourceEffectBoundaryDiagnostic::RawAddressAliasOutsideBoundary {
            function: String::from("mem_ptr_wrap"),
            kind: RawAddressAliasKind::InternalHelper,
            span,
        };

        assert!(resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
            &diagnostic,
            Some(&source_map),
        ));
    }

    #[test]
    fn resource_effect_gate_keeps_owner_token_construct_separate_from_raw_alias() {
        let mut source_map = SourceMap::new();
        let raw_file = source_map.add("stdlib/core/mem/pointer/region.nepl", String::new());
        let span = Span::new(raw_file, 0, 1);
        source_map.set_capabilities(raw_file, raw_address_alias_capabilities(span));
        let diagnostic = ResourceEffectBoundaryDiagnostic::RawAddressAliasOutsideBoundary {
            function: String::from("region_new"),
            kind: RawAddressAliasKind::OwnerTokenConstruct,
            span,
        };

        assert!(
            !resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
                &diagnostic,
                Some(&source_map),
            )
        );

        source_map.set_capabilities(raw_file, owner_token_construct_capabilities(span));
        assert!(resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
            &diagnostic,
            Some(&source_map),
        ));
    }

    #[test]
    fn resource_effect_gate_requires_matching_raw_operation_capability() {
        let mut source_map = SourceMap::new();
        let raw_file = source_map.add("stdlib/core/mem/raw_store.nepl", String::new());
        let span = Span::new(raw_file, 0, 1);
        source_map.set_capabilities(
            raw_file,
            raw_memory_operation_capabilities(RawMemoryOp::Store, span),
        );
        let store = ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary {
            function: String::from("store_raw"),
            operation: RawMemoryOp::Store,
            span,
        };
        let load = ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary {
            function: String::from("load_raw"),
            operation: RawMemoryOp::Load,
            span,
        };

        assert!(resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
            &store,
            Some(&source_map),
        ));
        assert!(
            !resource_effect_boundary_diagnostic_is_raw_boundary_allowed(&load, Some(&source_map),)
        );
    }

    #[test]
    fn resource_effect_gate_maps_unknown_effect_to_lower_incomplete_code() {
        let diagnostic = ResourceEffectBoundaryDiagnostic::UnknownEffect {
            function: String::from("main"),
            reason: crate::resource::UnknownEffectReason::SyntheticTestFixture,
            span: Span::dummy(),
        };

        let error = resource_effect_boundary_diagnostic_to_error(&diagnostic);

        assert_eq!(
            error.code,
            DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Lower(
                crate::diagnostic_codes::ResourceLowerDiagnosticCode::Incomplete,
            ))
        );
        assert!(error.message.contains("unknown effect"));
        assert!(error.message.contains("synthetic_test_fixture"));
    }
}

fn run_resource_effect_boundary_gate(
    report: &crate::resource::ResourceEffectBoundaryReport,
    diagnostics: &mut Vec<Diagnostic>,
    source_map: Option<&SourceMap>,
) -> Result<(), CoreError> {
    let mut effect_errors = Vec::new();
    for diagnostic in &report.diagnostics {
        if resource_effect_boundary_diagnostic_is_raw_boundary_allowed(diagnostic, source_map) {
            continue;
        }
        effect_errors.push(resource_effect_boundary_diagnostic_to_error(diagnostic));
    }
    if effect_errors.is_empty() {
        return Ok(());
    }
    diagnostics.extend(effect_errors);
    Err(CoreError::from_diagnostics(diagnostics.clone()))
}

fn resource_effect_boundary_diagnostic_span(
    diagnostic: &crate::resource::ResourceEffectBoundaryDiagnostic,
) -> Option<Span> {
    match diagnostic {
        crate::resource::ResourceEffectBoundaryDiagnostic::ImpureCallInPureFunction {
            span,
            ..
        }
        | crate::resource::ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction {
            span,
            ..
        }
        | crate::resource::ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary {
            span,
            ..
        }
        | crate::resource::ResourceEffectBoundaryDiagnostic::RawAddressViewOutsideBoundary {
            span,
            ..
        }
        | crate::resource::ResourceEffectBoundaryDiagnostic::RawAddressAliasOutsideBoundary {
            span,
            ..
        }
        | crate::resource::ResourceEffectBoundaryDiagnostic::CheckedMemPtrOutsideBoundary {
            span,
            ..
        }
        | crate::resource::ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            span,
            ..
        }
        | crate::resource::ResourceEffectBoundaryDiagnostic::UnknownEffect { span, .. } => {
            Some(*span)
        }
    }
}

fn resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
    diagnostic: &crate::resource::ResourceEffectBoundaryDiagnostic,
    source_map: Option<&SourceMap>,
) -> bool {
    match diagnostic {
        crate::resource::ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction {
            operation,
            ..
        } => {
            let Some(span) = resource_effect_boundary_diagnostic_span(diagnostic) else {
                return false;
            };
            source_map
                .map(|map| map.raw_memory_operation_boundary_allowed_at(span, *operation))
                .unwrap_or(false)
        }
        crate::resource::ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary {
            operation,
            ..
        } => {
            let Some(span) = resource_effect_boundary_diagnostic_span(diagnostic) else {
                return false;
            };
            source_map
                .map(|map| map.raw_memory_operation_boundary_allowed_at(span, *operation))
                .unwrap_or(false)
        }
        crate::resource::ResourceEffectBoundaryDiagnostic::RawAddressViewOutsideBoundary {
            ..
        } => {
            let Some(span) = resource_effect_boundary_diagnostic_span(diagnostic) else {
                return false;
            };
            source_map
                .map(|map| map.raw_address_view_boundary_allowed_at(span))
                .unwrap_or(false)
        }
        crate::resource::ResourceEffectBoundaryDiagnostic::RawAddressAliasOutsideBoundary {
            kind,
            ..
        } => {
            let Some(span) = resource_effect_boundary_diagnostic_span(diagnostic) else {
                return false;
            };
            source_map
                .map(|map| match kind {
                    crate::resource::RawAddressAliasKind::OwnerTokenConstruct => {
                        map.owner_token_construct_boundary_allowed_at(span)
                    }
                    crate::resource::RawAddressAliasKind::InternalHelper
                    | crate::resource::RawAddressAliasKind::Transparent => {
                        map.raw_address_alias_boundary_allowed_at(span)
                    }
                })
                .unwrap_or(false)
        }
        crate::resource::ResourceEffectBoundaryDiagnostic::CheckedMemPtrOutsideBoundary {
            operation,
            ..
        } => {
            let Some(span) = resource_effect_boundary_diagnostic_span(diagnostic) else {
                return false;
            };
            source_map
                .map(|map| map.raw_memory_operation_boundary_allowed_at(span, *operation))
                .unwrap_or(false)
        }
        crate::resource::ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            operation,
            origin_span,
            ..
        } => source_map
            .map(|map| raw_identity_escape_allowed(*operation, *origin_span, map))
            .unwrap_or(false),
        crate::resource::ResourceEffectBoundaryDiagnostic::ImpureCallInPureFunction { .. } => false,
        crate::resource::ResourceEffectBoundaryDiagnostic::UnknownEffect { .. } => false,
    }
}

fn raw_identity_escape_allowed(
    operation: RawMemoryOp,
    origin_span: Span,
    source_map: &SourceMap,
) -> bool {
    if source_map.raw_memory_structural_boundary_allowed_at(origin_span) {
        return true;
    }
    match operation {
        RawMemoryOp::Alloc => {
            source_map.raw_memory_operation_boundary_allowed_at(origin_span, RawMemoryOp::Alloc)
        }
        RawMemoryOp::Realloc => {
            source_map.raw_memory_operation_boundary_allowed_at(origin_span, RawMemoryOp::Realloc)
        }
        RawMemoryOp::Dealloc
        | RawMemoryOp::Load
        | RawMemoryOp::Store
        | RawMemoryOp::LoadU8
        | RawMemoryOp::StoreU8
        | RawMemoryOp::BulkCopy
        | RawMemoryOp::BulkMove
        | RawMemoryOp::MemorySize
        | RawMemoryOp::MemoryGrow
        | RawMemoryOp::FillBytes
        | RawMemoryOp::Fill => false,
    }
}

fn resource_effect_boundary_diagnostic_to_error(
    diagnostic: &crate::resource::ResourceEffectBoundaryDiagnostic,
) -> Diagnostic {
    let code = diagnostic.diagnostic_code();
    match diagnostic {
        crate::resource::ResourceEffectBoundaryDiagnostic::ImpureCallInPureFunction {
            function,
            call,
            span,
        } => {
            let call_description = match call {
                crate::resource::ResourceEffectCallKind::Direct { name } => {
                    format!("impure function '{}'", name)
                }
                crate::resource::ResourceEffectCallKind::ExternalIo { operation } => {
                    format!("external I/O '{}'", operation)
                }
                crate::resource::ResourceEffectCallKind::Nondet { operation } => {
                    format!("nondeterministic operation '{}'", operation)
                }
                crate::resource::ResourceEffectCallKind::Indirect => {
                    String::from("impure function value")
                }
            };
            Diagnostic::error_with_code(
                code,
                format!("pure function '{}' calls {}", function, call_description),
                *span,
            )
        }
        crate::resource::ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction {
            function,
            operation,
            span,
        } => Diagnostic::error_with_code(
            code,
            format!(
                "pure function '{}' uses unsafe memory operation '{}'",
                function, operation
            ),
            *span,
        ),
        crate::resource::ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary {
            function,
            operation,
            span,
        } => Diagnostic::error_with_code(
            code,
            format!(
                "function '{}' uses raw memory operation '{}' outside raw-memory boundary",
                function, operation
            ),
            *span,
        ),
        crate::resource::ResourceEffectBoundaryDiagnostic::RawAddressViewOutsideBoundary {
            function,
            kind,
            span,
        } => Diagnostic::error_with_code(
            code,
            format!(
                "function '{}' creates raw address view '{}' outside raw-memory boundary",
                function, kind
            ),
            *span,
        ),
        crate::resource::ResourceEffectBoundaryDiagnostic::RawAddressAliasOutsideBoundary {
            function,
            kind,
            span,
        } => Diagnostic::error_with_code(
            code,
            format!(
                "function '{}' creates raw address alias '{}' outside raw-memory boundary",
                function, kind
            ),
            *span,
        ),
        crate::resource::ResourceEffectBoundaryDiagnostic::CheckedMemPtrOutsideBoundary {
            function,
            operation,
            place,
            span,
        } => Diagnostic::error_with_code(
            code,
            format!(
                "function '{}' uses checked MemPtr raw memory operation '{}' without proven pointer provenance: {:?}",
                function, operation, place
            ),
            *span,
        ),
        crate::resource::ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function,
            operation,
            span,
            ..
        } => Diagnostic::error_with_code(
            code,
            format!(
                "pure function '{}' returns raw address identity from internal {:?}",
                function, operation
            ),
            *span,
        ),
        crate::resource::ResourceEffectBoundaryDiagnostic::UnknownEffect {
            function,
            reason,
            span,
        } => Diagnostic::error_with_code(
            code,
            format!(
                "resource ir effect lowering kept unknown effect in function '{}': {}",
                function, reason
            ),
            *span,
        ),
    }
}

fn run_resource_shadow_check(hir_module: &crate::hir::HirModule, types: &crate::types::TypeCtx) {
    if !crate::log::is_verbose() {
        return;
    }
    let report = crate::resource::check_hir_resource_safety_shadow(hir_module, types);
    emit_resource_shadow_report(&report);
}

#[cfg(not(target_os = "none"))]
fn emit_resource_shadow_report(report: &crate::resource::ResourceSafetyShadowReport) {
    std::eprintln!(
        "[resource-check-shadow] lowering={} cell={} owner={} borrow={} effect={} resource_total={}",
        report.lowering_diagnostic_count(),
        report.initialized_moves.diagnostics.len(),
        report.owner_obligations.diagnostics.len(),
        report.borrow_lifetimes.diagnostics.len(),
        report.effect_boundaries.diagnostics.len(),
        report.resource_diagnostic_count()
    );
}

#[cfg(target_os = "none")]
fn emit_resource_shadow_report(_report: &crate::resource::ResourceSafetyShadowReport) {}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
fn log_compile_stage_timing(stage: &str, start: std::time::Instant) {
    if crate::log::is_verbose() || std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] {}={}ms",
            stage,
            start.elapsed().as_millis()
        );
    }
}

pub fn prepare_module_for_codegen(
    module: &ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> Result<PreparedProgram, CoreError> {
    prepare_module_for_codegen_with_source_map(module, target, profile, None)
}

pub fn prepare_module_for_codegen_with_source_map(
    module: &ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
    source_map: Option<&SourceMap>,
) -> Result<PreparedProgram, CoreError> {
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let precheck_diags =
        crate::target_precheck::precheck_module_before_codegen(module, target, profile);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("target_precheck", stage_start);
    if precheck_diags
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostic::Severity::Error))
    {
        return Err(CoreError::from_diagnostics(precheck_diags));
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let resource_tc = run_typecheck(module, target, profile, source_map)?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_typecheck", stage_start);
    let mut diagnostics = resource_tc.diagnostics;
    let mut types = resource_tc.types;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let resource_monomorphize = monomorphize::monomorphize(&mut types, resource_tc.module);
    let mut hir_module = resource_monomorphize.module;
    let resource_unresolved_trait_calls = resource_monomorphize.unresolved_trait_calls;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_monomorphize", stage_start);
    if !resource_unresolved_trait_calls.is_empty() {
        extend_unresolved_trait_call_diagnostics(&mut diagnostics, resource_unresolved_trait_calls);
        return Err(CoreError::from_diagnostics(diagnostics));
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let resource_drop_elaboration_plan =
        run_resource_static_check(&hir_module, &types, &mut diagnostics, source_map)?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_static_check", stage_start);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    run_resource_drop_elaboration_hir_bridge_gate(
        &hir_module,
        &resource_drop_elaboration_plan,
        &mut diagnostics,
    )?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_drop_bridge", stage_start);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    passes::insert_resource_drops(&mut hir_module, &mut types, &resource_drop_elaboration_plan)
        .map_err(|_| CoreError::internal("resource drop elaboration plan could not be consumed"))?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("insert_resource_drops", stage_start);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let codegen_monomorphize = monomorphize::monomorphize(&mut types, hir_module);
    let hir_module = codegen_monomorphize.module;
    let unresolved_trait_calls = codegen_monomorphize.unresolved_trait_calls;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("codegen_monomorphize", stage_start);
    if !unresolved_trait_calls.is_empty() {
        extend_unresolved_trait_call_diagnostics(&mut diagnostics, unresolved_trait_calls);
        return Err(CoreError::from_diagnostics(diagnostics));
    }
    Ok(PreparedProgram {
        types,
        hir_module,
        resource_drop_elaboration_plan,
        diagnostics,
    })
}

pub fn prepare_module_for_llvm_codegen(
    module: &ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
    entry_names: &[String],
) -> Result<PreparedLlvmProgram, CoreError> {
    prepare_module_for_llvm_codegen_with_source_map(module, target, profile, entry_names, None)
}

pub fn prepare_module_for_llvm_codegen_with_source_map(
    module: &ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
    entry_names: &[String],
    source_map: Option<&SourceMap>,
) -> Result<PreparedLlvmProgram, CoreError> {
    let program = prepare_module_for_codegen_with_source_map(module, target, profile, source_map)?;
    let raw_entry_defs = collect_top_level_llvm_defined_functions(module, target, profile);
    let mut reachable_set = BTreeSet::new();
    let mut resolved_entries = BTreeMap::new();
    for entry in entry_names {
        let resolved = match resolve_hir_entry_name(module, &program.hir_module, entry.as_str()) {
            Ok(resolved) => resolved,
            Err(_) if raw_entry_defs.contains(entry) => continue,
            Err(err) => return Err(err),
        };
        resolved_entries.insert(entry.clone(), resolved.clone());
        for name in collect_reachable_functions(&program.hir_module, resolved.as_str()) {
            reachable_set.insert(name.clone());
            if let Some(sep) = find_mangled_signature_separator(name.as_str()) {
                reachable_set.insert(String::from(&name[..sep]));
            }
        }
    }
    let pre_codegen_diags = passes::codegen_precheck::precheck_llvm_codegen(
        &program.types,
        &program.hir_module,
        &reachable_set,
    );
    if pre_codegen_diags
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostic::Severity::Error))
    {
        return Err(CoreError::from_diagnostics(pre_codegen_diags));
    }
    Ok(PreparedLlvmProgram {
        program,
        reachable_set,
        resolved_entries,
    })
}

fn resolve_hir_entry_name(
    module: &ast::Module,
    hir_module: &crate::hir::HirModule,
    entry: &str,
) -> Result<String, CoreError> {
    let mut function_map: BTreeMap<String, &crate::hir::HirFunction> = BTreeMap::new();
    for f in &hir_module.functions {
        function_map.insert(f.name.clone(), f);
    }
    if function_map.contains_key(entry) {
        return Ok(String::from(entry));
    }
    if let Some(found) = function_map
        .keys()
        .find(|name| {
            name.starts_with(&format!("{}__", entry))
                || name.starts_with(&format!("{}::", entry))
                || name.ends_with(&format!("::{}", entry))
        })
        .cloned()
    {
        return Ok(found);
    }
    let mut sample = function_map.keys().take(6).cloned().collect::<Vec<_>>();
    if sample.is_empty() {
        sample.push(String::from("<none>"));
    }
    Err(CoreError::from_diagnostics(vec![
        Diagnostic::error_with_code(
            DiagnosticCode::Resolve(ResolveDiagnosticCode::EntryFunctionMissingOrAmbiguous),
            format!(
                "entry function '{}' was not found in lowered module (available: {})",
                entry,
                sample.join(", ")
            ),
            find_entry_directive_span(module, entry).unwrap_or_else(Span::dummy),
        ),
    ]))
}

fn find_entry_directive_span(module: &ast::Module, entry: &str) -> Option<Span> {
    module.root.items.iter().find_map(|stmt| match stmt {
        ast::Stmt::Directive(ast::Directive::Entry { name }) if name.name == entry => {
            Some(name.span)
        }
        _ => None,
    })
}

fn collect_top_level_llvm_defined_functions(
    module: &ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
) -> BTreeSet<String> {
    let mut names = Vec::new();
    for idx in crate::target_precheck::active_stmt_indices(&module.root, target, profile) {
        if let ast::Stmt::LlvmIr(block) = &module.root.items[idx] {
            crate::llvm_ir::collect_defined_functions_from_llvmir_block(block, &mut names);
        }
    }
    names.into_iter().collect()
}

fn find_mangled_signature_separator(name: &str) -> Option<usize> {
    let bytes = name.as_bytes();
    if bytes.len() < 3 {
        return None;
    }
    for i in 1..(bytes.len() - 1) {
        if bytes[i] == b'_' && bytes[i + 1] == b'_' {
            return Some(i);
        }
    }
    None
}

fn collect_reachable_functions(module: &crate::hir::HirModule, entry: &str) -> Vec<String> {
    let mut function_map: BTreeMap<String, &crate::hir::HirFunction> = BTreeMap::new();
    for f in &module.functions {
        function_map.insert(f.name.clone(), f);
    }
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut stack = Vec::new();
    stack.push(String::from(entry));
    while let Some(name) = stack.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        let Some(func) = function_map.get(name.as_str()) else {
            continue;
        };
        collect_called_functions_from_body(&func.body, &mut stack);
    }
    visited.into_iter().collect()
}

fn collect_called_functions_from_body(body: &crate::hir::HirBody, stack: &mut Vec<String>) {
    match body {
        crate::hir::HirBody::Block(block) => collect_called_functions_from_block(block, stack),
        crate::hir::HirBody::Wasm(_) => {}
        crate::hir::HirBody::LlvmIr(block) => {
            for line in &block.lines {
                // LLVM IR における call @name(...) または call void @name(...) などのパターンを最低限拾う。
                // 実際には codegen_llvm.rs 側の parse_llvm_call_requirement と同等のロジックを期待。
                // 簡略化して "@名前(" 形式を抽出する。
                let mut s = line.as_str();
                while let Some(at_idx) = s.find('@') {
                    let after_at = &s[at_idx + 1..];
                    if let Some(open_idx) = after_at.find('(') {
                        let mut name = after_at[..open_idx].trim();
                        // クォートされている場合は外す
                        if name.starts_with('"') && name.ends_with('"') && name.len() >= 2 {
                            name = &name[1..name.len() - 1];
                        }
                        if !name.is_empty() {
                            stack.push(String::from(name));
                        }
                        s = &after_at[open_idx + 1..];
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

fn collect_called_functions_from_block(block: &crate::hir::HirBlock, stack: &mut Vec<String>) {
    for line in &block.lines {
        collect_called_functions_from_expr(&line.expr, stack);
    }
}

fn collect_called_functions_from_expr(expr: &crate::hir::HirExpr, stack: &mut Vec<String>) {
    match &expr.kind {
        crate::hir::HirExprKind::Call { callee, args } => {
            if let crate::hir::FuncRef::User(name, _, _) = callee {
                stack.push(name.clone());
            }
            for arg in args {
                collect_called_functions_from_expr(arg, stack);
            }
        }
        crate::hir::HirExprKind::CallIndirect { callee, args, .. } => {
            collect_called_functions_from_expr(callee, stack);
            for arg in args {
                collect_called_functions_from_expr(arg, stack);
            }
        }
        crate::hir::HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_called_functions_from_expr(cond, stack);
            collect_called_functions_from_expr(then_branch, stack);
            collect_called_functions_from_expr(else_branch, stack);
        }
        crate::hir::HirExprKind::While { cond, body } => {
            collect_called_functions_from_expr(cond, stack);
            collect_called_functions_from_expr(body, stack);
        }
        crate::hir::HirExprKind::Match { scrutinee, arms } => {
            collect_called_functions_from_expr(scrutinee, stack);
            for arm in arms {
                collect_called_functions_from_expr(&arm.body, stack);
            }
        }
        crate::hir::HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(payload) = payload {
                collect_called_functions_from_expr(payload, stack);
            }
        }
        crate::hir::HirExprKind::StructConstruct { fields, .. }
        | crate::hir::HirExprKind::TupleConstruct { items: fields }
        | crate::hir::HirExprKind::Intrinsic { args: fields, .. } => {
            for field in fields {
                collect_called_functions_from_expr(field, stack);
            }
        }
        crate::hir::HirExprKind::Block(block) => {
            collect_called_functions_from_block(block, stack);
        }
        crate::hir::HirExprKind::Let { value, .. }
        | crate::hir::HirExprKind::Set { value, .. }
        | crate::hir::HirExprKind::AddrOf(value)
        | crate::hir::HirExprKind::Deref(value) => {
            collect_called_functions_from_expr(value, stack);
        }
        crate::hir::HirExprKind::LiteralI32(_)
        | crate::hir::HirExprKind::LiteralF32(_)
        | crate::hir::HirExprKind::LiteralBool(_)
        | crate::hir::HirExprKind::LiteralStr(_)
        | crate::hir::HirExprKind::Unit
        | crate::hir::HirExprKind::Var(_)
        | crate::hir::HirExprKind::FnValue(_)
        | crate::hir::HirExprKind::Drop { .. } => {}
    }
}

fn emit_wasm(
    types: &crate::types::TypeCtx,
    hir_module: &crate::hir::HirModule,
    mut diagnostics: Vec<Diagnostic>,
    include_wat_comments: bool,
) -> Result<CompilationArtifact, CoreError> {
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let cg = match codegen_wasm::generate_wasm(types, hir_module) {
        Ok(cg) => cg,
        Err(mut codegen_diags) => {
            diagnostics.append(&mut codegen_diags);
            return Err(CoreError::from_diagnostics(diagnostics));
        }
    };
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("wasm_codegen", stage_start);
    diagnostics.extend(cg.diagnostics);
    let Some(bytes) = cg.bytes else {
        return Err(CoreError::from_diagnostics(diagnostics));
    };

    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let mut validator = Validator::new();
    if let Err(err) = validator.validate_all(&bytes) {
        let err_msg = alloc::format!("invalid wasm generated: {}", err);
        let mut diag = Diagnostic::error_with_code(
            DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(
                WasmDiagnosticCode::ValidationFailed,
            )),
            err_msg.clone(),
            Span::dummy(),
        );
        if let Some(offset) = parse_wasm_error_offset(&err_msg) {
            if let Some(loc) = locate_wasm_function_at_offset(&bytes, offset) {
                let near_name = hir_module
                    .functions
                    .get(loc.defined_func_index as usize)
                    .map(|f| f.name.as_str())
                    .unwrap_or("<unknown>");
                diag.notes.push(alloc::format!(
                    "validation failed near function body: func_index={}, defined_func_index={}, name={}, body_range=0x{:x}..0x{:x}",
                        loc.func_index, loc.defined_func_index, near_name, loc.body_start, loc.body_end
                ));
            }
        }
        diagnostics.push(diag);
        return Err(CoreError::from_diagnostics(diagnostics));
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("wasm_validate", stage_start);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let wat_comments = if include_wat_comments {
        build_wat_comments(types, hir_module)
    } else {
        String::new()
    };
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("wat_comments", stage_start);
    Ok(CompilationArtifact {
        wasm: bytes,
        wat_comments,
    })
}

#[derive(Debug, Clone, Copy)]
struct WasmFuncLocation {
    func_index: u32,
    defined_func_index: u32,
    body_start: usize,
    body_end: usize,
}

fn parse_wasm_error_offset(message: &str) -> Option<usize> {
    let marker = "offset 0x";
    let start = message.find(marker)? + marker.len();
    let hex = message[start..]
        .chars()
        .take_while(|c| c.is_ascii_hexdigit())
        .collect::<String>();
    if hex.is_empty() {
        return None;
    }
    usize::from_str_radix(&hex, 16).ok()
}

fn locate_wasm_function_at_offset(bytes: &[u8], offset: usize) -> Option<WasmFuncLocation> {
    let mut imported_func_count: u32 = 0;
    let mut defined_func_index: u32 = 0;
    for payload in Parser::new(0).parse_all(bytes) {
        let Ok(payload) = payload else {
            return None;
        };
        match payload {
            Payload::ImportSection(reader) => {
                for imp in reader {
                    let Ok(imp) = imp else {
                        return None;
                    };
                    match imp {
                        Imports::Single(_, import) => {
                            if matches!(import.ty, TypeRef::Func(_) | TypeRef::FuncExact(_)) {
                                imported_func_count += 1;
                            }
                        }
                        Imports::Compact1 { items, .. } => {
                            for item in items {
                                let Ok(item) = item else {
                                    return None;
                                };
                                if matches!(item.ty, TypeRef::Func(_) | TypeRef::FuncExact(_)) {
                                    imported_func_count += 1;
                                }
                            }
                        }
                        Imports::Compact2 { ty, names, .. } => {
                            if matches!(ty, TypeRef::Func(_) | TypeRef::FuncExact(_)) {
                                imported_func_count += names.count();
                            }
                        }
                    }
                }
            }
            Payload::CodeSectionEntry(body) => {
                let range = body.range();
                if offset >= range.start && offset < range.end {
                    return Some(WasmFuncLocation {
                        func_index: imported_func_count + defined_func_index,
                        defined_func_index,
                        body_start: range.start,
                        body_end: range.end,
                    });
                }
                defined_func_index += 1;
            }
            _ => {}
        }
    }
    None
}

/// WAT 先頭に付与するための補助情報を生成する。
///
/// 含める情報:
/// - 関数名
/// - 関数シグネチャ
/// - 引数名と型
/// - `let` で導入されたローカル変数名と型
fn build_wat_comments(types: &crate::types::TypeCtx, module: &crate::hir::HirModule) -> String {
    let mut out = String::new();
    out.push_str("NEPL WAT debug info\n");
    for func in &module.functions {
        out.push_str(&format!(
            "func {} : {}\n",
            func.name,
            types.type_to_string(func.func_ty)
        ));
        if !func.params.is_empty() {
            out.push_str("  params:\n");
            for p in &func.params {
                out.push_str(&format!(
                    "    {} : {}\n",
                    p.name,
                    types.type_to_string(p.ty)
                ));
            }
        }
        let mut locals: BTreeMap<String, crate::types::TypeId> = BTreeMap::new();
        if let crate::hir::HirBody::Block(block) = &func.body {
            collect_block_locals(block, &mut locals);
        }
        if !locals.is_empty() {
            out.push_str("  locals:\n");
            for (name, ty) in locals {
                out.push_str(&format!("    {} : {}\n", name, types.type_to_string(ty)));
            }
        }
    }
    out
}

fn collect_block_locals(
    block: &crate::hir::HirBlock,
    locals: &mut BTreeMap<String, crate::types::TypeId>,
) {
    for line in &block.lines {
        collect_expr_locals(&line.expr, locals);
    }
}

fn collect_expr_locals(
    expr: &crate::hir::HirExpr,
    locals: &mut BTreeMap<String, crate::types::TypeId>,
) {
    match &expr.kind {
        crate::hir::HirExprKind::Let { name, value, .. } => {
            locals.entry(name.clone()).or_insert(value.ty);
            collect_expr_locals(value, locals);
        }
        crate::hir::HirExprKind::Set { value, .. } => {
            collect_expr_locals(value, locals);
        }
        crate::hir::HirExprKind::Call { args, .. } => {
            for arg in args {
                collect_expr_locals(arg, locals);
            }
        }
        crate::hir::HirExprKind::CallIndirect { callee, args, .. } => {
            collect_expr_locals(callee, locals);
            for arg in args {
                collect_expr_locals(arg, locals);
            }
        }
        crate::hir::HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_expr_locals(cond, locals);
            collect_expr_locals(then_branch, locals);
            collect_expr_locals(else_branch, locals);
        }
        crate::hir::HirExprKind::While { cond, body } => {
            collect_expr_locals(cond, locals);
            collect_expr_locals(body, locals);
        }
        crate::hir::HirExprKind::Match { scrutinee, arms } => {
            collect_expr_locals(scrutinee, locals);
            for arm in arms {
                collect_expr_locals(&arm.body, locals);
            }
        }
        crate::hir::HirExprKind::StructConstruct { fields, .. } => {
            for f in fields {
                collect_expr_locals(f, locals);
            }
        }
        crate::hir::HirExprKind::TupleConstruct { items } => {
            for item in items {
                collect_expr_locals(item, locals);
            }
        }
        crate::hir::HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(p) = payload {
                collect_expr_locals(p, locals);
            }
        }
        crate::hir::HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                collect_expr_locals(arg, locals);
            }
        }
        crate::hir::HirExprKind::AddrOf(inner) | crate::hir::HirExprKind::Deref(inner) => {
            collect_expr_locals(inner, locals);
        }
        crate::hir::HirExprKind::Block(block) => {
            collect_block_locals(block, locals);
        }
        crate::hir::HirExprKind::Var(_)
        | crate::hir::HirExprKind::FnValue(_)
        | crate::hir::HirExprKind::LiteralI32(_)
        | crate::hir::HirExprKind::LiteralF32(_)
        | crate::hir::HirExprKind::LiteralBool(_)
        | crate::hir::HirExprKind::LiteralStr(_)
        | crate::hir::HirExprKind::Unit
        | crate::hir::HirExprKind::Drop { .. } => {}
    }
}

fn resolve_target(
    module: &ast::Module,
    options: CompileOptions,
) -> Result<CompileTarget, CoreError> {
    let mut found: Option<(CompileTarget, Span)> = None;
    let mut diags = Vec::new();
    let mut saw_target_directive = false;
    // First, check explicit module-level directives parsed into module.directives
    for d in &module.directives {
        if let ast::Directive::Target { target, span } = d {
            saw_target_directive = true;
            let parsed = parse_target_name(target.as_str());
            if let Some(t) = parsed {
                if let Some((_, prev_span)) = found {
                    diags.push(
                        Diagnostic::error_with_code(
                            DiagnosticCode::Loader(LoaderDiagnosticCode::TargetMultipleDirective),
                            "multiple #target directives are not allowed",
                            *span,
                        )
                        .with_secondary_label(prev_span, Some("previous #target here".into())),
                    );
                } else {
                    found = Some((t, *span));
                }
            } else {
                diags.push(Diagnostic::error_with_code(
                    DiagnosticCode::Loader(LoaderDiagnosticCode::TargetUnknown),
                    "unknown target in #target",
                    *span,
                ));
            }
        }
    }

    // Fallback: some parsers/merging steps may leave a file-scoped #target as a top-level
    // statement rather than in module.directives; inspect root items as a safeguard.
    if !saw_target_directive {
        for it in &module.root.items {
            if let ast::Stmt::Directive(ast::Directive::Target { target, span }) = it {
                let parsed = parse_target_name(target.as_str());
                if let Some(t) = parsed {
                    if let Some((_, prev_span)) = found {
                        diags.push(
                            Diagnostic::error_with_code(
                                DiagnosticCode::Loader(
                                    LoaderDiagnosticCode::TargetMultipleDirective,
                                ),
                                "multiple #target directives are not allowed",
                                *span,
                            )
                            .with_secondary_label(prev_span, Some("previous #target here".into())),
                        );
                    } else {
                        found = Some((t, *span));
                    }
                } else {
                    diags.push(Diagnostic::error_with_code(
                        DiagnosticCode::Loader(LoaderDiagnosticCode::TargetUnknown),
                        "unknown target in #target",
                        *span,
                    ));
                }
            }
        }
    }
    if !diags.is_empty() {
        return Err(CoreError::from_diagnostics(diags));
    }
    if let Some(t) = options.target {
        return Ok(t);
    }
    Ok(found.map(|(t, _)| t).unwrap_or(CompileTarget::Wasm))
}

fn parse_target_name(name: &str) -> Option<CompileTarget> {
    match name {
        "wasm" | "core" => Some(CompileTarget::Wasm),
        "wasi" | "std" => Some(CompileTarget::Wasi),
        "wasix" => Some(CompileTarget::Wasix),
        "llvm" => Some(CompileTarget::Llvm),
        _ => None,
    }
}
