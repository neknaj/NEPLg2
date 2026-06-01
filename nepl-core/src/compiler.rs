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
    /// ソース側の条件付きコンパイルで使う既定 profile。
    ///
    /// コンパイラ自身を release build にしても、ユーザーソースの
    /// `#if[profile=...]` が暗黙に release へ切り替わるべきではない。
    /// profile を明示しない場合は debug を選び、実行環境や配布 artifact の
    /// build mode と source semantics を分離する。
    pub fn default_source_profile() -> Self {
        BuildProfile::Debug
    }

    /// コンパイラ実行ファイルそのものの Rust build profile を返す。
    ///
    /// これは診断や内部観測用であり、ソース側の profile 既定値には使わない。
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
    /// 依存側の型検査へ渡せる `.neplmeta` 相当の public interface artifact。
    ///
    /// この artifact は typed public signature と compile context header だけを保持し、
    /// `TypeId`、`Span`、`SourceMap`、typed HIR、Resource IR body を含まない。
    /// 実際に依存 module の再 typecheck を省く処理は後続 checkpoint で、この安定
    /// envelope を loader / typecheck environment へ注入する形で追加する。
    pub nepl_meta_artifact: crate::artifact::NeplMetaArtifact,
    /// Resource summary proof artifact を作るための compile-context header。
    ///
    /// 実際の snapshot payload は `ResourceSummaryValueCache` が保持するため、この成果物には
    /// header だけを載せる。Web / CLI 側はこの header と現在の cache snapshot を組み合わせて、
    /// `.neplproof` 相当の in-memory artifact を作る。
    pub resource_summary_proof_header: Option<crate::resource::ResourceSummaryProofArtifactHeader>,
}

/// Resource summary proof artifact を compile pipeline へ接続するための入力。
///
/// `.neplproof` の expected header は typecheck 後に確定する typed public signature と
/// `ResourceSummaryCacheNamespaceKey` を必要とするため、Web / CLI 側で同じ hash を
/// 再実装してはならない。この options は host が持つ optional な artifact と stdlib content
/// hash だけを渡し、target/profile/source-capability/private-effect policy は core compiler が
/// canonical な形で組み立てる。
#[derive(Debug, Clone, Copy)]
pub struct ResourceSummaryProofArtifactCacheOptions<'a> {
    pub preseed_artifact: Option<&'a crate::resource::ResourceSummaryProofArtifact>,
    pub stdlib_content_hash: Option<u64>,
}

impl<'a> ResourceSummaryProofArtifactCacheOptions<'a> {
    pub fn none() -> Self {
        Self {
            preseed_artifact: None,
            stdlib_content_hash: None,
        }
    }

    pub fn preseed(
        artifact: &'a crate::resource::ResourceSummaryProofArtifact,
        stdlib_content_hash: Option<u64>,
    ) -> Self {
        Self {
            preseed_artifact: Some(artifact),
            stdlib_content_hash,
        }
    }
}

impl<'a> Default for ResourceSummaryProofArtifactCacheOptions<'a> {
    fn default() -> Self {
        Self::none()
    }
}

/// compile pipeline へ渡す public interface artifact 群。
///
/// `.neplmeta` は依存先の公開 surface を arena-independent に保持する。一方で、
/// dependency public surface hash と root module surface は Resource proof namespace や
/// artifact header の invalidation 境界にも使う。これらを同じ入力としてまとめることで、
/// loader / Web session 側は body skip の有無にかかわらず同じ authority を core に渡せる。
#[derive(Debug, Clone, Copy)]
pub struct PublicInterfaceArtifactInputs<'a> {
    pub dependency_public_surface_hash: Option<u64>,
    pub module_surface: Option<&'a crate::artifact::NeplMetaModuleSurface>,
    pub materialized_public_surfaces: &'a [crate::typecheck::MaterializedPublicSurfaceInput],
    /// direct call 用 `.neplobj` body fragment が利用可能であることを示す key。
    ///
    /// この slice は単なる metadata cache hit ではなく、caller が対応する backend body
    /// fragment も同じ compile に接続できる場合だけ渡す。key だけで body-missing を消すと
    /// wasm / LLVM codegen が未知 symbol を受け取るため、現在の `CompilerSession` は空 slice を
    /// 渡して fail-closed の source fallback を維持する。
    pub nepl_obj_direct_call_keys: &'a [crate::artifact::NeplObjDirectCallKey],
}

impl<'a> PublicInterfaceArtifactInputs<'a> {
    pub fn none() -> Self {
        Self {
            dependency_public_surface_hash: None,
            module_surface: None,
            materialized_public_surfaces: &[],
            nepl_obj_direct_call_keys: &[],
        }
    }

    pub fn new(
        dependency_public_surface_hash: Option<u64>,
        module_surface: Option<&'a crate::artifact::NeplMetaModuleSurface>,
        materialized_public_surfaces: &'a [crate::typecheck::MaterializedPublicSurfaceInput],
    ) -> Self {
        Self {
            dependency_public_surface_hash,
            module_surface,
            materialized_public_surfaces,
            nepl_obj_direct_call_keys: &[],
        }
    }

    pub fn with_nepl_obj_direct_call_keys(
        dependency_public_surface_hash: Option<u64>,
        module_surface: Option<&'a crate::artifact::NeplMetaModuleSurface>,
        materialized_public_surfaces: &'a [crate::typecheck::MaterializedPublicSurfaceInput],
        nepl_obj_direct_call_keys: &'a [crate::artifact::NeplObjDirectCallKey],
    ) -> Self {
        Self {
            dependency_public_surface_hash,
            module_surface,
            materialized_public_surfaces,
            nepl_obj_direct_call_keys,
        }
    }
}

impl<'a> Default for PublicInterfaceArtifactInputs<'a> {
    fn default() -> Self {
        Self::none()
    }
}

/// コンパイル pipeline の 1 stage にかかった時間。
///
/// Web / Node の same-session 性能測定では、標準エラーへ出す native timing だけでは
/// compiled-output cache miss 後の支配項を継続観測できない。この構造体は呼び出し元が
/// 提供した単調時刻に基づき、typecheck / Resource IR / codegen などの粗い stage を
/// JSON へ運ぶための軽量な測定値である。
#[derive(Debug, Clone)]
pub struct CompileStageTiming {
    pub stage: &'static str,
    pub elapsed_ms: f64,
}

/// 1 回の compile 呼び出しで記録された stage timing。
///
/// 値は最適化判定用の観測情報であり、診断や cache key には使わない。通常実行で重い
/// per-op tracing を常時有効化しないため、stage 名と elapsed milliseconds の配列だけを
/// 保持する。
#[derive(Debug, Clone, Default)]
pub struct CompileStageTimings {
    stages: Vec<CompileStageTiming>,
}

pub type CompileStageNow = fn() -> f64;

impl CompileStageTimings {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn record(&mut self, stage: &'static str, elapsed_ms: f64) {
        let elapsed_ms = if elapsed_ms.is_finite() && elapsed_ms >= 0.0 {
            elapsed_ms
        } else {
            0.0
        };
        self.stages.push(CompileStageTiming { stage, elapsed_ms });
    }

    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &CompileStageTiming> {
        self.stages.iter()
    }

    pub fn to_json_array(&self) -> String {
        let mut out = String::from("[");
        for (idx, timing) in self.stages.iter().enumerate() {
            if idx > 0 {
                out.push(',');
            }
            out.push_str("{\"stage\":\"");
            out.push_str(timing.stage);
            out.push_str("\",\"elapsed_ms\":");
            out.push_str(&format!("{:.3}", timing.elapsed_ms));
            out.push('}');
        }
        out.push(']');
        out
    }
}

struct CompileStageRecorder<'a> {
    timings: Option<&'a mut CompileStageTimings>,
    now_ms: Option<CompileStageNow>,
}

impl<'a> CompileStageRecorder<'a> {
    fn disabled() -> Self {
        Self {
            timings: None,
            now_ms: None,
        }
    }

    fn enabled(timings: &'a mut CompileStageTimings, now_ms: CompileStageNow) -> Self {
        Self {
            timings: Some(timings),
            now_ms: Some(now_ms),
        }
    }

    fn start(&self) -> Option<f64> {
        self.now_ms.map(|now_ms| now_ms())
    }

    fn finish(&mut self, stage: &'static str, start_ms: Option<f64>) {
        let Some(start_ms) = start_ms else {
            return;
        };
        let Some(now_ms) = self.now_ms else {
            return;
        };
        let Some(timings) = self.timings.as_deref_mut() else {
            return;
        };
        timings.record(stage, now_ms() - start_ms);
    }
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
    compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash(
        module,
        source_map,
        options,
        artifact_options,
        None,
    )
}

pub fn compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash(
    module: ast::Module,
    source_map: Option<&SourceMap>,
    options: CompileOptions,
    artifact_options: CompilationArtifactOptions,
    dependency_public_surface_hash: Option<u64>,
) -> Result<CompilationArtifact, CoreError> {
    compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_and_resource_summary_value_cache(
        module,
        source_map,
        options,
        artifact_options,
        dependency_public_surface_hash,
        None,
    )
}

/// Resource summary value cache を明示的に受け取る artifact pipeline。
///
/// 通常の CLI / stateless compile API は cache を持たないため、従来の wrapper から
/// `None` を渡す。`CompilerSession` のように session 寿命を持つ呼び出し元だけが、
/// compiled-output cache miss の実 compile に限ってこの経路を使う。
/// 現段階の cache は stable mirror value を保存せず、保存候補の bypass 計測だけを行う。
pub fn compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_and_resource_summary_value_cache(
    module: ast::Module,
    source_map: Option<&SourceMap>,
    options: CompileOptions,
    artifact_options: CompilationArtifactOptions,
    dependency_public_surface_hash: Option<u64>,
    resource_summary_value_cache: Option<&mut crate::resource::ResourceSummaryValueCache>,
) -> Result<CompilationArtifact, CoreError> {
    let mut stage_recorder = CompileStageRecorder::disabled();
    compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_and_resource_summary_value_cache_internal(
        module,
        source_map,
        options,
        artifact_options,
        PublicInterfaceArtifactInputs::new(dependency_public_surface_hash, None, &[]),
        resource_summary_value_cache,
        ResourceSummaryProofArtifactCacheOptions::none(),
        &mut stage_recorder,
    )
}

/// Resource summary value cache と `.neplproof` preseed artifact を受け取る artifact pipeline。
///
/// `.neplproof` artifact は compile 文脈と header が一致した場合だけ cache へ preseed される。
/// header が一致しない場合は payload を Resource proof authority として扱わず、通常の
/// Resource IR static check へ fail-closed に戻る。
pub fn compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_resource_summary_value_cache_and_neplproof(
    module: ast::Module,
    source_map: Option<&SourceMap>,
    options: CompileOptions,
    artifact_options: CompilationArtifactOptions,
    dependency_public_surface_hash: Option<u64>,
    resource_summary_value_cache: Option<&mut crate::resource::ResourceSummaryValueCache>,
    resource_summary_proof_options: ResourceSummaryProofArtifactCacheOptions<'_>,
) -> Result<CompilationArtifact, CoreError> {
    let mut stage_recorder = CompileStageRecorder::disabled();
    compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_and_resource_summary_value_cache_internal(
        module,
        source_map,
        options,
        artifact_options,
        PublicInterfaceArtifactInputs::new(dependency_public_surface_hash, None, &[]),
        resource_summary_value_cache,
        resource_summary_proof_options,
        &mut stage_recorder,
    )
}

pub fn compile_module_with_source_map_artifact_options_dependency_public_surface_hash_module_surface_resource_summary_value_cache_and_neplproof(
    module: ast::Module,
    source_map: Option<&SourceMap>,
    options: CompileOptions,
    artifact_options: CompilationArtifactOptions,
    dependency_public_surface_hash: Option<u64>,
    module_surface: Option<&crate::artifact::NeplMetaModuleSurface>,
    resource_summary_value_cache: Option<&mut crate::resource::ResourceSummaryValueCache>,
    resource_summary_proof_options: ResourceSummaryProofArtifactCacheOptions<'_>,
) -> Result<CompilationArtifact, CoreError> {
    let mut stage_recorder = CompileStageRecorder::disabled();
    compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_and_resource_summary_value_cache_internal(
        module,
        source_map,
        options,
        artifact_options,
        PublicInterfaceArtifactInputs::new(dependency_public_surface_hash, module_surface, &[]),
        resource_summary_value_cache,
        resource_summary_proof_options,
        &mut stage_recorder,
    )
}

/// `.neplmeta` 由来の public interface artifact を含める非計測 pipeline。
///
/// stage timing を必要としない CLI / Web 呼び出し元でも、body skip 用の
/// `MaterializedPublicSurfaceInput` と通常の Resource summary / `.neplproof` 入力を
/// 同じ core boundary に通す。実行 body が必要な materialized callable に到達した場合は、
/// 呼び出し元が source fallback へ戻る。
pub fn compile_module_with_source_map_artifact_options_public_interface_artifacts_resource_summary_value_cache_and_neplproof(
    module: ast::Module,
    source_map: Option<&SourceMap>,
    options: CompileOptions,
    artifact_options: CompilationArtifactOptions,
    public_interface_artifacts: PublicInterfaceArtifactInputs<'_>,
    resource_summary_value_cache: Option<&mut crate::resource::ResourceSummaryValueCache>,
    resource_summary_proof_options: ResourceSummaryProofArtifactCacheOptions<'_>,
) -> Result<CompilationArtifact, CoreError> {
    let mut stage_recorder = CompileStageRecorder::disabled();
    compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_and_resource_summary_value_cache_internal(
        module,
        source_map,
        options,
        artifact_options,
        public_interface_artifacts,
        resource_summary_value_cache,
        resource_summary_proof_options,
        &mut stage_recorder,
    )
}

/// `.neplmeta` 由来の public surface を typecheck 入力に含める artifact pipeline。
///
/// この経路は dependency body skip のための core 側 boundary である。materialized callable が
/// 実際の codegen 入力として必要になった場合は、`.neplobj` / code fragment artifact ができるまで
/// caller が source fallback へ戻る必要がある。
pub fn compile_module_with_source_map_artifact_options_public_interface_artifacts_resource_summary_value_cache_neplproof_and_stage_timings(
    module: ast::Module,
    source_map: Option<&SourceMap>,
    options: CompileOptions,
    artifact_options: CompilationArtifactOptions,
    public_interface_artifacts: PublicInterfaceArtifactInputs<'_>,
    resource_summary_value_cache: Option<&mut crate::resource::ResourceSummaryValueCache>,
    resource_summary_proof_options: ResourceSummaryProofArtifactCacheOptions<'_>,
    stage_timings: &mut CompileStageTimings,
    now_ms: CompileStageNow,
) -> Result<CompilationArtifact, CoreError> {
    let mut stage_recorder = CompileStageRecorder::enabled(stage_timings, now_ms);
    compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_and_resource_summary_value_cache_internal(
        module,
        source_map,
        options,
        artifact_options,
        public_interface_artifacts,
        resource_summary_value_cache,
        resource_summary_proof_options,
        &mut stage_recorder,
    )
}

/// Resource summary cache と stage timing collector を同時に受け取る artifact pipeline。
///
/// `CompilerSession` のような長寿命呼び出し元は、compiled-output cache miss の実 compile
/// だけを測定し、結果を Node / Web の JSON timing へ載せる。この関数は測定値を診断や
/// cache key には使わず、既存 pipeline と同じ静的検査を実行する。
pub fn compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_resource_summary_value_cache_and_stage_timings(
    module: ast::Module,
    source_map: Option<&SourceMap>,
    options: CompileOptions,
    artifact_options: CompilationArtifactOptions,
    dependency_public_surface_hash: Option<u64>,
    resource_summary_value_cache: Option<&mut crate::resource::ResourceSummaryValueCache>,
    stage_timings: &mut CompileStageTimings,
    now_ms: CompileStageNow,
) -> Result<CompilationArtifact, CoreError> {
    let mut stage_recorder = CompileStageRecorder::enabled(stage_timings, now_ms);
    compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_and_resource_summary_value_cache_internal(
        module,
        source_map,
        options,
        artifact_options,
        PublicInterfaceArtifactInputs::new(dependency_public_surface_hash, None, &[]),
        resource_summary_value_cache,
        ResourceSummaryProofArtifactCacheOptions::none(),
        &mut stage_recorder,
    )
}

/// Resource summary cache / `.neplproof` preseed / stage timing を同時に受け取る pipeline。
///
/// Web `CompilerSession` は compile stage timing と same-session proof artifact の両方を
/// 観測するため、この wrapper で core 側の canonical header 生成と timing collection を
/// 同じ経路に通す。
pub fn compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_resource_summary_value_cache_neplproof_and_stage_timings(
    module: ast::Module,
    source_map: Option<&SourceMap>,
    options: CompileOptions,
    artifact_options: CompilationArtifactOptions,
    dependency_public_surface_hash: Option<u64>,
    resource_summary_value_cache: Option<&mut crate::resource::ResourceSummaryValueCache>,
    resource_summary_proof_options: ResourceSummaryProofArtifactCacheOptions<'_>,
    stage_timings: &mut CompileStageTimings,
    now_ms: CompileStageNow,
) -> Result<CompilationArtifact, CoreError> {
    let mut stage_recorder = CompileStageRecorder::enabled(stage_timings, now_ms);
    compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_and_resource_summary_value_cache_internal(
        module,
        source_map,
        options,
        artifact_options,
        PublicInterfaceArtifactInputs::new(dependency_public_surface_hash, None, &[]),
        resource_summary_value_cache,
        resource_summary_proof_options,
        &mut stage_recorder,
    )
}

pub fn compile_module_with_source_map_artifact_options_dependency_public_surface_hash_module_surface_resource_summary_value_cache_neplproof_and_stage_timings(
    module: ast::Module,
    source_map: Option<&SourceMap>,
    options: CompileOptions,
    artifact_options: CompilationArtifactOptions,
    dependency_public_surface_hash: Option<u64>,
    module_surface: Option<&crate::artifact::NeplMetaModuleSurface>,
    resource_summary_value_cache: Option<&mut crate::resource::ResourceSummaryValueCache>,
    resource_summary_proof_options: ResourceSummaryProofArtifactCacheOptions<'_>,
    stage_timings: &mut CompileStageTimings,
    now_ms: CompileStageNow,
) -> Result<CompilationArtifact, CoreError> {
    let mut stage_recorder = CompileStageRecorder::enabled(stage_timings, now_ms);
    compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_and_resource_summary_value_cache_internal(
        module,
        source_map,
        options,
        artifact_options,
        PublicInterfaceArtifactInputs::new(dependency_public_surface_hash, module_surface, &[]),
        resource_summary_value_cache,
        resource_summary_proof_options,
        &mut stage_recorder,
    )
}

fn compile_module_with_source_map_artifact_options_and_dependency_public_surface_hash_and_resource_summary_value_cache_internal(
    module: ast::Module,
    source_map: Option<&SourceMap>,
    options: CompileOptions,
    artifact_options: CompilationArtifactOptions,
    public_interface_artifacts: PublicInterfaceArtifactInputs<'_>,
    resource_summary_value_cache: Option<&mut crate::resource::ResourceSummaryValueCache>,
    resource_summary_proof_options: ResourceSummaryProofArtifactCacheOptions<'_>,
    stage_recorder: &mut CompileStageRecorder<'_>,
) -> Result<CompilationArtifact, CoreError> {
    // loader が計算した dependency public surface hash を artifact pipeline へ渡す。
    // この入力は Resource summary namespace key だけに使い、汎用 `CompileOptions`
    // へは入れない。通常の CLI / test compile と、session-backed bundled stdlib compile
    // で必要な cache invalidation 境界が異なるためである。
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
    let profile = options
        .profile
        .unwrap_or(BuildProfile::default_source_profile());
    let prepared = prepare_module_for_codegen_with_source_map_dependency_public_surface_hash_and_resource_summary_value_cache_internal(
        &module,
        target,
        profile,
        source_map,
        public_interface_artifacts,
        resource_summary_value_cache,
        resource_summary_proof_options,
        stage_recorder,
    )?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let pre_codegen_diags =
        passes::codegen_precheck::precheck_wasm_codegen(&prepared.types, &prepared.hir_module);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("codegen_precheck", stage_start);
    stage_recorder.finish("codegen_precheck", stage_start_ms);
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
        prepared.nepl_meta_artifact,
        Some(prepared.resource_summary_proof_header),
        stage_recorder,
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
    let profile = options
        .profile
        .unwrap_or(BuildProfile::default_source_profile());
    prepare_module_for_codegen_with_source_map_and_dependency_public_surface_hash(
        &module, target, profile, source_map, None,
    )?;
    Ok(())
}

/// 依存 module 用の `.neplmeta` interface artifact だけを生成する。
///
/// この経路は typecheck までで止まり、Resource IR 検査、drop 挿入、wasm codegen は実行しない。
/// `.neplmeta` は依存側の名前解決・型検査へ渡す公開 interface を保存する artifact であり、
/// body の安全性 proof や実行コードの authority ではないためである。source identity は
/// loader が target module 単位で計算した値を受け取り、dependency edge probe と同じ
/// invalidation 境界に固定する。
pub fn compile_nepl_meta_artifact_with_source_identity(
    module: ast::Module,
    source_map: Option<&SourceMap>,
    options: CompileOptions,
    dependency_public_surface_hash: Option<u64>,
    module_surface: Option<&crate::artifact::NeplMetaModuleSurface>,
    stdlib_content_hash: Option<u64>,
    source_key_hash: u64,
    source_capability_policy_set_hash: Option<u64>,
) -> Result<crate::artifact::NeplMetaArtifact, CoreError> {
    crate::log::set_verbose(options.verbose);
    let target = resolve_target(&module, options)?;
    if matches!(target, CompileTarget::Llvm) {
        let mut diags = Vec::new();
        diags.push(Diagnostic::error_with_code(
            DiagnosticCode::Backend(BackendDiagnosticCode::TargetRequiresCli),
            "llvm target is CLI-only and does not produce wasm-session .neplmeta artifacts",
            Span::dummy(),
        ));
        return Err(CoreError::from_diagnostics(diags));
    }
    let profile = options
        .profile
        .unwrap_or(BuildProfile::default_source_profile());
    let precheck_diags =
        crate::target_precheck::precheck_module_before_codegen(&module, target, profile);
    if precheck_diags
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostic::Severity::Error))
    {
        return Err(CoreError::from_diagnostics(precheck_diags));
    }
    let typed = run_typecheck(&module, target, profile, source_map, &[])?;
    Ok(
        crate::artifact::NeplMetaArtifact::from_public_surface_and_module_surface_with_source_identity(
            target,
            profile,
            stdlib_content_hash,
            dependency_public_surface_hash,
            source_key_hash,
            source_capability_policy_set_hash,
            typed.public_signatures,
            module_surface.cloned(),
            typed.public_surface,
        ),
    )
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
    public_signatures: crate::typecheck::TypedPublicSignatureTable,
    public_surface: crate::typecheck::TypedPublicSurfaceTable,
    diagnostics: Vec<Diagnostic>,
}

pub struct PreparedProgram {
    pub types: crate::types::TypeCtx,
    pub hir_module: crate::hir::HirModule,
    pub public_signatures: crate::typecheck::TypedPublicSignatureTable,
    pub public_surface: crate::typecheck::TypedPublicSurfaceTable,
    pub nepl_meta_artifact: crate::artifact::NeplMetaArtifact,
    pub resource_summary_cache_namespace_key: ResourceSummaryCacheNamespaceKey,
    pub resource_summary_proof_header: crate::resource::ResourceSummaryProofArtifactHeader,
    pub resource_drop_elaboration_plan: crate::resource::ResourceDropElaborationPlan,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct PreparedLlvmProgram {
    pub program: PreparedProgram,
    pub reachable_set: BTreeSet<String>,
    pub resolved_entries: BTreeMap<String, String>,
}

/// Resource IR summary cache の module-level namespace key。
///
/// この key は `TypeId`、`Span`、`SourceMap`、typed HIR、Resource IR body を保持しない。
/// それらは compile session ごとの arena や source-map allocation に結び付くため、
/// 長寿命 cache value の key として直接保存すると stale hit の原因になる。
///
/// 現段階では、target/profile、typed public signature hash、任意の dependency
/// public surface hash から作る staging artifact である。実際に Resource IR
/// summary value を再利用する段階では、この namespace key に function body hash、
/// generic type-argument hash、source capability policy hash、summary kind/version を
/// 組み合わせた per-summary-value key を作る。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceSummaryCacheNamespaceKey {
    pub stable_hash: u64,
    pub typed_public_signature_hash: u64,
    pub dependency_public_surface_hash: Option<u64>,
    pub target: CompileTarget,
    pub profile: BuildProfile,
}

impl ResourceSummaryCacheNamespaceKey {
    pub fn new(
        target: CompileTarget,
        profile: BuildProfile,
        typed_public_signature_hash: u64,
        dependency_public_surface_hash: Option<u64>,
    ) -> Self {
        let stable_hash = resource_summary_cache_namespace_hash(
            target,
            profile,
            typed_public_signature_hash,
            dependency_public_surface_hash,
        );
        Self {
            stable_hash,
            typed_public_signature_hash,
            dependency_public_surface_hash,
            target,
            profile,
        }
    }

    /// 現在 compile の `.neplproof` expected header を作る。
    ///
    /// target/profile と resource summary namespace はこの key から取り出し、source
    /// capability policy set は `SourceMap` から集約する。private effect policy は
    /// compiler-owned version hash として常に `Some` にし、`None == None` による stale
    /// artifact の受け入れを避ける。
    pub fn resource_summary_proof_header(
        &self,
        source_map: Option<&SourceMap>,
        stdlib_content_hash: Option<u64>,
    ) -> crate::resource::ResourceSummaryProofArtifactHeader {
        crate::resource::ResourceSummaryProofArtifactHeader::new(
            resource_summary_proof_compiler_identity_hash(),
            resource_summary_proof_target_hash(self.target),
            resource_summary_proof_profile_hash(self.profile),
            stdlib_content_hash,
            self.dependency_public_surface_hash,
            self.stable_hash,
            resource_summary_source_capability_policy_set_hash(source_map),
            Some(resource_summary_private_effect_policy_hash()),
        )
    }
}

const RESOURCE_SUMMARY_CACHE_NAMESPACE_KEY_VERSION: &str =
    "neplg2-resource-summary-cache-namespace-v1";
const RESOURCE_SUMMARY_PROOF_HEADER_HASH_VERSION: &str = "neplg2-resource-summary-proof-header-v1";
const RESOURCE_SUMMARY_PROOF_COMPILER_IDENTITY_INPUT: &str = concat!(
    "neplg2-compiler:",
    env!("CARGO_PKG_VERSION"),
    ":resource-summary-proof-v1"
);
const RESOURCE_SUMMARY_PRIVATE_EFFECT_POLICY_VERSION: &str = "neplg2-private-effect-policy-v2";

fn resource_summary_cache_namespace_hash(
    target: CompileTarget,
    profile: BuildProfile,
    typed_public_signature_hash: u64,
    dependency_public_surface_hash: Option<u64>,
) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    resource_summary_cache_hash_str(&mut hash, RESOURCE_SUMMARY_CACHE_NAMESPACE_KEY_VERSION);
    resource_summary_cache_hash_str(&mut hash, resource_summary_cache_target_tag(target));
    resource_summary_cache_hash_str(&mut hash, resource_summary_cache_profile_tag(profile));
    resource_summary_cache_hash_u64(&mut hash, typed_public_signature_hash);
    match dependency_public_surface_hash {
        Some(value) => {
            resource_summary_cache_hash_u8(&mut hash, 1);
            resource_summary_cache_hash_u64(&mut hash, value);
        }
        None => resource_summary_cache_hash_u8(&mut hash, 0),
    }
    hash
}

fn resource_summary_cache_target_tag(target: CompileTarget) -> &'static str {
    match target {
        CompileTarget::Wasm => "wasm",
        CompileTarget::Wasi => "wasi",
        CompileTarget::Wasix => "wasix",
        CompileTarget::Llvm => "llvm",
    }
}

fn resource_summary_cache_profile_tag(profile: BuildProfile) -> &'static str {
    match profile {
        BuildProfile::Debug => "debug",
        BuildProfile::Release => "release",
    }
}

fn resource_summary_cache_hash_str(hash: &mut u64, value: &str) {
    resource_summary_cache_hash_bytes(hash, value.as_bytes());
    resource_summary_cache_hash_bytes(hash, &[0]);
}

fn resource_summary_cache_hash_u64(hash: &mut u64, value: u64) {
    resource_summary_cache_hash_bytes(hash, &value.to_le_bytes());
}

fn resource_summary_cache_hash_u8(hash: &mut u64, value: u8) {
    resource_summary_cache_hash_bytes(hash, &[value, 0xff]);
}

fn resource_summary_cache_hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

pub fn resource_summary_proof_compiler_identity_hash() -> u64 {
    resource_summary_proof_hash_tag("compiler", RESOURCE_SUMMARY_PROOF_COMPILER_IDENTITY_INPUT)
}

pub fn resource_summary_proof_target_hash(target: CompileTarget) -> u64 {
    resource_summary_proof_hash_tag("target", resource_summary_cache_target_tag(target))
}

pub fn resource_summary_proof_profile_hash(profile: BuildProfile) -> u64 {
    resource_summary_proof_hash_tag("profile", resource_summary_cache_profile_tag(profile))
}

pub fn resource_summary_private_effect_policy_hash() -> u64 {
    resource_summary_proof_hash_tag(
        "private-effect-policy",
        RESOURCE_SUMMARY_PRIVATE_EFFECT_POLICY_VERSION,
    )
}

pub fn resource_summary_source_capability_policy_set_hash(
    source_map: Option<&SourceMap>,
) -> Option<u64> {
    let source_map = source_map?;
    let mut hash = 0xcbf29ce484222325;
    resource_summary_cache_hash_str(&mut hash, RESOURCE_SUMMARY_PROOF_HEADER_HASH_VERSION);
    resource_summary_cache_hash_str(&mut hash, "source-capability-policy-set");
    for (file_id, path) in source_map.iter_paths() {
        let policy_hash = source_map.source_capability_policy_hash_for_file(file_id)?;
        resource_summary_cache_hash_str(&mut hash, path.as_str());
        resource_summary_cache_hash_u64(&mut hash, policy_hash);
    }
    Some(hash)
}

/// 1つの source file だけを対象にした capability policy set hash を作る。
///
/// dependency edge artifact の事前照合では、root compile 全体の `SourceMap` ではなく target
/// module file の source capability policy だけを boundary にする。dependency の公開面変更は
/// `dependency_public_surface_hash` で別に invalidation する。
pub fn resource_summary_source_capability_policy_set_hash_for_single_file(
    path: &str,
    policy_hash: u64,
) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    resource_summary_cache_hash_str(&mut hash, RESOURCE_SUMMARY_PROOF_HEADER_HASH_VERSION);
    resource_summary_cache_hash_str(&mut hash, "source-capability-policy-set");
    resource_summary_cache_hash_str(&mut hash, path);
    resource_summary_cache_hash_u64(&mut hash, policy_hash);
    hash
}

fn resource_summary_proof_hash_tag(domain: &str, value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    resource_summary_cache_hash_str(&mut hash, RESOURCE_SUMMARY_PROOF_HEADER_HASH_VERSION);
    resource_summary_cache_hash_str(&mut hash, domain);
    resource_summary_cache_hash_str(&mut hash, value);
    hash
}

fn resource_summary_value_cache_context(
    namespace_key: &ResourceSummaryCacheNamespaceKey,
    source_map: Option<&SourceMap>,
) -> Option<crate::resource::ResourceSummaryValueCacheContext> {
    let source_map = source_map?;
    let mut context =
        crate::resource::ResourceSummaryValueCacheContext::new(namespace_key.stable_hash);
    for (file_id, path) in source_map.iter_paths() {
        let policy_hash = source_map.source_capability_policy_hash_for_file(file_id)?;
        context.insert_source_policy_hash(file_id, policy_hash);
        let source = source_map.get(file_id)?;
        context.insert_source_policy_file(
            file_id,
            path.as_str(),
            source,
            source_map.capabilities(file_id),
        );
    }
    Some(context)
}

fn run_typecheck(
    module: &ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
    source_map: Option<&SourceMap>,
    materialized_public_surfaces: &[crate::typecheck::MaterializedPublicSurfaceInput],
) -> Result<TypedProgram, CoreError> {
    let tc = typecheck::typecheck_with_materialized_public_surfaces(
        module,
        target,
        profile,
        source_map,
        materialized_public_surfaces,
    );
    match tc.module {
        Some(m) => Ok(TypedProgram {
            types: tc.types,
            module: m,
            public_signatures: tc.public_signatures,
            public_surface: tc.public_surface,
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

const MATERIALIZED_CALLABLE_SYMBOL_PREFIX: &str = "neplmeta$";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MaterializedCodegenDependencyKind {
    DirectCall,
    FunctionValue,
    MemoizedFunctionValue,
}

impl MaterializedCodegenDependencyKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::DirectCall => "direct call",
            Self::FunctionValue => "function value",
            Self::MemoizedFunctionValue => "memoized function value",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MaterializedCodegenDependency {
    symbol: String,
    span: Span,
    kind: MaterializedCodegenDependencyKind,
}

fn materialized_codegen_dependency_diagnostics(
    hir_module: &crate::hir::HirModule,
    nepl_obj_direct_call_keys: &[crate::artifact::NeplObjDirectCallKey],
) -> Vec<Diagnostic> {
    let mut dependencies = Vec::new();
    for function in &hir_module.functions {
        collect_materialized_codegen_dependencies_in_body(&function.body, &mut dependencies);
    }
    dependencies
        .into_iter()
        .filter(|dependency| {
            !materialized_codegen_dependency_has_available_body(
                dependency,
                nepl_obj_direct_call_keys,
            )
        })
        .map(|dependency| {
            Diagnostic::error_with_code(
                DiagnosticCode::Backend(BackendDiagnosticCode::MaterializedFunctionBodyMissing),
                format!(
                    "materialized callable '{}' needs source fallback for {} because no code artifact is available yet",
                    dependency.symbol,
                    dependency.kind.as_str()
                ),
                dependency.span,
            )
        })
        .collect()
}

fn materialized_codegen_dependency_has_available_body(
    dependency: &MaterializedCodegenDependency,
    nepl_obj_direct_call_keys: &[crate::artifact::NeplObjDirectCallKey],
) -> bool {
    // `.neplobj` direct-call MVP は通常の直接呼び出しだけを解決する。
    // function value、indirect call、memoized function value は body fragment だけでは
    // identity / table / PrivateCache proof が不足するため、同じ symbol に見えても解決しない。
    if dependency.kind != MaterializedCodegenDependencyKind::DirectCall {
        return false;
    }
    nepl_obj_direct_call_keys
        .iter()
        .any(|key| key.materialized_symbol == dependency.symbol)
}

fn collect_materialized_codegen_dependencies_in_body(
    body: &crate::hir::HirBody,
    dependencies: &mut Vec<MaterializedCodegenDependency>,
) {
    if let crate::hir::HirBody::Block(block) = body {
        collect_materialized_codegen_dependencies_in_block(block, dependencies);
    }
}

fn collect_materialized_codegen_dependencies_in_block(
    block: &crate::hir::HirBlock,
    dependencies: &mut Vec<MaterializedCodegenDependency>,
) {
    for line in &block.lines {
        collect_materialized_codegen_dependencies_in_expr(&line.expr, dependencies);
    }
}

fn record_materialized_symbol(
    symbol: &str,
    span: Span,
    kind: MaterializedCodegenDependencyKind,
    dependencies: &mut Vec<MaterializedCodegenDependency>,
) {
    if symbol.starts_with(MATERIALIZED_CALLABLE_SYMBOL_PREFIX) {
        dependencies.push(MaterializedCodegenDependency {
            symbol: String::from(symbol),
            span,
            kind,
        });
    }
}

fn collect_materialized_codegen_dependencies_in_expr(
    expr: &crate::hir::HirExpr,
    dependencies: &mut Vec<MaterializedCodegenDependency>,
) {
    use crate::hir::{FuncRef, HirExprKind};

    match &expr.kind {
        HirExprKind::FnValue(identity) => {
            record_materialized_symbol(
                identity.symbol(),
                expr.span,
                MaterializedCodegenDependencyKind::FunctionValue,
                dependencies,
            );
        }
        HirExprKind::MemoizedFunctionValue(identity) => {
            record_materialized_symbol(
                identity.symbol(),
                expr.span,
                MaterializedCodegenDependencyKind::MemoizedFunctionValue,
                dependencies,
            );
        }
        HirExprKind::Call { callee, args } => {
            if let FuncRef::User(symbol, _, _) = callee {
                record_materialized_symbol(
                    symbol,
                    expr.span,
                    MaterializedCodegenDependencyKind::DirectCall,
                    dependencies,
                );
            }
            for arg in args {
                collect_materialized_codegen_dependencies_in_expr(arg, dependencies);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            collect_materialized_codegen_dependencies_in_expr(callee, dependencies);
            for arg in args {
                collect_materialized_codegen_dependencies_in_expr(arg, dependencies);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_materialized_codegen_dependencies_in_expr(cond, dependencies);
            collect_materialized_codegen_dependencies_in_expr(then_branch, dependencies);
            collect_materialized_codegen_dependencies_in_expr(else_branch, dependencies);
        }
        HirExprKind::While { cond, body } => {
            collect_materialized_codegen_dependencies_in_expr(cond, dependencies);
            collect_materialized_codegen_dependencies_in_expr(body, dependencies);
        }
        HirExprKind::Match { scrutinee, arms } => {
            collect_materialized_codegen_dependencies_in_expr(scrutinee, dependencies);
            for arm in arms {
                collect_materialized_codegen_dependencies_in_expr(&arm.body, dependencies);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(payload) = payload {
                collect_materialized_codegen_dependencies_in_expr(payload, dependencies);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for field in fields {
                collect_materialized_codegen_dependencies_in_expr(field, dependencies);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for item in items {
                collect_materialized_codegen_dependencies_in_expr(item, dependencies);
            }
        }
        HirExprKind::Block(block) => {
            collect_materialized_codegen_dependencies_in_block(block, dependencies);
        }
        HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
            collect_materialized_codegen_dependencies_in_expr(value, dependencies);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                collect_materialized_codegen_dependencies_in_expr(arg, dependencies);
            }
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            collect_materialized_codegen_dependencies_in_expr(inner, dependencies);
        }
        HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit
        | HirExprKind::Var(_)
        | HirExprKind::Drop { .. } => {}
    }
}

fn run_resource_static_check(
    hir_module: &crate::hir::HirModule,
    types: &crate::types::TypeCtx,
    diagnostics: &mut Vec<Diagnostic>,
    source_map: Option<&SourceMap>,
    mut resource_summary_value_cache: Option<&mut crate::resource::ResourceSummaryValueCache>,
    resource_summary_value_cache_context: Option<
        &crate::resource::ResourceSummaryValueCacheContext,
    >,
    stage_recorder: &mut CompileStageRecorder<'_>,
) -> Result<crate::resource::ResourceDropElaborationPlan, CoreError> {
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    run_resource_shadow_check(hir_module, types);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_shadow_check", stage_start);
    stage_recorder.finish("resource_static_shadow_check", stage_start_ms);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let resource = crate::resource::lower_hir_module(hir_module, types);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_lowering", stage_start);
    stage_recorder.finish("resource_static_lowering", stage_start_ms);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let lowering_coverage =
        crate::resource::compare_hir_resource_lowering_typed(hir_module, &resource, types);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_lowering_coverage", stage_start);
    stage_recorder.finish("resource_static_lowering_coverage", stage_start_ms);
    run_resource_lowering_coverage_gate(&lowering_coverage, diagnostics)?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let initialized_moves = match (
        resource_summary_value_cache.as_deref_mut(),
        resource_summary_value_cache_context,
    ) {
        (Some(cache), Some(context)) => {
            crate::resource::check_resource_initialized_moves_with_summary_cache(
                &resource, types, cache, context,
            )
        }
        (Some(_), None) | (None, _) => {
            crate::resource::check_resource_initialized_moves(&resource, types)
        }
    };
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_initialized_moves", stage_start);
    stage_recorder.finish("resource_static_initialized_moves", stage_start_ms);
    run_resource_cell_gate(&initialized_moves, diagnostics)?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let drop_elaboration_plan = run_resource_drop_elaboration_plan_gate(
        crate::resource::compute_resource_drop_elaboration_plan(&resource, &initialized_moves),
        diagnostics,
    )?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_drop_elaboration_plan", stage_start);
    stage_recorder.finish("resource_static_drop_elaboration_plan", stage_start_ms);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let borrow_lifetimes = crate::resource::check_resource_borrow_lifetimes(&resource, types);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_borrow_lifetimes", stage_start);
    stage_recorder.finish("resource_static_borrow_lifetimes", stage_start_ms);
    run_resource_borrow_lifetime_gate(&borrow_lifetimes, diagnostics)?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let effect_boundaries =
        crate::resource::check_resource_effect_boundaries_typed(&resource, types);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_effect_boundaries", stage_start);
    stage_recorder.finish("resource_static_effect_boundaries", stage_start_ms);
    run_resource_effect_boundary_gate(&effect_boundaries, diagnostics, source_map)?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let owner_obligations = match (
        resource_summary_value_cache.as_deref_mut(),
        resource_summary_value_cache_context,
    ) {
        (Some(cache), Some(context)) => {
            crate::resource::check_resource_owner_obligations_with_summary_cache(
                &resource, types, cache, context,
            )
        }
        (Some(_), None) | (None, _) => {
            crate::resource::check_resource_owner_obligations(&resource, types)
        }
    };
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_owner_obligations", stage_start);
    stage_recorder.finish("resource_static_owner_obligations", stage_start_ms);
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
        crate::resource::ResourceCheckDiagnostic::CollectionSlotRefuted {
            function,
            target,
            reason,
            span,
        } => Diagnostic::error_with_code(
            diagnostic.diagnostic_code(),
            format!(
                "resource ir collection slot lifecycle violation in function '{}': {:?} on {:?}",
                function, reason, target
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
    use crate::effects::{PrivateEffectRegion, PrivateEffectRegionId};
    use crate::resource::{
        BorrowState, CellState, OwnerState, Place, PrivateCacheOp, RawAddressAliasKind,
        RawAddressViewKind, RawMemoryOp, ResourceBorrowDiagnostic, ResourceBorrowOperation,
        ResourceCheckDiagnostic, ResourceCheckOperation, ResourceEffectBoundaryDiagnostic,
        ResourceEffectCallKind, ResourceId, ResourceOwnerDiagnostic, ResourceOwnerOperation,
        StorageId,
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

    fn private_cache_capabilities(operation: PrivateCacheOp, span: Span) -> SourceCapabilities {
        private_cache_capabilities_in_region(
            operation,
            PrivateEffectRegion::UnsealedIntrinsic,
            span,
        )
    }

    fn private_cache_capabilities_in_region(
        operation: PrivateCacheOp,
        region: PrivateEffectRegion,
        span: Span,
    ) -> SourceCapabilities {
        use_site_capabilities(SourceCapabilityUseSite::PrivateCacheBoundary {
            operation,
            region,
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

    fn test_hir_function(
        types: &mut crate::types::TypeCtx,
        name: &str,
        body: crate::hir::HirBody,
    ) -> crate::hir::HirFunction {
        let i32_ty = types.i32();
        crate::hir::HirFunction {
            doc: None,
            name: String::from(name),
            origin_name: String::from(name),
            func_ty: types.function(Vec::new(), Vec::new(), i32_ty, ast::Effect::Pure),
            params: Vec::new(),
            result: i32_ty,
            effect: ast::Effect::Pure,
            body,
            span: Span::dummy(),
        }
    }

    fn test_literal_body(types: &mut crate::types::TypeCtx, value: i32) -> crate::hir::HirBody {
        let i32_ty = types.i32();
        crate::hir::HirBody::Block(crate::hir::HirBlock {
            lines: vec![crate::hir::HirLine {
                expr: crate::hir::HirExpr {
                    ty: i32_ty,
                    kind: crate::hir::HirExprKind::LiteralI32(value),
                    span: Span::dummy(),
                },
                drop_result: false,
            }],
            ty: i32_ty,
            span: Span::dummy(),
        })
    }

    fn test_direct_call_body(
        types: &mut crate::types::TypeCtx,
        callee: &str,
    ) -> crate::hir::HirBody {
        let i32_ty = types.i32();
        crate::hir::HirBody::Block(crate::hir::HirBlock {
            lines: vec![crate::hir::HirLine {
                expr: crate::hir::HirExpr {
                    ty: i32_ty,
                    kind: crate::hir::HirExprKind::Call {
                        callee: crate::hir::FuncRef::User(String::from(callee), Vec::new(), None),
                        args: Vec::new(),
                    },
                    span: Span::dummy(),
                },
                drop_result: false,
            }],
            ty: i32_ty,
            span: Span::dummy(),
        })
    }

    fn test_module(functions: Vec<crate::hir::HirFunction>) -> crate::hir::HirModule {
        crate::hir::HirModule {
            functions,
            entry: Some(String::from("main")),
            externs: Vec::new(),
            string_literals: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
        }
    }

    fn parse_test_module_for_file(file_id: FileId, source: &str) -> ast::Module {
        let lex = lexer::lex(file_id, source);
        assert!(
            lex.diagnostics.is_empty(),
            "lexer diagnostics: {:?}",
            lex.diagnostics
        );
        let parsed = parser::parse_tokens(file_id, lex);
        assert!(
            parsed.diagnostics.is_empty(),
            "parser diagnostics: {:?}",
            parsed.diagnostics
        );
        parsed.module.expect("parser should produce a module")
    }

    fn parse_test_module(source: &str) -> ast::Module {
        parse_test_module_for_file(FileId(0), source)
    }

    fn prepared_resource_summary_cache_namespace_key(
        source: &str,
    ) -> ResourceSummaryCacheNamespaceKey {
        let module = parse_test_module(source);
        prepare_module_for_codegen(&module, CompileTarget::Wasm, BuildProfile::Debug)
            .expect("test module should pass prepare")
            .resource_summary_cache_namespace_key
    }

    fn prepared_resource_summary_cache_namespace_key_with_dependency_hash(
        source: &str,
        dependency_public_surface_hash: u64,
    ) -> ResourceSummaryCacheNamespaceKey {
        let module = parse_test_module(source);
        prepare_module_for_codegen_with_source_map_and_dependency_public_surface_hash(
            &module,
            CompileTarget::Wasm,
            BuildProfile::Debug,
            None,
            Some(dependency_public_surface_hash),
        )
        .expect("test module should pass prepare")
        .resource_summary_cache_namespace_key
    }

    #[test]
    fn resource_reachability_keeps_only_entry_direct_graph() {
        let mut types = crate::types::TypeCtx::new();
        let main_body = test_direct_call_body(&mut types, "used");
        let used_body = test_literal_body(&mut types, 1);
        let unused_body = test_literal_body(&mut types, 2);
        let main = test_hir_function(&mut types, "main", main_body);
        let used = test_hir_function(&mut types, "used", used_body);
        let unused = test_hir_function(&mut types, "unused", unused_body);
        let module = test_module(vec![main, used, unused]);

        let reachable = collect_reachable_function_set(&module, &[String::from("main")]);

        assert!(!reachable.is_conservative_all);
        assert!(reachable.names.contains("main"));
        assert!(reachable.names.contains("used"));
        assert!(!reachable.names.contains("unused"));
    }

    #[test]
    fn resource_reachability_is_conservative_for_ambiguous_mangled_call() {
        let mut types = crate::types::TypeCtx::new();
        let main_body = test_direct_call_body(&mut types, "helper");
        let helper_i32_body = test_literal_body(&mut types, 1);
        let helper_u8_body = test_literal_body(&mut types, 2);
        let main = test_hir_function(&mut types, "main", main_body);
        let helper_i32 = test_hir_function(&mut types, "helper__i32", helper_i32_body);
        let helper_u8 = test_hir_function(&mut types, "helper__u8", helper_u8_body);
        let module = test_module(vec![main, helper_i32, helper_u8]);

        let reachable = collect_reachable_function_set(&module, &[String::from("main")]);

        assert!(reachable.is_conservative_all);
        assert_eq!(reachable.names.len(), 3);
    }

    /// Resource summary namespace key は typed public signature を invalidation 境界にする。
    /// 関数本体だけの差分は stdlib summary namespace を変えず、後続の function body hash が
    /// 個別 summary value を invalidation する段階まで過剰に広げない。
    #[test]
    fn resource_summary_cache_namespace_key_ignores_function_body_only_edits() {
        let first = prepared_resource_summary_cache_namespace_key(
            "pub fn answer %fn unit i32 \\unit:\n    1\n",
        );
        let second = prepared_resource_summary_cache_namespace_key(
            "pub fn answer %fn unit i32 \\unit:\n    2\n",
        );

        assert_eq!(first, second);
        assert_eq!(first.dependency_public_surface_hash, None);
    }

    /// public callable の型境界が変わる場合は、Resource IR summary cache の namespace も
    /// stale hit を避けるために変わる。これは typed HIR や `TypeId` を保存せず、
    /// stable text/hash だけで公開面の差分を検出するための contract である。
    #[test]
    fn resource_summary_cache_namespace_key_tracks_public_signature_edits() {
        let returns_i32 = prepared_resource_summary_cache_namespace_key(
            "pub fn answer %fn unit i32 \\unit:\n    1\n",
        );
        let returns_unit = prepared_resource_summary_cache_namespace_key(
            "pub fn answer %fn unit unit \\unit:\n    unit\n",
        );

        assert_ne!(returns_i32, returns_unit);
    }

    /// dependency public surface hash は loader から compiler へ接続する次段階の入力である。
    /// 同じ typed public signature でも dependency aggregate が変わる場合、namespace key は
    /// 別物として扱える形にしておく。
    #[test]
    fn resource_summary_cache_namespace_key_tracks_dependency_surface_input() {
        let base = ResourceSummaryCacheNamespaceKey::new(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            7,
            None,
        );
        let with_dependency = ResourceSummaryCacheNamespaceKey::new(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            7,
            Some(1),
        );
        let other_dependency = ResourceSummaryCacheNamespaceKey::new(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            7,
            Some(2),
        );

        assert_ne!(base, with_dependency);
        assert_ne!(with_dependency, other_dependency);
    }

    /// loader が計算した dependency public surface hash は、prepare phase を通じて
    /// Resource summary namespace key の一部になる。これは compile path で消費する
    /// semantic invalidation input であり、prewarm 専用 artifact ではない。
    #[test]
    fn resource_summary_cache_namespace_key_uses_prepare_dependency_surface_hash() {
        let source = "pub fn answer %fn unit i32 \\unit:\n    1\n";
        let without_dependency = prepared_resource_summary_cache_namespace_key(source);
        let with_dependency =
            prepared_resource_summary_cache_namespace_key_with_dependency_hash(source, 123);

        assert_eq!(with_dependency.dependency_public_surface_hash, Some(123));
        assert_ne!(without_dependency, with_dependency);
    }

    #[test]
    fn resource_summary_proof_header_uses_compile_namespace_and_policy_hashes() {
        let namespace = ResourceSummaryCacheNamespaceKey::new(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            7,
            Some(11),
        );
        let mut source_map = SourceMap::new();
        let span = Span::new(FileId(0), 2, 8);
        source_map.add_with_capabilities(
            "/virtual/private-cache.nepl",
            String::from("private_cache_lookup"),
            private_cache_capabilities(PrivateCacheOp::Lookup, span),
        );

        let header = namespace.resource_summary_proof_header(Some(&source_map), Some(13));

        assert_eq!(
            header.compiler_identity_hash,
            resource_summary_proof_compiler_identity_hash()
        );
        assert_eq!(
            header.target_hash,
            resource_summary_proof_target_hash(CompileTarget::Wasm)
        );
        assert_eq!(
            header.profile_hash,
            resource_summary_proof_profile_hash(BuildProfile::Debug)
        );
        assert_eq!(header.stdlib_content_hash, Some(13));
        assert_eq!(header.dependency_public_surface_hash, Some(11));
        assert_eq!(
            header.resource_summary_namespace_hash,
            namespace.stable_hash
        );
        assert_eq!(
            header.source_capability_policy_set_hash,
            resource_summary_source_capability_policy_set_hash(Some(&source_map))
        );
        assert_eq!(
            header.private_effect_policy_hash,
            Some(resource_summary_private_effect_policy_hash())
        );
    }

    #[test]
    fn resource_summary_proof_header_changes_when_source_capability_policy_changes() {
        let namespace = ResourceSummaryCacheNamespaceKey::new(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            7,
            None,
        );
        let mut lookup_map = SourceMap::new();
        lookup_map.add_with_capabilities(
            "/virtual/private-cache.nepl",
            String::from("private_cache_lookup"),
            private_cache_capabilities(PrivateCacheOp::Lookup, Span::new(FileId(0), 2, 8)),
        );
        let mut insert_map = SourceMap::new();
        insert_map.add_with_capabilities(
            "/virtual/private-cache.nepl",
            String::from("private_cache_insert"),
            private_cache_capabilities(PrivateCacheOp::Insert, Span::new(FileId(0), 2, 8)),
        );

        let lookup_header = namespace.resource_summary_proof_header(Some(&lookup_map), None);
        let insert_header = namespace.resource_summary_proof_header(Some(&insert_map), None);

        assert_ne!(
            lookup_header.source_capability_policy_set_hash,
            insert_header.source_capability_policy_set_hash
        );
        assert_eq!(
            lookup_header.compatibility_reject(insert_header),
            Some(crate::resource::ResourceSummaryProofArtifactCompatibilityReject::SourceCapabilityPolicySet)
        );
    }

    #[test]
    fn prepare_exposes_resource_summary_proof_header() {
        let source = "pub fn answer %fn unit i32 \\unit:\n    1\n";
        let module = parse_test_module(source);
        let mut source_map = SourceMap::new();
        source_map.add("/virtual/entry.nepl", String::from(source));

        let prepared =
            prepare_module_for_codegen_with_source_map_and_dependency_public_surface_hash(
                &module,
                CompileTarget::Wasm,
                BuildProfile::Debug,
                Some(&source_map),
                Some(123),
            )
            .expect("test module should pass prepare");

        assert_eq!(
            prepared
                .resource_summary_proof_header
                .resource_summary_namespace_hash,
            prepared.resource_summary_cache_namespace_key.stable_hash
        );
        assert_eq!(
            prepared
                .resource_summary_proof_header
                .dependency_public_surface_hash,
            Some(123)
        );
        assert_eq!(
            prepared
                .resource_summary_proof_header
                .private_effect_policy_hash,
            Some(resource_summary_private_effect_policy_hash())
        );
    }

    /// prepare phase は `.neplmeta` の public interface artifact も生成する。
    /// `.neplmeta` は typed public signature と compile context hash だけを持つ。
    /// typed HIR や `TypeId` を保存しないため、body-only edit では public signature hash が
    /// 変わらない。loader 由来の module surface がない prepare path では source key を
    /// 作らず、materializer / body skip 側で fail-closed に扱う。
    #[test]
    fn prepare_exposes_neplmeta_artifact_for_public_interface() {
        let first_source = "pub fn answer %fn unit i32 \\unit:\n    1\n";
        let second_source = "pub fn answer %fn unit i32 \\unit:\n    2\n";
        let first_module = parse_test_module(first_source);
        let second_module = parse_test_module(second_source);
        let mut first_source_map = SourceMap::new();
        first_source_map.add("/virtual/entry.nepl", String::from(first_source));
        let mut second_source_map = SourceMap::new();
        second_source_map.add("/virtual/entry.nepl", String::from(second_source));

        let first = prepare_module_for_codegen_with_source_map_and_dependency_public_surface_hash(
            &first_module,
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(&first_source_map),
            Some(123),
        )
        .expect("first test module should pass prepare");
        let second = prepare_module_for_codegen_with_source_map_and_dependency_public_surface_hash(
            &second_module,
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(&second_source_map),
            Some(123),
        )
        .expect("second test module should pass prepare");

        assert_eq!(
            first
                .nepl_meta_artifact
                .header()
                .typed_public_signature_hash,
            first.public_signatures.stable_hash
        );
        assert_eq!(first.nepl_meta_artifact.payload_consistency_reject(), None);
        assert_eq!(
            first
                .nepl_meta_artifact
                .header()
                .typed_public_signature_hash,
            second
                .nepl_meta_artifact
                .header()
                .typed_public_signature_hash
        );
        assert_eq!(first.nepl_meta_artifact.header().source_key_hash, None);
        assert_eq!(second.nepl_meta_artifact.header().source_key_hash, None);
        assert_eq!(
            first
                .nepl_meta_artifact
                .header()
                .dependency_public_surface_hash,
            Some(123)
        );
    }

    fn materialized_dependency_surface(
        path: &str,
        source: &str,
    ) -> crate::typecheck::TypedPublicSurfaceTable {
        let mut source_map = SourceMap::new();
        let file = source_map.add(path, String::from(source));
        let module = parse_test_module_for_file(file, source);
        let checked = crate::typecheck::typecheck(
            &module,
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(&source_map),
        );
        assert!(
            checked.diagnostics.is_empty(),
            "dependency diagnostics: {:?}",
            checked.diagnostics
        );
        checked.public_surface
    }

    /// unused callable surface は名前解決にだけ存在し、codegen body を要求しない。
    ///
    /// loader が prelude / import body を artifact に置き換える最初の安全圏はこの形である。
    /// 依存先の callable が実際に選択されない限り、root module の HIR / Resource IR / wasm
    /// 生成は root body だけで閉じる。
    #[test]
    fn prepare_accepts_unused_materialized_callable_without_dependency_body() {
        let dep_source = "pub fn dep_value %fn unit i32 \\unit:\n    7\n";
        let dep_surface = materialized_dependency_surface("project/dep.nepl", dep_source);
        let root_source =
            "#no_prelude\n#import \"dep\" as *\nfn main %fn unit i32 \\unit:\n    1\n";
        let mut root_source_map = SourceMap::new();
        let root_file = root_source_map.add("project/root.nepl", String::from(root_source));
        let dep_file = root_source_map.add("project/dep.nepl", String::from(dep_source));
        let root_module = parse_test_module_for_file(root_file, root_source);
        let materialized = [crate::typecheck::MaterializedPublicSurfaceInput {
            table: dep_surface,
            module_path: String::from("project/dep.nepl"),
            file_id: dep_file,
        }];

        let prepared =
            prepare_module_for_codegen_with_public_interface_artifacts_and_resource_summary_value_cache(
                &root_module,
                CompileTarget::Wasm,
                BuildProfile::Debug,
                Some(&root_source_map),
                PublicInterfaceArtifactInputs::new(Some(44), None, &materialized),
                None,
            )
            .expect("unused materialized callable should not require source fallback");

        assert!(
            prepared
                .hir_module
                .functions
                .iter()
                .all(|function| !function
                    .name
                    .starts_with(MATERIALIZED_CALLABLE_SYMBOL_PREFIX)),
            "unused materialized dependency must not be emitted as a codegen function"
        );
    }

    /// selected materialized callable は `.neplobj` ができるまで source fallback へ戻す。
    ///
    /// `.neplmeta` は名前解決・型検査の authority であり、関数本体や Resource proof を
    /// 保存しない。直接呼び出しが root HIR に残った場合は、未知 wasm symbol になる前に
    /// backend 診断で停止する。
    #[test]
    fn prepare_rejects_selected_materialized_callable_without_code_artifact() {
        let dep_source = "pub fn dep_value %fn unit i32 \\unit:\n    7\n";
        let dep_surface = materialized_dependency_surface("project/dep.nepl", dep_source);
        let root_source =
            "#no_prelude\n#import \"dep\" as *\nfn main %fn unit i32 \\unit:\n    dep_value\n";
        let mut root_source_map = SourceMap::new();
        let root_file = root_source_map.add("project/root.nepl", String::from(root_source));
        let dep_file = root_source_map.add("project/dep.nepl", String::from(dep_source));
        let root_module = parse_test_module_for_file(root_file, root_source);
        let materialized = [crate::typecheck::MaterializedPublicSurfaceInput {
            table: dep_surface,
            module_path: String::from("project/dep.nepl"),
            file_id: dep_file,
        }];

        let result =
            prepare_module_for_codegen_with_public_interface_artifacts_and_resource_summary_value_cache(
                &root_module,
                CompileTarget::Wasm,
                BuildProfile::Debug,
                Some(&root_source_map),
                PublicInterfaceArtifactInputs::new(Some(44), None, &materialized),
                None,
            );
        let Err(err) = result else {
            panic!("selected materialized callable must require source fallback");
        };
        let CoreError::Diagnostics(diagnostics) = err else {
            panic!("expected diagnostics for materialized callable fallback");
        };

        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.code
                    == DiagnosticCode::Backend(
                        BackendDiagnosticCode::MaterializedFunctionBodyMissing,
                    )
            }),
            "diagnostics should contain materialized function body fallback code: {:?}",
            diagnostics
        );
    }

    /// `.neplobj` availability resolver は direct call だけを段階的に許可する。
    ///
    /// function value、memoized function value、indirect call は Resource proof と backend
    /// representation が揃うまで fail-closed に残す必要があるため、body-missing 収集時点で
    /// direct call と同一視しない。
    #[test]
    fn materialized_body_missing_diagnostics_classify_codegen_use_kind() {
        let ty = crate::types::TypeId(0);
        let span = Span::dummy();
        let plain_identity = crate::function_identity::FunctionValueIdentity::new(
            String::from("neplmeta$dep$1$2"),
            None,
            ty,
            ast::Effect::Pure,
            Vec::new(),
        );
        let memo_identity = crate::function_identity::FunctionValueIdentity::new(
            String::from("neplmeta$memo_dep$1$2"),
            None,
            ty,
            ast::Effect::Pure,
            Vec::new(),
        );
        let module = crate::hir::HirModule {
            functions: Vec::from([crate::hir::HirFunction {
                doc: None,
                name: String::from("main"),
                origin_name: String::from("main"),
                func_ty: ty,
                params: Vec::new(),
                result: ty,
                effect: ast::Effect::Pure,
                body: crate::hir::HirBody::Block(crate::hir::HirBlock {
                    lines: Vec::from([
                        crate::hir::HirLine {
                            expr: crate::hir::HirExpr {
                                ty,
                                kind: crate::hir::HirExprKind::Call {
                                    callee: crate::hir::FuncRef::User(
                                        String::from("neplmeta$direct$1$2"),
                                        Vec::new(),
                                        None,
                                    ),
                                    args: Vec::new(),
                                },
                                span,
                            },
                            drop_result: true,
                        },
                        crate::hir::HirLine {
                            expr: crate::hir::HirExpr {
                                ty,
                                kind: crate::hir::HirExprKind::FnValue(plain_identity),
                                span,
                            },
                            drop_result: true,
                        },
                        crate::hir::HirLine {
                            expr: crate::hir::HirExpr {
                                ty,
                                kind: crate::hir::HirExprKind::MemoizedFunctionValue(memo_identity),
                                span,
                            },
                            drop_result: true,
                        },
                    ]),
                    ty,
                    span,
                }),
                span,
            }]),
            entry: Some(String::from("main")),
            externs: Vec::new(),
            string_literals: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
        };

        let diagnostics = materialized_codegen_dependency_diagnostics(&module, &[]);

        assert_eq!(diagnostics.len(), 3);
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("for direct call")));
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("for function value")));
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic
                .message
                .contains("for memoized function value")));
    }

    /// indirect call は function value を呼び出す形であり、direct call 用 `.neplobj`
    /// availability の対象ではない。
    #[test]
    fn materialized_body_missing_diagnostics_keep_indirect_callee_as_function_value() {
        let ty = crate::types::TypeId(0);
        let span = Span::dummy();
        let identity = crate::function_identity::FunctionValueIdentity::new(
            String::from("neplmeta$indirect$1$2"),
            None,
            ty,
            ast::Effect::Pure,
            Vec::new(),
        );
        let module = crate::hir::HirModule {
            functions: Vec::from([crate::hir::HirFunction {
                doc: None,
                name: String::from("main"),
                origin_name: String::from("main"),
                func_ty: ty,
                params: Vec::new(),
                result: ty,
                effect: ast::Effect::Pure,
                body: crate::hir::HirBody::Block(crate::hir::HirBlock {
                    lines: Vec::from([crate::hir::HirLine {
                        expr: crate::hir::HirExpr {
                            ty,
                            kind: crate::hir::HirExprKind::CallIndirect {
                                callee: Box::new(crate::hir::HirExpr {
                                    ty,
                                    kind: crate::hir::HirExprKind::FnValue(identity),
                                    span,
                                }),
                                params: Vec::new(),
                                result: ty,
                                effect: ast::Effect::Pure,
                                args: Vec::new(),
                            },
                            span,
                        },
                        drop_result: true,
                    }]),
                    ty,
                    span,
                }),
                span,
            }]),
            entry: Some(String::from("main")),
            externs: Vec::new(),
            string_literals: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
        };

        let diagnostics = materialized_codegen_dependency_diagnostics(&module, &[]);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("neplmeta$indirect$1$2"));
        assert!(diagnostics[0].message.contains("for function value"));
        assert!(!diagnostics[0].message.contains("for direct call"));
    }

    /// direct call の引数に materialized function value が入る場合、direct call 自体と
    /// 高階関数依存は別々に扱う。
    #[test]
    fn materialized_body_missing_diagnostics_do_not_mask_nested_function_value() {
        let ty = crate::types::TypeId(0);
        let span = Span::dummy();
        let identity = crate::function_identity::FunctionValueIdentity::new(
            String::from("neplmeta$arg$1$2"),
            None,
            ty,
            ast::Effect::Pure,
            Vec::new(),
        );
        let module = crate::hir::HirModule {
            functions: Vec::from([crate::hir::HirFunction {
                doc: None,
                name: String::from("main"),
                origin_name: String::from("main"),
                func_ty: ty,
                params: Vec::new(),
                result: ty,
                effect: ast::Effect::Pure,
                body: crate::hir::HirBody::Block(crate::hir::HirBlock {
                    lines: Vec::from([crate::hir::HirLine {
                        expr: crate::hir::HirExpr {
                            ty,
                            kind: crate::hir::HirExprKind::Call {
                                callee: crate::hir::FuncRef::User(
                                    String::from("neplmeta$direct_nested$1$2"),
                                    Vec::new(),
                                    None,
                                ),
                                args: Vec::from([crate::hir::HirExpr {
                                    ty,
                                    kind: crate::hir::HirExprKind::FnValue(identity),
                                    span,
                                }]),
                            },
                            span,
                        },
                        drop_result: true,
                    }]),
                    ty,
                    span,
                }),
                span,
            }]),
            entry: Some(String::from("main")),
            externs: Vec::new(),
            string_literals: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
        };

        let diagnostics = materialized_codegen_dependency_diagnostics(&module, &[]);

        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("neplmeta$direct_nested$1$2")
            && diagnostic.message.contains("for direct call")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("neplmeta$arg$1$2")
            && diagnostic.message.contains("for function value")));
    }

    fn test_neplobj_direct_call_key(
        name: &str,
        signature_hash: u64,
    ) -> crate::artifact::NeplObjDirectCallKey {
        let link_symbol = crate::typecheck::PublicCallableLinkSymbol {
            source_path: String::from("project/dep.nepl"),
            name: String::from(name),
            signature_hash,
        };
        crate::artifact::NeplObjDirectCallKey::new(
            CompileTarget::Wasm,
            BuildProfile::Debug,
            Some(0x10),
            link_symbol,
            0x20,
            0x30,
            crate::artifact::nepl_obj_empty_generic_instantiation_hash(),
            0x40,
            0x50,
        )
    }

    /// `.neplobj` direct-call availability は diagnostic 化の前に structured dependency を
    /// 消費する。実際の caller は、対応する codegen fragment payload を同時に接続できる
    /// 場合だけ key を渡す。
    #[test]
    fn materialized_body_missing_diagnostics_resolve_available_direct_call_key() {
        let ty = crate::types::TypeId(0);
        let span = Span::dummy();
        let key = test_neplobj_direct_call_key("dep_value", 0x1234);
        let module = crate::hir::HirModule {
            functions: Vec::from([crate::hir::HirFunction {
                doc: None,
                name: String::from("main"),
                origin_name: String::from("main"),
                func_ty: ty,
                params: Vec::new(),
                result: ty,
                effect: ast::Effect::Pure,
                body: crate::hir::HirBody::Block(crate::hir::HirBlock {
                    lines: Vec::from([crate::hir::HirLine {
                        expr: crate::hir::HirExpr {
                            ty,
                            kind: crate::hir::HirExprKind::Call {
                                callee: crate::hir::FuncRef::User(
                                    key.materialized_symbol.clone(),
                                    Vec::new(),
                                    None,
                                ),
                                args: Vec::new(),
                            },
                            span,
                        },
                        drop_result: true,
                    }]),
                    ty,
                    span,
                }),
                span,
            }]),
            entry: Some(String::from("main")),
            externs: Vec::new(),
            string_literals: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
        };

        let diagnostics =
            materialized_codegen_dependency_diagnostics(&module, core::slice::from_ref(&key));

        assert!(diagnostics.is_empty());
    }

    /// direct-call key は function value / indirect / memoization の証明を代替しない。
    /// 同じ materialized symbol に見える場合でも、function value として使われた依存は
    /// body-missing に残す。
    #[test]
    fn materialized_body_missing_diagnostics_do_not_resolve_function_value_key() {
        let ty = crate::types::TypeId(0);
        let span = Span::dummy();
        let key = test_neplobj_direct_call_key("dep_value", 0x5678);
        let identity = crate::function_identity::FunctionValueIdentity::new(
            key.materialized_symbol.clone(),
            None,
            ty,
            ast::Effect::Pure,
            Vec::new(),
        );
        let module = crate::hir::HirModule {
            functions: Vec::from([crate::hir::HirFunction {
                doc: None,
                name: String::from("main"),
                origin_name: String::from("main"),
                func_ty: ty,
                params: Vec::new(),
                result: ty,
                effect: ast::Effect::Pure,
                body: crate::hir::HirBody::Block(crate::hir::HirBlock {
                    lines: Vec::from([crate::hir::HirLine {
                        expr: crate::hir::HirExpr {
                            ty,
                            kind: crate::hir::HirExprKind::FnValue(identity),
                            span,
                        },
                        drop_result: true,
                    }]),
                    ty,
                    span,
                }),
                span,
            }]),
            entry: Some(String::from("main")),
            externs: Vec::new(),
            string_literals: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
        };

        let diagnostics =
            materialized_codegen_dependency_diagnostics(&module, core::slice::from_ref(&key));

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("for function value"));
    }

    #[test]
    fn resource_reachability_is_conservative_for_indirect_call() {
        let mut types = crate::types::TypeCtx::new();
        let i32_ty = types.i32();
        let fn_ty = types.function(Vec::new(), Vec::new(), i32_ty, ast::Effect::Pure);
        let main_body = crate::hir::HirBody::Block(crate::hir::HirBlock {
            lines: vec![crate::hir::HirLine {
                expr: crate::hir::HirExpr {
                    ty: i32_ty,
                    kind: crate::hir::HirExprKind::CallIndirect {
                        callee: Box::new(crate::hir::HirExpr {
                            ty: fn_ty,
                            kind: crate::hir::HirExprKind::FnValue(
                                crate::function_identity::FunctionValueIdentity::new(
                                    String::from("used"),
                                    None,
                                    fn_ty,
                                    ast::Effect::Pure,
                                    Vec::new(),
                                ),
                            ),
                            span: Span::dummy(),
                        }),
                        params: Vec::new(),
                        result: i32_ty,
                        effect: ast::Effect::Pure,
                        args: Vec::new(),
                    },
                    span: Span::dummy(),
                },
                drop_result: false,
            }],
            ty: i32_ty,
            span: Span::dummy(),
        });
        let main = test_hir_function(&mut types, "main", main_body);
        let used_body = test_literal_body(&mut types, 1);
        let used = test_hir_function(&mut types, "used", used_body);
        let module = test_module(vec![main, used]);

        let reachable = collect_reachable_function_set(&module, &[String::from("main")]);

        assert!(reachable.is_conservative_all);
        assert_eq!(reachable.names.len(), 2);
    }

    #[test]
    fn resource_reachability_is_conservative_for_raw_llvm_body() {
        let mut types = crate::types::TypeCtx::new();
        let raw_body = crate::hir::HirBody::LlvmIr(ast::LlvmIrBlock {
            lines: vec![String::from("  ret i32 0")],
            span: Span::dummy(),
        });
        let main = test_hir_function(&mut types, "main", raw_body);
        let unused_body = test_literal_body(&mut types, 1);
        let unused = test_hir_function(&mut types, "unused", unused_body);
        let module = test_module(vec![main, unused]);

        let reachable = collect_reachable_function_set(&module, &[String::from("main")]);

        assert!(reachable.is_conservative_all);
        assert_eq!(reachable.names.len(), 2);
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
    fn resource_effect_gate_maps_private_cache_outside_boundary_to_resource_raw_code() {
        let diagnostic = ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
            function: String::from("memo_backend"),
            operation: PrivateCacheOp::Lookup,
            region: PrivateEffectRegion::UnsealedIntrinsic,
            span: Span::dummy(),
        };

        let error = resource_effect_boundary_diagnostic_to_error(&diagnostic);

        assert_eq!(
            error.code,
            DiagnosticCode::Resource(crate::diagnostic_codes::ResourceDiagnosticCode::Raw(
                crate::diagnostic_codes::ResourceRawDiagnosticCode::MemoryOutsideBoundary,
            ))
        );
        assert!(error.message.contains("private cache operation"));
        assert!(error.message.contains("private_cache.lookup"));
        assert!(error.message.contains("outside private-cache boundary"));
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
    fn resource_effect_gate_allows_private_cache_inside_exact_private_cache_boundary() {
        for operation in PrivateCacheOp::ALL {
            let mut source_map = SourceMap::new();
            let cache_file = source_map.add("stdlib/core/memo/internal.nepl", String::new());
            let span = Span::new(cache_file, 10, 30);
            source_map.set_capabilities(cache_file, private_cache_capabilities(operation, span));
            let diagnostic = ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
                function: String::from("memo_backend"),
                operation,
                region: PrivateEffectRegion::UnsealedIntrinsic,
                span,
            };

            assert!(
                resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
                    &diagnostic,
                    Some(&source_map),
                ),
                "{operation} must be allowed only by its exact private-cache boundary"
            );
        }
    }

    #[test]
    fn resource_effect_gate_rejects_private_cache_operation_and_span_mismatch() {
        let mut source_map = SourceMap::new();
        let cache_file = source_map.add("stdlib/core/memo/internal.nepl", String::new());
        let span = Span::new(cache_file, 10, 30);
        let shifted_span = Span::new(cache_file, 11, 31);
        source_map.set_capabilities(
            cache_file,
            private_cache_capabilities(PrivateCacheOp::Lookup, span),
        );
        let operation_mismatch = ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
            function: String::from("memo_backend_insert"),
            operation: PrivateCacheOp::Insert,
            region: PrivateEffectRegion::UnsealedIntrinsic,
            span,
        };
        let span_mismatch = ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
            function: String::from("memo_backend_lookup"),
            operation: PrivateCacheOp::Lookup,
            region: PrivateEffectRegion::UnsealedIntrinsic,
            span: shifted_span,
        };

        assert!(
            !resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
                &operation_mismatch,
                Some(&source_map),
            )
        );
        assert!(
            !resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
                &span_mismatch,
                Some(&source_map),
            )
        );
    }

    /// sealed private cache region は proof identity であり、file / span /
    /// operation が一致しても別 region の境界へ拡張してはならない。
    #[test]
    fn resource_effect_gate_rejects_private_cache_region_mismatch() {
        let mut source_map = SourceMap::new();
        let cache_file = source_map.add("stdlib/core/memo/internal.nepl", String::new());
        let span = Span::new(cache_file, 10, 30);
        source_map.set_capabilities(
            cache_file,
            private_cache_capabilities_in_region(
                PrivateCacheOp::Lookup,
                PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(1)),
                span,
            ),
        );
        let matching_region = ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
            function: String::from("memo_backend_lookup"),
            operation: PrivateCacheOp::Lookup,
            region: PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(1)),
            span,
        };
        let mismatched_region = ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
            function: String::from("memo_backend_lookup"),
            operation: PrivateCacheOp::Lookup,
            region: PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(2)),
            span,
        };
        let unsealed_region = ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
            function: String::from("memo_backend_lookup"),
            operation: PrivateCacheOp::Lookup,
            region: PrivateEffectRegion::UnsealedIntrinsic,
            span,
        };

        assert!(resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
            &matching_region,
            Some(&source_map),
        ));
        assert!(
            !resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
                &mismatched_region,
                Some(&source_map),
            )
        );
        assert!(
            !resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
                &unsealed_region,
                Some(&source_map),
            )
        );
    }

    #[test]
    fn resource_effect_gate_rejects_private_cache_boundary_from_other_file() {
        let mut source_map = SourceMap::new();
        let allowed_file = source_map.add("stdlib/core/memo/internal.nepl", String::new());
        let user_file = source_map.add("src/main.nepl", String::new());
        let allowed_span = Span::new(allowed_file, 10, 30);
        let user_span = Span::new(user_file, 10, 30);
        source_map.set_capabilities(
            allowed_file,
            private_cache_capabilities(PrivateCacheOp::Lookup, allowed_span),
        );
        let diagnostic = ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
            function: String::from("user_lookup"),
            operation: PrivateCacheOp::Lookup,
            region: PrivateEffectRegion::UnsealedIntrinsic,
            span: user_span,
        };

        assert!(
            !resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
                &diagnostic,
                Some(&source_map),
            )
        );
    }

    #[test]
    fn resource_effect_gate_does_not_mask_private_cache_in_pure_function() {
        let mut source_map = SourceMap::new();
        let cache_file = source_map.add("stdlib/core/memo/internal.nepl", String::new());
        let span = Span::new(cache_file, 10, 30);
        source_map.set_capabilities(
            cache_file,
            private_cache_capabilities(PrivateCacheOp::Lookup, span),
        );
        let diagnostic = ResourceEffectBoundaryDiagnostic::PrivateCacheInPureFunction {
            function: String::from("memo_backend_lookup"),
            operation: PrivateCacheOp::Lookup,
            region: PrivateEffectRegion::UnsealedIntrinsic,
            span,
        };

        assert!(
            !resource_effect_boundary_diagnostic_is_raw_boundary_allowed(
                &diagnostic,
                Some(&source_map),
            )
        );
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
        | crate::resource::ResourceEffectBoundaryDiagnostic::PrivateStateInPureFunction {
            span,
            ..
        }
        | crate::resource::ResourceEffectBoundaryDiagnostic::PrivateCacheInPureFunction {
            span,
            ..
        }
        | crate::resource::ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
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
        | crate::resource::ResourceEffectBoundaryDiagnostic::PrivateCacheRegionEscape {
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
        crate::resource::ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
            operation,
            region,
            ..
        } => {
            let Some(span) = resource_effect_boundary_diagnostic_span(diagnostic) else {
                return false;
            };
            source_map
                .map(|map| map.private_cache_boundary_allowed_at(span, *operation, *region))
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
        crate::resource::ResourceEffectBoundaryDiagnostic::PrivateCacheRegionEscape { .. } => false,
        crate::resource::ResourceEffectBoundaryDiagnostic::ImpureCallInPureFunction { .. } => false,
        crate::resource::ResourceEffectBoundaryDiagnostic::PrivateStateInPureFunction {
            ..
        }
        | crate::resource::ResourceEffectBoundaryDiagnostic::PrivateCacheInPureFunction {
            ..
        } => false,
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
        crate::resource::ResourceEffectBoundaryDiagnostic::PrivateStateInPureFunction {
            function,
            operation,
            region,
            span,
        } => Diagnostic::error_with_code(
            code,
            format!(
                "pure function '{}' uses unmasked private state operation '{}' in private effect region '{}'",
                function, operation, region
            ),
            *span,
        ),
        crate::resource::ResourceEffectBoundaryDiagnostic::PrivateCacheInPureFunction {
            function,
            operation,
            region,
            span,
        } => Diagnostic::error_with_code(
            code,
            format!(
                "pure function '{}' uses unmasked private cache operation '{}' in private effect region '{}'",
                function, operation, region
            ),
            *span,
        ),
        crate::resource::ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
            function,
            operation,
            region,
            span,
        } => Diagnostic::error_with_code(
            code,
            format!(
                "function '{}' uses private cache operation '{}' in private effect region '{}' outside private-cache boundary",
                function, operation, region
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
        crate::resource::ResourceEffectBoundaryDiagnostic::PrivateCacheRegionEscape {
            function,
            region,
            place,
            span,
        } => Diagnostic::error_with_code(
            code,
            format!(
                "function '{}' lets private cache region '{}' escape through returned place {:?}",
                function, region, place
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
    prepare_module_for_codegen_with_source_map_and_dependency_public_surface_hash(
        module, target, profile, source_map, None,
    )
}

pub fn prepare_module_for_codegen_with_source_map_and_dependency_public_surface_hash(
    module: &ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
    source_map: Option<&SourceMap>,
    dependency_public_surface_hash: Option<u64>,
) -> Result<PreparedProgram, CoreError> {
    prepare_module_for_codegen_with_source_map_dependency_public_surface_hash_and_resource_summary_value_cache(
        module,
        target,
        profile,
        source_map,
        dependency_public_surface_hash,
        None,
    )
}

/// Resource summary value cache を Resource IR static check へ渡す prepare phase。
///
/// target/profile、typed public signature hash、dependency public surface hash は
/// `ResourceSummaryCacheNamespaceKey` の材料であり、この関数はその namespace と
/// session cache の寿命を同じ compile の中で結び付ける。cache value はまだ再利用せず、
/// `DropTraversal + ForallInitializedRange` の最終 summary 候補を bypass として観測する。
pub fn prepare_module_for_codegen_with_source_map_dependency_public_surface_hash_and_resource_summary_value_cache(
    module: &ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
    source_map: Option<&SourceMap>,
    dependency_public_surface_hash: Option<u64>,
    resource_summary_value_cache: Option<&mut crate::resource::ResourceSummaryValueCache>,
) -> Result<PreparedProgram, CoreError> {
    let mut stage_recorder = CompileStageRecorder::disabled();
    prepare_module_for_codegen_with_source_map_dependency_public_surface_hash_and_resource_summary_value_cache_internal(
        module,
        target,
        profile,
        source_map,
        PublicInterfaceArtifactInputs::new(dependency_public_surface_hash, None, &[]),
        resource_summary_value_cache,
        ResourceSummaryProofArtifactCacheOptions::none(),
        &mut stage_recorder,
    )
}

/// `.neplmeta` 由来の dependency surface を含めて prepare phase を実行する。
///
/// materialized surface は source body を持たないため、直接呼び出しの codegen body が必要な
/// 場合はこの phase で診断に倒す。loader / Web session はその診断を source fallback の根拠に
/// し、`.neplobj` が未実装の段階でも未知 symbol を backend へ渡さない。
pub fn prepare_module_for_codegen_with_public_interface_artifacts_and_resource_summary_value_cache(
    module: &ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
    source_map: Option<&SourceMap>,
    public_interface_artifacts: PublicInterfaceArtifactInputs<'_>,
    resource_summary_value_cache: Option<&mut crate::resource::ResourceSummaryValueCache>,
) -> Result<PreparedProgram, CoreError> {
    let mut stage_recorder = CompileStageRecorder::disabled();
    prepare_module_for_codegen_with_source_map_dependency_public_surface_hash_and_resource_summary_value_cache_internal(
        module,
        target,
        profile,
        source_map,
        public_interface_artifacts,
        resource_summary_value_cache,
        ResourceSummaryProofArtifactCacheOptions::none(),
        &mut stage_recorder,
    )
}

fn prepare_module_for_codegen_with_source_map_dependency_public_surface_hash_and_resource_summary_value_cache_internal(
    module: &ast::Module,
    target: CompileTarget,
    profile: BuildProfile,
    source_map: Option<&SourceMap>,
    public_interface_artifacts: PublicInterfaceArtifactInputs<'_>,
    mut resource_summary_value_cache: Option<&mut crate::resource::ResourceSummaryValueCache>,
    resource_summary_proof_options: ResourceSummaryProofArtifactCacheOptions<'_>,
    stage_recorder: &mut CompileStageRecorder<'_>,
) -> Result<PreparedProgram, CoreError> {
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let precheck_diags =
        crate::target_precheck::precheck_module_before_codegen(module, target, profile);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("target_precheck", stage_start);
    stage_recorder.finish("target_precheck", stage_start_ms);
    if precheck_diags
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostic::Severity::Error))
    {
        return Err(CoreError::from_diagnostics(precheck_diags));
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let resource_tc = run_typecheck(
        module,
        target,
        profile,
        source_map,
        public_interface_artifacts.materialized_public_surfaces,
    );
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_typecheck", stage_start);
    stage_recorder.finish("resource_typecheck", stage_start_ms);
    let resource_tc = resource_tc?;
    let mut diagnostics = resource_tc.diagnostics;
    let mut types = resource_tc.types;
    let public_signatures = resource_tc.public_signatures;
    let public_surface = resource_tc.public_surface;
    let nepl_meta_artifact =
        crate::artifact::NeplMetaArtifact::from_public_surface_and_module_surface(
            target,
            profile,
            resource_summary_proof_options.stdlib_content_hash,
            public_interface_artifacts.dependency_public_surface_hash,
            source_map,
            public_signatures.clone(),
            public_interface_artifacts.module_surface.cloned(),
            public_surface.clone(),
        );
    let resource_summary_cache_namespace_key = ResourceSummaryCacheNamespaceKey::new(
        target,
        profile,
        public_signatures.stable_hash,
        public_interface_artifacts.dependency_public_surface_hash,
    );
    let resource_summary_proof_header = resource_summary_cache_namespace_key
        .resource_summary_proof_header(
            source_map,
            resource_summary_proof_options.stdlib_content_hash,
        );
    let resource_summary_value_cache_context = if resource_summary_value_cache.is_some() {
        resource_summary_value_cache_context(&resource_summary_cache_namespace_key, source_map)
    } else {
        None
    };
    if let Some(artifact) = resource_summary_proof_options.preseed_artifact {
        if let Some(cache) = resource_summary_value_cache.as_deref_mut() {
            let _ = cache.preseed_neplproof_artifact(artifact, resource_summary_proof_header);
        }
    }
    if !public_interface_artifacts
        .materialized_public_surfaces
        .is_empty()
    {
        let materialized_codegen_diags = materialized_codegen_dependency_diagnostics(
            &resource_tc.module,
            public_interface_artifacts.nepl_obj_direct_call_keys,
        );
        if !materialized_codegen_diags.is_empty() {
            diagnostics.extend(materialized_codegen_diags);
            return Err(CoreError::from_diagnostics(diagnostics));
        }
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let resource_monomorphize = monomorphize::monomorphize(&mut types, resource_tc.module);
    let mut hir_module = resource_monomorphize.module;
    let resource_unresolved_trait_calls = resource_monomorphize.unresolved_trait_calls;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_monomorphize", stage_start);
    stage_recorder.finish("resource_monomorphize", stage_start_ms);
    if !resource_unresolved_trait_calls.is_empty() {
        extend_unresolved_trait_call_diagnostics(&mut diagnostics, resource_unresolved_trait_calls);
        return Err(CoreError::from_diagnostics(diagnostics));
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let prune_result = prune_hir_module_to_entry_reachable(module, &mut hir_module, &types);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_reachable_prune", stage_start);
    stage_recorder.finish("resource_reachable_prune", stage_start_ms);
    prune_result?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let resource_drop_elaboration_plan = run_resource_static_check(
        &hir_module,
        &types,
        &mut diagnostics,
        source_map,
        resource_summary_value_cache,
        resource_summary_value_cache_context.as_ref(),
        stage_recorder,
    );
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_static_check", stage_start);
    stage_recorder.finish("resource_static_check", stage_start_ms);
    let resource_drop_elaboration_plan = resource_drop_elaboration_plan?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let resource_drop_bridge_result = run_resource_drop_elaboration_hir_bridge_gate(
        &hir_module,
        &resource_drop_elaboration_plan,
        &mut diagnostics,
    );
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("resource_drop_bridge", stage_start);
    stage_recorder.finish("resource_drop_bridge", stage_start_ms);
    resource_drop_bridge_result?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let insert_resource_drops_result =
        passes::insert_resource_drops(&mut hir_module, &mut types, &resource_drop_elaboration_plan)
            .map_err(|_| {
                CoreError::internal("resource drop elaboration plan could not be consumed")
            });
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("insert_resource_drops", stage_start);
    stage_recorder.finish("insert_resource_drops", stage_start_ms);
    insert_resource_drops_result?;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let codegen_monomorphize = monomorphize::monomorphize(&mut types, hir_module);
    let hir_module = codegen_monomorphize.module;
    let unresolved_trait_calls = codegen_monomorphize.unresolved_trait_calls;
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("codegen_monomorphize", stage_start);
    stage_recorder.finish("codegen_monomorphize", stage_start_ms);
    if !unresolved_trait_calls.is_empty() {
        extend_unresolved_trait_call_diagnostics(&mut diagnostics, unresolved_trait_calls);
        return Err(CoreError::from_diagnostics(diagnostics));
    }
    if !public_interface_artifacts
        .materialized_public_surfaces
        .is_empty()
    {
        let materialized_codegen_diags = materialized_codegen_dependency_diagnostics(
            &hir_module,
            public_interface_artifacts.nepl_obj_direct_call_keys,
        );
        if !materialized_codegen_diags.is_empty() {
            diagnostics.extend(materialized_codegen_diags);
            return Err(CoreError::from_diagnostics(diagnostics));
        }
    }
    Ok(PreparedProgram {
        types,
        hir_module,
        public_signatures,
        public_surface,
        nepl_meta_artifact,
        resource_summary_cache_namespace_key,
        resource_summary_proof_header,
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

fn prune_hir_module_to_entry_reachable(
    ast_module: &ast::Module,
    hir_module: &mut crate::hir::HirModule,
    _types: &crate::types::TypeCtx,
) -> Result<(), CoreError> {
    // 実行可能 program では、entry から到達できる関数だけが後段の安全性検査と codegen の対象になる。
    // 標準ライブラリの未使用関数を同じ固定点へ入れると、純粋な検査結果を毎回再計算することになるため、
    // HIR の呼び出しグラフ上で必要な関数集合を先に確定する。
    let Some(entry) = hir_module.entry.clone() else {
        return Ok(());
    };
    let resolved_entry = resolve_hir_entry_name(ast_module, hir_module, entry.as_str())?;
    let reachable = collect_reachable_function_set(hir_module, &[resolved_entry.clone()]);
    if reachable.is_conservative_all {
        // 呼び出しグラフが静的に閉じない場合は、安全性検査の取りこぼしを避けるため pruning しない。
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        if crate::log::is_verbose() || std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
            std::eprintln!(
                "[compile-stage] resource_reachable_prune_functions={} kept={} reason=unknown_call_graph",
                hir_module.functions.len(),
                hir_module.functions.len()
            );
        }
        return Ok(());
    }

    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let before = hir_module.functions.len();
    hir_module
        .functions
        .retain(|function| reachable.names.contains(&function.name));
    hir_module.entry = Some(resolved_entry);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if crate::log::is_verbose() || std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] resource_reachable_prune_functions={} kept={}",
            before,
            hir_module.functions.len()
        );
    }
    Ok(())
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

struct ReachableFunctionSet {
    names: BTreeSet<String>,
    is_conservative_all: bool,
}

fn collect_reachable_functions(module: &crate::hir::HirModule, entry: &str) -> Vec<String> {
    collect_reachable_function_set(module, &[String::from(entry)])
        .names
        .into_iter()
        .collect()
}

fn collect_reachable_function_set(
    module: &crate::hir::HirModule,
    roots: &[String],
) -> ReachableFunctionSet {
    let mut function_map: BTreeMap<String, &crate::hir::HirFunction> = BTreeMap::new();
    let mut all_names: BTreeSet<String> = BTreeSet::new();
    for f in &module.functions {
        function_map.insert(f.name.clone(), f);
        all_names.insert(f.name.clone());
    }
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut stack = Vec::new();
    let mut requires_conservative_all = false;
    for root in roots {
        stack.push(root.clone());
    }
    while let Some(name) = stack.pop() {
        let resolved_name = match resolve_hir_function_ref_name(&all_names, name.as_str()) {
            HirFunctionRefResolution::Resolved(name) => name,
            HirFunctionRefResolution::Missing => continue,
            HirFunctionRefResolution::Ambiguous => {
                requires_conservative_all = true;
                continue;
            }
        };
        if !visited.insert(resolved_name.clone()) {
            continue;
        }
        let Some(func) = function_map.get(resolved_name.as_str()) else {
            continue;
        };
        collect_called_functions_from_body(&func.body, &mut stack, &mut requires_conservative_all);
    }
    if requires_conservative_all {
        return ReachableFunctionSet {
            names: all_names,
            is_conservative_all: true,
        };
    }
    ReachableFunctionSet {
        names: visited,
        is_conservative_all: false,
    }
}

enum HirFunctionRefResolution {
    Resolved(String),
    Missing,
    Ambiguous,
}

fn resolve_hir_function_ref_name(
    all_names: &BTreeSet<String>,
    name: &str,
) -> HirFunctionRefResolution {
    if all_names.contains(name) {
        return HirFunctionRefResolution::Resolved(String::from(name));
    }
    let mut prefix = String::from(name);
    prefix.push_str("__");
    let mut found: Option<String> = None;
    for candidate in all_names {
        if candidate.starts_with(prefix.as_str()) {
            if found.is_some() {
                return HirFunctionRefResolution::Ambiguous;
            }
            found = Some(candidate.clone());
        }
    }
    found
        .map(HirFunctionRefResolution::Resolved)
        .unwrap_or(HirFunctionRefResolution::Missing)
}

fn collect_called_functions_from_body(
    body: &crate::hir::HirBody,
    stack: &mut Vec<String>,
    requires_conservative_all: &mut bool,
) {
    match body {
        crate::hir::HirBody::Block(block) => {
            collect_called_functions_from_block(block, stack, requires_conservative_all)
        }
        crate::hir::HirBody::Wasm(block) => {
            for line in &block.lines {
                if wasm_raw_body_line_contains_direct_call(line) {
                    *requires_conservative_all = true;
                }
            }
        }
        crate::hir::HirBody::LlvmIr(_) => {
            // LLVM IR は extern、宣言、metadata、間接呼び出しの構文を持つため、ここで簡略 parser を持たない。
            // raw LLVM 関数が到達した場合は pruning を無効化し、後段の LLVM 専用 reachability に委ねる。
            *requires_conservative_all = true;
        }
    }
}

fn wasm_raw_body_line_contains_direct_call(line: &str) -> bool {
    let semi = line.find(";;");
    let slash = line.find("//");
    let code = match (semi, slash) {
        (Some(a), Some(b)) => &line[..core::cmp::min(a, b)],
        (Some(a), None) | (None, Some(a)) => &line[..a],
        (None, None) => line,
    };
    code.trim_start().starts_with("call ")
}

fn collect_called_functions_from_block(
    block: &crate::hir::HirBlock,
    stack: &mut Vec<String>,
    requires_conservative_all: &mut bool,
) {
    for line in &block.lines {
        collect_called_functions_from_expr(&line.expr, stack, requires_conservative_all);
    }
}

fn collect_called_functions_from_expr(
    expr: &crate::hir::HirExpr,
    stack: &mut Vec<String>,
    requires_conservative_all: &mut bool,
) {
    match &expr.kind {
        crate::hir::HirExprKind::Call { callee, args } => {
            if let crate::hir::FuncRef::User(name, _, _) = callee {
                stack.push(name.clone());
            }
            for arg in args {
                collect_called_functions_from_expr(arg, stack, requires_conservative_all);
            }
        }
        crate::hir::HirExprKind::CallIndirect { callee, args, .. } => {
            *requires_conservative_all = true;
            collect_called_functions_from_expr(callee, stack, requires_conservative_all);
            for arg in args {
                collect_called_functions_from_expr(arg, stack, requires_conservative_all);
            }
        }
        crate::hir::HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_called_functions_from_expr(cond, stack, requires_conservative_all);
            collect_called_functions_from_expr(then_branch, stack, requires_conservative_all);
            collect_called_functions_from_expr(else_branch, stack, requires_conservative_all);
        }
        crate::hir::HirExprKind::While { cond, body } => {
            collect_called_functions_from_expr(cond, stack, requires_conservative_all);
            collect_called_functions_from_expr(body, stack, requires_conservative_all);
        }
        crate::hir::HirExprKind::Match { scrutinee, arms } => {
            collect_called_functions_from_expr(scrutinee, stack, requires_conservative_all);
            for arm in arms {
                collect_called_functions_from_expr(&arm.body, stack, requires_conservative_all);
            }
        }
        crate::hir::HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(payload) = payload {
                collect_called_functions_from_expr(payload, stack, requires_conservative_all);
            }
        }
        crate::hir::HirExprKind::StructConstruct { fields, .. }
        | crate::hir::HirExprKind::TupleConstruct { items: fields }
        | crate::hir::HirExprKind::Intrinsic { args: fields, .. } => {
            for field in fields {
                collect_called_functions_from_expr(field, stack, requires_conservative_all);
            }
        }
        crate::hir::HirExprKind::Block(block) => {
            collect_called_functions_from_block(block, stack, requires_conservative_all);
        }
        crate::hir::HirExprKind::Let { value, .. }
        | crate::hir::HirExprKind::Set { value, .. }
        | crate::hir::HirExprKind::AddrOf(value)
        | crate::hir::HirExprKind::Deref(value) => {
            collect_called_functions_from_expr(value, stack, requires_conservative_all);
        }
        crate::hir::HirExprKind::LiteralI32(_)
        | crate::hir::HirExprKind::LiteralF32(_)
        | crate::hir::HirExprKind::LiteralBool(_)
        | crate::hir::HirExprKind::LiteralStr(_)
        | crate::hir::HirExprKind::Unit
        | crate::hir::HirExprKind::Var(_)
        | crate::hir::HirExprKind::Drop { .. } => {}
        crate::hir::HirExprKind::FnValue(identity)
        | crate::hir::HirExprKind::MemoizedFunctionValue(identity) => {
            stack.push(identity.symbol.clone())
        }
    }
}

fn emit_wasm(
    types: &crate::types::TypeCtx,
    hir_module: &crate::hir::HirModule,
    mut diagnostics: Vec<Diagnostic>,
    include_wat_comments: bool,
    nepl_meta_artifact: crate::artifact::NeplMetaArtifact,
    resource_summary_proof_header: Option<crate::resource::ResourceSummaryProofArtifactHeader>,
    stage_recorder: &mut CompileStageRecorder<'_>,
) -> Result<CompilationArtifact, CoreError> {
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let cg = codegen_wasm::generate_wasm(types, hir_module);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("wasm_codegen", stage_start);
    stage_recorder.finish("wasm_codegen", stage_start_ms);
    let cg = match cg {
        Ok(cg) => cg,
        Err(mut codegen_diags) => {
            diagnostics.append(&mut codegen_diags);
            return Err(CoreError::from_diagnostics(diagnostics));
        }
    };
    diagnostics.extend(cg.diagnostics);
    let Some(bytes) = cg.bytes else {
        return Err(CoreError::from_diagnostics(diagnostics));
    };

    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let mut validator = Validator::new();
    let validation_result = validator.validate_all(&bytes);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("wasm_validate", stage_start);
    stage_recorder.finish("wasm_validate", stage_start_ms);
    if let Err(err) = validation_result {
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
    let stage_start = std::time::Instant::now();
    let stage_start_ms = stage_recorder.start();
    let wat_comments = if include_wat_comments {
        build_wat_comments(types, hir_module)
    } else {
        String::new()
    };
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    log_compile_stage_timing("wat_comments", stage_start);
    stage_recorder.finish("wat_comments", stage_start_ms);
    Ok(CompilationArtifact {
        wasm: bytes,
        wat_comments,
        nepl_meta_artifact,
        resource_summary_proof_header,
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
        | crate::hir::HirExprKind::MemoizedFunctionValue(_)
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
