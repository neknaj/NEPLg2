use crate::ast::{
    remap_module_file_id, Directive, Effect, EnumDef, FnAlias, FnDef, ImplDef, ImportClause,
    Module, Stmt, StructDef, TraitCapability, TraitDef, TraitRef, TypeExpr, TypeParam, Visibility,
};
use crate::diagnostic::Severity;
use crate::error::CoreError;
use crate::lexer::{self, TokenKind};
use crate::parser;
use crate::source_capability::module_source_capabilities;
use crate::source_map::{CompilerMemoryType, SourceCapabilityUseSite};
use crate::span::FileId;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::result::Result;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
use std::path::{Component, PathBuf};
extern crate std;

pub use crate::source_map::{SourceCapabilities, SourceMap, SourcePath};

const LOADER_SESSION_CACHE_VERSION: &str = "neplg2-loader-session-cache-v1";
const CACHED_MODULE_FILE_ID: FileId = FileId(u32::MAX - 1);

macro_rules! loader_log {
    ($($arg:tt)*) => {
        if crate::log::is_verbose() {
            std::eprintln!($($arg)*);
        }
    };
}

#[derive(Debug)]
pub enum LoaderError {
    Io(String),
    Core(CoreError),
}

impl core::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LoaderError::Io(s) => write!(f, "IO error: {}", s),
            LoaderError::Core(e) => write!(f, "Core error: {:?}", e),
        }
    }
}

impl std::error::Error for LoaderError {}

impl From<CoreError> for LoaderError {
    fn from(e: CoreError) -> Self {
        LoaderError::Core(e)
    }
}

/// Result of loading sources.
#[derive(Debug, Clone)]
pub struct LoadResult {
    pub module: Module,
    pub source_map: SourceMap,
}

/// Cumulative counters for a loader session cache.
///
/// The counters are intentionally about compiler query behavior, not wall-clock
/// time.  A caller can use them to prove that a warm compile avoided stdlib
/// parsing before judging the result with higher-level timing metadata.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LoaderSessionCacheStats {
    pub parsed_module_hits: usize,
    pub parsed_module_misses: usize,
    pub parsed_module_stores: usize,
    pub parsed_module_bypasses: usize,
    pub arity_surface_hits: usize,
    pub arity_surface_misses: usize,
    pub arity_surface_stores: usize,
    pub arity_surface_bypasses: usize,
    pub public_surface_hash_hits: usize,
    pub public_surface_hash_stores: usize,
    pub public_surface_hash_bypasses: usize,
    pub dependency_aggregate_public_surface_hash_hits: usize,
    pub dependency_aggregate_public_surface_hash_misses: usize,
    pub dependency_aggregate_public_surface_hash_stores: usize,
    pub dependency_aggregate_public_surface_hash_bypasses: usize,
    pub stdlib_override_bypasses: usize,
}

/// Session-local cache for pure loader queries.
///
/// This cache is designed for long-lived compiler sessions such as the Web
/// playground's `CompilerSession`.  It keeps only path/hash keyed stdlib
/// artifacts and never stores `SourceMap` or typed HIR state.  Cached parsed
/// modules are normalized to a neutral `FileId` and remapped into the fresh
/// `SourceMap` allocated for each compile, so diagnostic spans and source
/// capabilities still belong to the current load.
#[derive(Debug)]
pub struct LoaderSessionCache {
    namespace_hash: String,
    parsed_modules: BTreeMap<LoaderParsedModuleKey, CachedParsedModule>,
    arity_surfaces: BTreeMap<LoaderAritySurfaceKey, CachedAritySurface>,
    dependency_aggregate_public_surfaces: BTreeMap<LoaderDependencyAggregatePublicSurfaceKey, u64>,
    stats: LoaderSessionCacheStats,
}

impl LoaderSessionCache {
    pub fn new(namespace_hash: impl Into<String>) -> Self {
        Self {
            namespace_hash: namespace_hash.into(),
            parsed_modules: BTreeMap::new(),
            arity_surfaces: BTreeMap::new(),
            dependency_aggregate_public_surfaces: BTreeMap::new(),
            stats: LoaderSessionCacheStats::default(),
        }
    }

    pub fn stats(&self) -> LoaderSessionCacheStats {
        self.stats
    }

    pub fn clear(&mut self) {
        self.parsed_modules.clear();
        self.arity_surfaces.clear();
        self.dependency_aggregate_public_surfaces.clear();
        self.stats = LoaderSessionCacheStats::default();
    }

    pub fn record_stdlib_override_bypass(&mut self) {
        self.stats.stdlib_override_bypasses += 1;
    }

    fn key_for(
        &self,
        canon: &PathBuf,
        src: &str,
        type_arity_hints: &[(String, usize)],
    ) -> LoaderParsedModuleKey {
        LoaderParsedModuleKey {
            cache_version: String::from(LOADER_SESSION_CACHE_VERSION),
            namespace_hash: self.namespace_hash.clone(),
            path: canon.clone(),
            source_hash: fnv1a64(src.as_bytes()),
            type_arity_hints_hash: hash_type_arity_hints(type_arity_hints),
        }
    }

    fn arity_surface_key_for(
        &self,
        stdlib_root: &PathBuf,
        canon: &PathBuf,
        src: &str,
    ) -> LoaderAritySurfaceKey {
        LoaderAritySurfaceKey {
            cache_version: String::from(LOADER_SESSION_CACHE_VERSION),
            namespace_hash: self.namespace_hash.clone(),
            stdlib_root: canonicalize_path(stdlib_root),
            path: canon.clone(),
            source_hash: fnv1a64(src.as_bytes()),
        }
    }

    fn get_parsed_module(&mut self, key: &LoaderParsedModuleKey) -> Option<CachedParsedModule> {
        if let Some(entry) = self.parsed_modules.get(key) {
            self.stats.parsed_module_hits += 1;
            Some(entry.clone())
        } else {
            self.stats.parsed_module_misses += 1;
            None
        }
    }

    fn store_parsed_module(&mut self, key: LoaderParsedModuleKey, entry: CachedParsedModule) {
        self.stats.parsed_module_stores += 1;
        self.parsed_modules.insert(key, entry);
    }

    fn record_parsed_module_bypass(&mut self) {
        self.stats.parsed_module_bypasses += 1;
    }

    fn get_arity_surface(&mut self, key: &LoaderAritySurfaceKey) -> Option<CachedAritySurface> {
        if let Some(entry) = self.arity_surfaces.get(key) {
            self.stats.arity_surface_hits += 1;
            Some(entry.clone())
        } else {
            self.stats.arity_surface_misses += 1;
            None
        }
    }

    fn store_arity_surface(&mut self, key: LoaderAritySurfaceKey, entry: CachedAritySurface) {
        self.stats.arity_surface_stores += 1;
        self.arity_surfaces.insert(key, entry);
    }

    fn record_arity_surface_bypass(&mut self) {
        self.stats.arity_surface_bypasses += 1;
    }

    fn record_public_surface_hash_hit(&mut self, _hash: u64) {
        self.stats.public_surface_hash_hits += 1;
    }

    fn record_public_surface_hash_store(&mut self) {
        self.stats.public_surface_hash_stores += 1;
    }

    fn record_public_surface_hash_bypass(&mut self) {
        self.stats.public_surface_hash_bypasses += 1;
    }

    fn dependency_aggregate_public_surface_key_for(
        &self,
        stdlib_root: &PathBuf,
        canon: &PathBuf,
        module_public_surface_hash: u64,
        dependency_aggregate_public_surface_hash: u64,
    ) -> LoaderDependencyAggregatePublicSurfaceKey {
        LoaderDependencyAggregatePublicSurfaceKey {
            cache_version: String::from(LOADER_SESSION_CACHE_VERSION),
            namespace_hash: self.namespace_hash.clone(),
            stdlib_root: canonicalize_path(stdlib_root),
            path: canon.clone(),
            module_public_surface_hash,
            dependency_aggregate_public_surface_hash,
        }
    }

    fn get_dependency_aggregate_public_surface(
        &mut self,
        key: &LoaderDependencyAggregatePublicSurfaceKey,
    ) -> Option<u64> {
        if let Some(hash) = self.dependency_aggregate_public_surfaces.get(key) {
            self.stats.dependency_aggregate_public_surface_hash_hits += 1;
            Some(*hash)
        } else {
            self.stats.dependency_aggregate_public_surface_hash_misses += 1;
            None
        }
    }

    fn store_dependency_aggregate_public_surface(
        &mut self,
        key: LoaderDependencyAggregatePublicSurfaceKey,
        hash: u64,
    ) {
        self.stats.dependency_aggregate_public_surface_hash_stores += 1;
        self.dependency_aggregate_public_surfaces.insert(key, hash);
    }

    fn record_dependency_aggregate_public_surface_bypass(&mut self) {
        self.stats.dependency_aggregate_public_surface_hash_bypasses += 1;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct LoaderParsedModuleKey {
    cache_version: String,
    namespace_hash: String,
    path: PathBuf,
    source_hash: u64,
    type_arity_hints_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct LoaderAritySurfaceKey {
    cache_version: String,
    namespace_hash: String,
    stdlib_root: PathBuf,
    path: PathBuf,
    source_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct LoaderDependencyAggregatePublicSurfaceKey {
    cache_version: String,
    namespace_hash: String,
    stdlib_root: PathBuf,
    path: PathBuf,
    module_public_surface_hash: u64,
    dependency_aggregate_public_surface_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct LoaderShallowTypeArityKey {
    path: PathBuf,
    source_hash: u64,
}

type ShallowTypeArityHintCache = BTreeMap<LoaderShallowTypeArityKey, Vec<(String, usize)>>;

#[derive(Debug, Clone)]
struct ShallowTypeArityHints {
    hints: Vec<(String, usize)>,
    complete: bool,
}

#[derive(Debug, Clone)]
struct CachedParsedModule {
    module: Module,
    capabilities: SourceCapabilities,
    public_surface_hash: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceImportEdgeKind {
    Prelude,
    Import,
    Include,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceImportEdge {
    kind: SourceImportEdgeKind,
    target_path: PathBuf,
    visibility: Visibility,
    import_clause: Option<ImportClause>,
    source_order: usize,
}

impl SourceImportEdge {
    fn public_reexport_eligible(&self) -> bool {
        match self.kind {
            SourceImportEdgeKind::Include => true,
            SourceImportEdgeKind::Import => {
                self.visibility == Visibility::Pub && self.import_clause.is_some()
            }
            SourceImportEdgeKind::Prelude => false,
        }
    }
}

#[derive(Debug, Clone)]
struct CachedAritySurface {
    local_type_arity_hints: Vec<(String, usize)>,
    edges: Vec<SourceImportEdge>,
    default_prelude_path: PathBuf,
    no_prelude: bool,
    implicit_default_prelude: bool,
}

impl CachedAritySurface {
    fn prewarm_surface_hash(&self, is_root: bool, stdlib_root: &PathBuf) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        fnv1a64_update(&mut hash, LOADER_SESSION_CACHE_VERSION.as_bytes());
        fnv1a64_update(&mut hash, &[0]);
        hash_path(&mut hash, stdlib_root);
        hash_bool(&mut hash, is_root);
        hash_bool(&mut hash, self.no_prelude);
        hash_bool(&mut hash, self.implicit_default_prelude);
        hash_path(&mut hash, &self.default_prelude_path);
        let mut dependency_edges = self.edges.iter().collect::<Vec<_>>();
        dependency_edges.sort_by_key(|edge| edge.source_order);
        for edge in dependency_edges {
            hash_source_import_edge(&mut hash, edge);
        }
        hash
    }

    fn preload_paths(&self, is_root: bool) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let mut seen = BTreeSet::new();
        let has_explicit_prelude = self
            .edges
            .iter()
            .any(|edge| edge.kind == SourceImportEdgeKind::Prelude);
        if is_root && self.implicit_default_prelude && !self.no_prelude && !has_explicit_prelude {
            push_unique_canonical_path(&mut paths, &mut seen, &self.default_prelude_path);
        }
        let mut dependency_edges = self.edges.iter().collect::<Vec<_>>();
        dependency_edges.sort_by_key(|edge| edge.source_order);
        for edge in dependency_edges.into_iter().filter(|edge| {
            matches!(
                edge.kind,
                SourceImportEdgeKind::Prelude
                    | SourceImportEdgeKind::Import
                    | SourceImportEdgeKind::Include
            )
        }) {
            push_unique_canonical_path(&mut paths, &mut seen, &edge.target_path);
        }
        paths
    }

    fn public_reexport_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let mut seen = BTreeSet::new();
        let mut reexport_edges = self.edges.iter().collect::<Vec<_>>();
        reexport_edges.sort_by_key(|edge| edge.source_order);
        for edge in reexport_edges
            .into_iter()
            .filter(|edge| edge.public_reexport_eligible())
        {
            push_unique_canonical_path(&mut paths, &mut seen, &edge.target_path);
        }
        paths
    }
}

/// Loader that builds a single merged module from an entry file,
/// preserving FileId/Span for diagnostics. #include inlines AST;
/// #import loads the module once and inlines its items (simple model).
#[derive(Debug)]
pub struct Loader {
    stdlib_root: PathBuf,
    source_map: SourceMap,
}

impl Loader {
    pub fn new(stdlib_root: PathBuf) -> Self {
        Self {
            stdlib_root: canonicalize_path(&stdlib_root),
            source_map: SourceMap::new(),
        }
    }

    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    /// Load an already-provided source string as a pseudo file (for stdin use).
    pub fn load_inline(&mut self, path: PathBuf, src: String) -> Result<LoadResult, LoaderError> {
        let mut sm = SourceMap::new();
        let mut cache: BTreeMap<PathBuf, Module> = BTreeMap::new();
        let mut processing: BTreeSet<PathBuf> = BTreeSet::new();
        let mut imported: BTreeSet<PathBuf> = BTreeSet::new();
        let mut shallow_type_arity_cache = BTreeMap::new();
        let module = match self.load_from_contents(
            path,
            src,
            &mut sm,
            &mut cache,
            &mut processing,
            &mut imported,
            &mut shallow_type_arity_cache,
            true,
        ) {
            Ok(m) => m,
            Err(e) => {
                self.source_map = sm.clone();
                return Err(e);
            }
        };
        self.source_map = sm.clone();
        Ok(LoadResult {
            module,
            source_map: sm,
        })
    }

    pub fn load_inline_with_provider(
        &mut self,
        path: PathBuf,
        src: String,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
    ) -> Result<LoadResult, LoaderError> {
        loader_log!("[Loader] load_inline_with_provider: path={:?}", path);
        let mut sm = SourceMap::new();
        let mut cache: BTreeMap<PathBuf, Module> = BTreeMap::new();
        let mut processing: BTreeSet<PathBuf> = BTreeSet::new();
        let mut imported: BTreeSet<PathBuf> = BTreeSet::new();
        let mut shallow_type_arity_cache = BTreeMap::new();
        let module = match self.load_from_contents_with(
            path,
            src,
            &mut sm,
            &mut cache,
            &mut processing,
            &mut imported,
            &mut shallow_type_arity_cache,
            true,
            provider,
            None,
        ) {
            Ok(m) => m,
            Err(e) => {
                loader_log!("[Loader] load_inline_with_provider: failed: {:?}", e);
                self.source_map = sm.clone();
                return Err(e);
            }
        };
        loader_log!(
            "[Loader] load_inline_with_provider: success. cache_size={}",
            cache.len()
        );
        self.source_map = sm.clone();
        Ok(LoadResult {
            module,
            source_map: sm,
        })
    }

    pub fn load_inline_with_provider_and_cache(
        &mut self,
        path: PathBuf,
        src: String,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
        session_cache: &mut LoaderSessionCache,
    ) -> Result<LoadResult, LoaderError> {
        loader_log!(
            "[Loader] load_inline_with_provider_and_cache: path={:?}",
            path
        );
        let mut sm = SourceMap::new();
        let mut cache: BTreeMap<PathBuf, Module> = BTreeMap::new();
        let mut processing: BTreeSet<PathBuf> = BTreeSet::new();
        let mut imported: BTreeSet<PathBuf> = BTreeSet::new();
        let mut shallow_type_arity_cache = BTreeMap::new();
        let module = match self.load_from_contents_with(
            path,
            src,
            &mut sm,
            &mut cache,
            &mut processing,
            &mut imported,
            &mut shallow_type_arity_cache,
            true,
            provider,
            Some(session_cache),
        ) {
            Ok(m) => m,
            Err(e) => {
                loader_log!(
                    "[Loader] load_inline_with_provider_and_cache: failed: {:?}",
                    e
                );
                self.source_map = sm.clone();
                return Err(e);
            }
        };
        loader_log!(
            "[Loader] load_inline_with_provider_and_cache: success. cache_size={}",
            cache.len()
        );
        self.source_map = sm.clone();
        Ok(LoadResult {
            module,
            source_map: sm,
        })
    }

    /// Prewarm the session-local provider cache for configured stdlib roots.
    ///
    /// This is a loader-only query warmup. It parses stdlib modules through the
    /// same provider path as normal Web/Node compilation, stores only the
    /// path/hash keyed artifacts owned by `LoaderSessionCache`, and discards the
    /// temporary `SourceMap` and merged modules created during traversal. That
    /// keeps the long-lived cache free of per-compile `FileId`,
    /// `ImportResolution`, typed HIR, and Resource IR state.
    ///
    /// The caller must provide roots derived from the current source/import
    /// graph. Walking the whole bundled stdlib file list would make warmup depend
    /// on packaging details rather than program dependencies and would blur the
    /// invalidation boundary for future prechecked artifacts.
    pub fn prewarm_provider_cache(
        &self,
        roots: &[PathBuf],
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
        session_cache: &mut LoaderSessionCache,
    ) -> Result<usize, LoaderError> {
        let mut sm = SourceMap::new();
        let mut cache: BTreeMap<PathBuf, Module> = BTreeMap::new();
        let mut warmed = 0;
        let mut shallow_type_arity_cache = BTreeMap::new();
        for root in roots {
            let canon = canonicalize_path(root);
            if !self.configured_stdlib_source_path(&canon) {
                session_cache.record_parsed_module_bypass();
                continue;
            }
            let mut processing = BTreeSet::new();
            let mut imported_once = BTreeSet::new();
            // Prewarm starts from an already selected import root.  Facade modules
            // may re-enter that root through public re-export cycles, and normal
            // compile treats that edge as already imported rather than as a hard
            // loader failure.  Mark the root before traversal so warmup mirrors
            // the real import-once boundary.
            imported_once.insert(canon.clone());
            self.load_file_with(
                &canon,
                &mut sm,
                &mut cache,
                &mut processing,
                &mut imported_once,
                &mut shallow_type_arity_cache,
                false,
                provider,
                Some(session_cache),
            )?;
            warmed += 1;
        }
        Ok(warmed)
    }

    /// Prewarm loader queries reachable from a root source's import surface.
    ///
    /// NEPLg2.1 type syntax requires import arity information before the root
    /// module is fully parsed. The same shallow source surface also gives a
    /// stable warmup boundary: default prelude, explicit prelude, import, and
    /// include roots are resolved first, then only configured stdlib roots are
    /// loaded into `LoaderSessionCache`. User modules and stdlib overlays are
    /// left to the normal compile path.
    pub fn prewarm_provider_cache_for_source(
        &self,
        entry_path: PathBuf,
        source: &str,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
        session_cache: &mut LoaderSessionCache,
    ) -> Result<usize, LoaderError> {
        let (_, roots) =
            self.root_prewarm_surface_for_source_with_cache(entry_path, source, session_cache);
        self.prewarm_provider_cache(&roots, provider, session_cache)
    }

    /// Compute the stable loader warmup surface for a root source.
    ///
    /// The hash intentionally describes only the dependency surface that can
    /// affect loader prewarm: root default prelude state plus prelude/import/include
    /// edges with their visibility and import clauses.  Function bodies and local
    /// type declarations are not part of this key, so body-only edits can reuse
    /// the already warmed stdlib loader artifacts.  The value is still computed
    /// from the current source text and contains no `FileId`, `Span`,
    /// `ImportResolution`, typed HIR, or `TypeId`.
    pub fn root_prewarm_surface_for_source(
        &self,
        entry_path: PathBuf,
        source: &str,
    ) -> (u64, Vec<PathBuf>) {
        self.compute_root_prewarm_surface_for_source(entry_path, source, None)
    }

    /// Compute a root prewarm surface while recording loader-cache bypass stats.
    ///
    /// User entry source is intentionally not retained in `LoaderSessionCache`,
    /// but the scan should still be visible in session statistics. This helper is
    /// used by prewarm paths that already own a session cache so the observable
    /// boundary stays the same as normal loader query traversal.
    pub fn root_prewarm_surface_for_source_with_cache(
        &self,
        entry_path: PathBuf,
        source: &str,
        session_cache: &mut LoaderSessionCache,
    ) -> (u64, Vec<PathBuf>) {
        self.compute_root_prewarm_surface_for_source(entry_path, source, Some(session_cache))
    }

    fn compute_root_prewarm_surface_for_source(
        &self,
        entry_path: PathBuf,
        source: &str,
        mut session_cache: Option<&mut LoaderSessionCache>,
    ) -> (u64, Vec<PathBuf>) {
        let canon = canonicalize_path(&entry_path);
        let mut sm = SourceMap::new();
        let file_id = sm.add_with_capabilities(
            path_to_source_label(&canon),
            source.to_string(),
            SourceCapabilities::none(),
        );
        let surface =
            self.source_arity_surface(&canon, file_id, source, session_cache.as_deref_mut());
        let hash = surface.prewarm_surface_hash(true, &self.stdlib_root);
        let roots = surface.preload_paths(true);
        (hash, roots)
    }

    /// Compute the public surface hash of stdlib dependencies reachable from a root source.
    ///
    /// This query is a loader-level staging artifact for future typed public
    /// surface caches.  It folds the root import surface together with the
    /// public surface hashes of reachable configured stdlib modules, but it
    /// deliberately does not retain the root source, `SourceMap`,
    /// `ImportResolution`, typed HIR, `TypeId`, Resource IR, or codegen
    /// fragments.  User modules and stdlib overlays are outside the long-lived
    /// bundled stdlib cache and therefore become conservative bypass edges.
    pub fn root_dependency_aggregate_public_surface_hash_for_source_with_cache(
        &self,
        entry_path: PathBuf,
        source: &str,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
        session_cache: &mut LoaderSessionCache,
    ) -> Result<u64, LoaderError> {
        let canon = canonicalize_path(&entry_path);
        let mut sm = SourceMap::new();
        let file_id = sm.add_with_capabilities(
            path_to_source_label(&canon),
            source.to_string(),
            SourceCapabilities::none(),
        );
        let surface = self.source_arity_surface(&canon, file_id, source, Some(session_cache));
        let mut visiting = BTreeSet::new();
        let mut computed = BTreeMap::new();
        let dependencies = self.dependency_aggregate_public_surface_hashes_for_paths(
            surface.preload_paths(true),
            provider,
            session_cache,
            &mut visiting,
            &mut computed,
        )?;
        let mut hash = FNV_OFFSET_BASIS;
        fnv1a64_update(&mut hash, LOADER_SESSION_CACHE_VERSION.as_bytes());
        hash_str(&mut hash, "root-dependency-public-surface-v1");
        hash_path(&mut hash, &canonicalize_path(&self.stdlib_root));
        hash_public_dependency_surface(&mut hash, &surface);
        hash_dependency_aggregate_public_surface_entries(&mut hash, &dependencies);
        Ok(hash)
    }

    pub fn load(&mut self, entry: &PathBuf) -> Result<LoadResult, LoaderError> {
        let mut sm = SourceMap::new();
        let mut cache: BTreeMap<PathBuf, Module> = BTreeMap::new();
        let mut processing: BTreeSet<PathBuf> = BTreeSet::new();
        let mut imported: BTreeSet<PathBuf> = BTreeSet::new();
        let mut shallow_type_arity_cache = BTreeMap::new();
        let module = match self.load_file(
            entry,
            &mut sm,
            &mut cache,
            &mut processing,
            &mut imported,
            &mut shallow_type_arity_cache,
            true,
        ) {
            Ok(m) => m,
            Err(e) => {
                self.source_map = sm.clone();
                return Err(e);
            }
        };
        self.source_map = sm.clone();
        Ok(LoadResult {
            module,
            source_map: sm,
        })
    }

    fn load_from_contents(
        &self,
        path: PathBuf,
        src: String,
        sm: &mut SourceMap,
        cache: &mut BTreeMap<PathBuf, Module>,
        processing: &mut BTreeSet<PathBuf>,
        imported_once: &mut BTreeSet<PathBuf>,
        shallow_type_arity_cache: &mut ShallowTypeArityHintCache,
        is_root: bool,
    ) -> Result<Module, LoaderError> {
        // For pseudo files (stdin) canonicalize may fail; fall back to provided path.
        let canon = canonicalize_path(&path);
        if let Some(m) = cache.get(&canon) {
            return Ok(m.clone());
        }
        if !processing.insert(canon.clone()) {
            return Err(LoaderError::Io(format!(
                "circular import/include detected at {:?}",
                canon
            )));
        }
        let file_id = sm.add_with_capabilities(
            path_to_source_label(&canon),
            src.clone(),
            SourceCapabilities::none(),
        );
        let type_arity_hints = self.imported_type_arity_hints(
            &canon,
            file_id,
            &src,
            sm,
            cache,
            processing,
            imported_once,
            shallow_type_arity_cache,
            is_root,
        )?;
        let module = self.parse_module_with_type_arity_hints(file_id, src, type_arity_hints)?;
        sm.set_capabilities(
            file_id,
            self.source_capabilities_for_module(&canon, &module),
        );
        let module = self.process_directives(
            canon.clone(),
            module,
            sm,
            cache,
            processing,
            imported_once,
            shallow_type_arity_cache,
            is_root,
        )?;
        processing.remove(&canon);
        cache.insert(canon.clone(), module.clone());
        Ok(module)
    }

    fn load_from_contents_with(
        &self,
        path: PathBuf,
        src: String,
        sm: &mut SourceMap,
        cache: &mut BTreeMap<PathBuf, Module>,
        processing: &mut BTreeSet<PathBuf>,
        imported_once: &mut BTreeSet<PathBuf>,
        shallow_type_arity_cache: &mut ShallowTypeArityHintCache,
        is_root: bool,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
        mut session_cache: Option<&mut LoaderSessionCache>,
    ) -> Result<Module, LoaderError> {
        let canon = canonicalize_path(&path);
        loader_log!(
            "[Loader] load_from_contents_with: path={:?}, canon={:?}",
            path,
            canon
        );
        if let Some(m) = cache.get(&canon) {
            return Ok(m.clone());
        }
        if !processing.insert(canon.clone()) {
            return Err(LoaderError::Io(format!(
                "circular import/include detected at {:?}",
                canon
            )));
        }
        let file_id = sm.add_with_capabilities(
            path_to_source_label(&canon),
            src.clone(),
            SourceCapabilities::none(),
        );
        let type_arity_hints = self.imported_type_arity_hints_with(
            &canon,
            file_id,
            &src,
            sm,
            cache,
            processing,
            imported_once,
            shallow_type_arity_cache,
            is_root,
            provider,
            session_cache.as_deref_mut(),
        )?;
        let module = self.parse_module_with_type_arity_hints(file_id, src, type_arity_hints)?;
        sm.set_capabilities(
            file_id,
            self.source_capabilities_for_module(&canon, &module),
        );
        loader_log!("[Loader] processing directives for {:?}", canon);
        let module = self.process_directives_with(
            canon.clone(),
            module,
            sm,
            cache,
            processing,
            imported_once,
            shallow_type_arity_cache,
            is_root,
            provider,
            session_cache.as_deref_mut(),
        )?;
        processing.remove(&canon);
        cache.insert(canon.clone(), module.clone());
        loader_log!("[Loader] finished contents for {:?}", canon);
        Ok(module)
    }

    fn load_file(
        &self,
        path: &PathBuf,
        sm: &mut SourceMap,
        cache: &mut BTreeMap<PathBuf, Module>,
        processing: &mut BTreeSet<PathBuf>,
        imported_once: &mut BTreeSet<PathBuf>,
        shallow_type_arity_cache: &mut ShallowTypeArityHintCache,
        is_root: bool,
    ) -> Result<Module, LoaderError> {
        let canon = canonicalize_path(&path);
        if let Some(m) = cache.get(&canon) {
            return Ok(m.clone());
        }
        if !processing.insert(canon.clone()) {
            return Err(LoaderError::Io(format!(
                "circular import/include detected at {:?}",
                canon
            )));
        }
        loader_log!("[Loader] Loading file: {:?}", canon);
        let src = read_file_to_string(&canon)?;
        let file_id = sm.add_with_capabilities(
            path_to_source_label(&canon),
            src.clone(),
            SourceCapabilities::none(),
        );
        loader_log!("[Loader] Parsing module: {:?}", canon);
        let type_arity_hints = self.imported_type_arity_hints(
            &canon,
            file_id,
            &src,
            sm,
            cache,
            processing,
            imported_once,
            shallow_type_arity_cache,
            is_root,
        )?;
        let module = self.parse_module_with_type_arity_hints(file_id, src, type_arity_hints)?;
        sm.set_capabilities(
            file_id,
            self.source_capabilities_for_module(&canon, &module),
        );
        loader_log!("[Loader] Processing directives for: {:?}", canon);
        let module = self.process_directives(
            canon.clone(),
            module,
            sm,
            cache,
            processing,
            imported_once,
            shallow_type_arity_cache,
            is_root,
        )?;
        loader_log!("[Loader] Finished loading: {:?}", canon);
        processing.remove(&canon);
        cache.insert(canon.clone(), module.clone());
        Ok(module)
    }

    fn load_file_with(
        &self,
        path: &PathBuf,
        sm: &mut SourceMap,
        cache: &mut BTreeMap<PathBuf, Module>,
        processing: &mut BTreeSet<PathBuf>,
        imported_once: &mut BTreeSet<PathBuf>,
        shallow_type_arity_cache: &mut ShallowTypeArityHintCache,
        is_root: bool,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
        mut session_cache: Option<&mut LoaderSessionCache>,
    ) -> Result<Module, LoaderError> {
        let canon = canonicalize_path(&path);
        if let Some(m) = cache.get(&canon) {
            return Ok(m.clone());
        }
        if !processing.insert(canon.clone()) {
            return Err(LoaderError::Io(format!(
                "circular import/include detected at {:?}",
                canon
            )));
        }
        let src = provider(&canon)?;
        let file_id = sm.add_with_capabilities(
            path_to_source_label(&canon),
            src.clone(),
            SourceCapabilities::none(),
        );
        let type_arity_hints = self.imported_type_arity_hints_with(
            &canon,
            file_id,
            &src,
            sm,
            cache,
            processing,
            imported_once,
            shallow_type_arity_cache,
            is_root,
            provider,
            session_cache.as_deref_mut(),
        )?;
        let public_surface_dependency = if self.configured_stdlib_source_path(&canon) {
            Some(self.source_arity_surface(&canon, file_id, &src, session_cache.as_deref_mut()))
        } else {
            None
        };
        let (module, capabilities) = self.parse_provider_module_with_session_cache(
            &canon,
            file_id,
            src,
            type_arity_hints,
            public_surface_dependency.as_ref(),
            session_cache.as_deref_mut(),
        )?;
        sm.set_capabilities(file_id, capabilities);
        let module = self.process_directives_with(
            canon.clone(),
            module,
            sm,
            cache,
            processing,
            imported_once,
            shallow_type_arity_cache,
            is_root,
            provider,
            session_cache.as_deref_mut(),
        )?;
        processing.remove(&canon);
        cache.insert(canon.clone(), module.clone());
        Ok(module)
    }

    fn imported_type_arity_hints(
        &self,
        base: &PathBuf,
        file_id: FileId,
        src: &str,
        _sm: &mut SourceMap,
        _cache: &mut BTreeMap<PathBuf, Module>,
        processing: &mut BTreeSet<PathBuf>,
        _imported_once: &BTreeSet<PathBuf>,
        shallow_type_arity_cache: &mut ShallowTypeArityHintCache,
        is_root: bool,
    ) -> Result<Vec<(String, usize)>, LoaderError> {
        // NEPLg2.1 prefix type annotations need type-constructor arities before
        // the importing module can be parsed. The arity query must stay shallow:
        // it reads declaration heads from direct dependencies and public
        // facade re-exports, but it does not run the normal loader. A full
        // load would cache context-dependent merged modules before the real
        // import pass and can multiply diamond stdlib graphs into very large
        // merged ASTs.
        let paths = self.type_arity_preload_paths(base, file_id, src, is_root);
        let mut hints = Vec::new();
        for path in paths {
            let mut visited = BTreeSet::new();
            let canon = canonicalize_path(&path);
            if processing.contains(&canon) {
                let shallow_hints = self.shallow_type_arity_hints_from_file(
                    &path,
                    file_id,
                    &mut visited,
                    shallow_type_arity_cache,
                )?;
                push_loader_type_arity_hints(&mut hints, shallow_hints.hints);
                continue;
            }
            let shallow_hints = self.shallow_type_arity_hints_from_file(
                &canon,
                file_id,
                &mut visited,
                shallow_type_arity_cache,
            )?;
            push_loader_type_arity_hints(&mut hints, shallow_hints.hints);
        }
        Ok(hints)
    }

    fn imported_type_arity_hints_with(
        &self,
        base: &PathBuf,
        file_id: FileId,
        src: &str,
        _sm: &mut SourceMap,
        _cache: &mut BTreeMap<PathBuf, Module>,
        processing: &mut BTreeSet<PathBuf>,
        _imported_once: &BTreeSet<PathBuf>,
        shallow_type_arity_cache: &mut ShallowTypeArityHintCache,
        is_root: bool,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
        mut session_cache: Option<&mut LoaderSessionCache>,
    ) -> Result<Vec<(String, usize)>, LoaderError> {
        // Provider-backed loading has the same shallow arity contract as
        // filesystem loading. Every dependency source comes from the caller's
        // virtual file provider, but this query still avoids normal loading so
        // Web sessions do not cache merged modules from a speculative preload
        // context.
        let paths = self.type_arity_preload_paths_with_cache(
            base,
            file_id,
            src,
            is_root,
            session_cache.as_deref_mut(),
        );
        let mut hints = Vec::new();
        for path in paths {
            let mut visited = BTreeSet::new();
            let canon = canonicalize_path(&path);
            if processing.contains(&canon) {
                let shallow_hints = self.shallow_type_arity_hints_from_file_with(
                    &canon,
                    file_id,
                    &mut visited,
                    shallow_type_arity_cache,
                    provider,
                    session_cache.as_deref_mut(),
                )?;
                push_loader_type_arity_hints(&mut hints, shallow_hints.hints);
                continue;
            }
            let shallow_hints = self.shallow_type_arity_hints_from_file_with(
                &canon,
                file_id,
                &mut visited,
                shallow_type_arity_cache,
                provider,
                session_cache.as_deref_mut(),
            )?;
            push_loader_type_arity_hints(&mut hints, shallow_hints.hints);
        }
        Ok(hints)
    }

    fn shallow_type_arity_hints_from_file(
        &self,
        path: &PathBuf,
        file_id: FileId,
        visited: &mut BTreeSet<PathBuf>,
        shallow_type_arity_cache: &mut ShallowTypeArityHintCache,
    ) -> Result<ShallowTypeArityHints, LoaderError> {
        let canon = canonicalize_path(path);
        if !visited.insert(canon.clone()) {
            return Ok(ShallowTypeArityHints {
                hints: Vec::new(),
                complete: false,
            });
        }
        let src = read_file_to_string(&canon)?;
        let key = LoaderShallowTypeArityKey {
            path: canon.clone(),
            source_hash: fnv1a64(src.as_bytes()),
        };
        if let Some(hints) = shallow_type_arity_cache.get(&key) {
            return Ok(ShallowTypeArityHints {
                hints: hints.clone(),
                complete: true,
            });
        }
        let hints = self.shallow_type_arity_hints_from_source(
            &canon,
            file_id,
            &src,
            visited,
            shallow_type_arity_cache,
        );
        if hints.complete {
            shallow_type_arity_cache.insert(key, hints.hints.clone());
        }
        Ok(hints)
    }

    fn shallow_type_arity_hints_from_file_with(
        &self,
        path: &PathBuf,
        file_id: FileId,
        visited: &mut BTreeSet<PathBuf>,
        shallow_type_arity_cache: &mut ShallowTypeArityHintCache,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
        session_cache: Option<&mut LoaderSessionCache>,
    ) -> Result<ShallowTypeArityHints, LoaderError> {
        let canon = canonicalize_path(path);
        if !visited.insert(canon.clone()) {
            return Ok(ShallowTypeArityHints {
                hints: Vec::new(),
                complete: false,
            });
        }
        let src = provider(&canon)?;
        let key = LoaderShallowTypeArityKey {
            path: canon.clone(),
            source_hash: fnv1a64(src.as_bytes()),
        };
        if let Some(hints) = shallow_type_arity_cache.get(&key) {
            return Ok(ShallowTypeArityHints {
                hints: hints.clone(),
                complete: true,
            });
        }
        let hints = self.shallow_type_arity_hints_from_source_with(
            &canon,
            file_id,
            &src,
            visited,
            shallow_type_arity_cache,
            provider,
            session_cache,
        )?;
        if hints.complete {
            shallow_type_arity_cache.insert(key, hints.hints.clone());
        }
        Ok(hints)
    }

    fn shallow_type_arity_hints_from_source(
        &self,
        canon: &PathBuf,
        file_id: FileId,
        src: &str,
        visited: &mut BTreeSet<PathBuf>,
        shallow_type_arity_cache: &mut ShallowTypeArityHintCache,
    ) -> ShallowTypeArityHints {
        // A module that is already on the import stack cannot be fully parsed
        // again, but its declaration heads and public re-export/include
        // declaration heads are still enough kind metadata for NEPLg2.1 prefix
        // type parsing. Private implementation imports are intentionally not
        // followed here: exposing them to parser-facing type annotations would
        // both leak module boundaries and multiply stdlib implementation
        // graphs during speculative arity discovery.
        let mut hints = parser::type_arity_hints_from_source(file_id, src);
        let mut complete = true;
        for dep in self.shallow_type_arity_dependency_paths(canon, file_id, src) {
            let Ok(dep_hints) = self.shallow_type_arity_hints_from_file(
                &dep,
                file_id,
                visited,
                shallow_type_arity_cache,
            ) else {
                continue;
            };
            complete &= dep_hints.complete;
            push_loader_type_arity_hints(&mut hints, dep_hints.hints);
        }
        ShallowTypeArityHints { hints, complete }
    }

    fn shallow_type_arity_hints_from_source_with(
        &self,
        canon: &PathBuf,
        file_id: FileId,
        src: &str,
        visited: &mut BTreeSet<PathBuf>,
        shallow_type_arity_cache: &mut ShallowTypeArityHintCache,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
        mut session_cache: Option<&mut LoaderSessionCache>,
    ) -> Result<ShallowTypeArityHints, LoaderError> {
        let surface = self.source_arity_surface(canon, file_id, src, session_cache.as_deref_mut());
        let dependency_paths = surface.public_reexport_paths();
        let mut hints = surface.local_type_arity_hints;
        let mut complete = true;
        for dep in dependency_paths {
            let dep_hints = self.shallow_type_arity_hints_from_file_with(
                &dep,
                file_id,
                visited,
                shallow_type_arity_cache,
                provider,
                session_cache.as_deref_mut(),
            )?;
            complete &= dep_hints.complete;
            push_loader_type_arity_hints(&mut hints, dep_hints.hints);
        }
        Ok(ShallowTypeArityHints { hints, complete })
    }

    fn shallow_type_arity_dependency_paths(
        &self,
        base: &PathBuf,
        file_id: FileId,
        src: &str,
    ) -> Vec<PathBuf> {
        // Cycle recovery must stay shallow. Parser-facing type arities follow
        // public facade edges and includes, but not private implementation
        // imports. That keeps `%FacadeType ...` parseable through public
        // re-exports without making every implementation helper's private
        // dependencies visible to unrelated importers.
        self.compute_source_arity_surface(base, file_id, src)
            .public_reexport_paths()
    }

    fn type_arity_preload_paths(
        &self,
        base: &PathBuf,
        file_id: FileId,
        src: &str,
        is_root: bool,
    ) -> Vec<PathBuf> {
        self.type_arity_preload_paths_with_cache(base, file_id, src, is_root, None)
    }

    fn type_arity_preload_paths_with_cache(
        &self,
        base: &PathBuf,
        file_id: FileId,
        src: &str,
        is_root: bool,
        session_cache: Option<&mut LoaderSessionCache>,
    ) -> Vec<PathBuf> {
        self.source_arity_surface(base, file_id, src, session_cache)
            .preload_paths(is_root)
    }

    fn dependency_aggregate_public_surface_hashes_for_paths(
        &self,
        paths: Vec<PathBuf>,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
        session_cache: &mut LoaderSessionCache,
        visiting: &mut BTreeSet<PathBuf>,
        computed: &mut BTreeMap<PathBuf, u64>,
    ) -> Result<Vec<(PathBuf, u64)>, LoaderError> {
        let mut entries = Vec::new();
        for path in paths {
            let canon = canonicalize_path(&path);
            let hash = self.dependency_aggregate_public_surface_hash_for_path_with(
                &canon,
                provider,
                session_cache,
                visiting,
                computed,
            )?;
            entries.push((canon, hash));
        }
        Ok(entries)
    }

    fn dependency_aggregate_public_surface_hash_for_path_with(
        &self,
        canon: &PathBuf,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
        session_cache: &mut LoaderSessionCache,
        visiting: &mut BTreeSet<PathBuf>,
        computed: &mut BTreeMap<PathBuf, u64>,
    ) -> Result<u64, LoaderError> {
        if let Some(hash) = computed.get(canon) {
            return Ok(*hash);
        }
        if !self.configured_stdlib_source_path(canon) {
            session_cache.record_dependency_aggregate_public_surface_bypass();
            let hash = external_dependency_aggregate_public_surface_hash(canon);
            computed.insert(canon.clone(), hash);
            return Ok(hash);
        }

        if !visiting.insert(canon.clone()) {
            session_cache.record_dependency_aggregate_public_surface_bypass();
            let src = provider(canon)?;
            return Ok(cyclic_dependency_aggregate_public_surface_hash(canon, &src));
        }

        let src = provider(canon)?;
        let mut sm = SourceMap::new();
        let file_id = sm.add_with_capabilities(
            path_to_source_label(canon),
            src.clone(),
            SourceCapabilities::none(),
        );
        let surface = self.source_arity_surface(canon, file_id, &src, Some(session_cache));
        let dependencies = self.dependency_aggregate_public_surface_hashes_for_paths(
            surface.preload_paths(false),
            provider,
            session_cache,
            visiting,
            computed,
        )?;

        let mut child_hash = FNV_OFFSET_BASIS;
        fnv1a64_update(&mut child_hash, LOADER_SESSION_CACHE_VERSION.as_bytes());
        hash_str(&mut child_hash, "dependency-public-surface-children-v1");
        hash_dependency_aggregate_public_surface_entries(&mut child_hash, &dependencies);

        let mut module_sm = SourceMap::new();
        let module_file_id = module_sm.add_with_capabilities(
            path_to_source_label(canon),
            src.clone(),
            SourceCapabilities::none(),
        );
        let mut module_cache = BTreeMap::new();
        let mut processing = BTreeSet::new();
        processing.insert(canon.clone());
        let imported_once = BTreeSet::new();
        let mut shallow_type_arity_cache = BTreeMap::new();
        let type_arity_hints = self.imported_type_arity_hints_with(
            canon,
            module_file_id,
            &src,
            &mut module_sm,
            &mut module_cache,
            &mut processing,
            &imported_once,
            &mut shallow_type_arity_cache,
            false,
            provider,
            Some(session_cache),
        )?;
        let (module, _) = self.parse_provider_module_with_session_cache(
            canon,
            module_file_id,
            src,
            type_arity_hints,
            Some(&surface),
            Some(session_cache),
        )?;
        let module_hash = module_public_surface_hash(&module, Some(&surface));
        let key = session_cache.dependency_aggregate_public_surface_key_for(
            &self.stdlib_root,
            canon,
            module_hash,
            child_hash,
        );
        if let Some(hash) = session_cache.get_dependency_aggregate_public_surface(&key) {
            visiting.remove(canon);
            computed.insert(canon.clone(), hash);
            return Ok(hash);
        }

        let mut aggregate_hash = FNV_OFFSET_BASIS;
        fnv1a64_update(&mut aggregate_hash, LOADER_SESSION_CACHE_VERSION.as_bytes());
        hash_str(&mut aggregate_hash, "dependency-public-surface-v1");
        hash_path(&mut aggregate_hash, canon);
        hash_u64(&mut aggregate_hash, module_hash);
        hash_u64(&mut aggregate_hash, child_hash);
        hash_dependency_aggregate_public_surface_entries(&mut aggregate_hash, &dependencies);
        session_cache.store_dependency_aggregate_public_surface(key, aggregate_hash);
        visiting.remove(canon);
        computed.insert(canon.clone(), aggregate_hash);
        Ok(aggregate_hash)
    }

    fn source_arity_surface(
        &self,
        base: &PathBuf,
        file_id: FileId,
        src: &str,
        session_cache: Option<&mut LoaderSessionCache>,
    ) -> CachedAritySurface {
        let canon = canonicalize_path(base);
        let Some(session_cache) = session_cache else {
            return self.compute_source_arity_surface(&canon, file_id, src);
        };

        if !self.configured_stdlib_source_path(&canon) {
            session_cache.record_arity_surface_bypass();
            return self.compute_source_arity_surface(&canon, file_id, src);
        }

        let key = session_cache.arity_surface_key_for(&self.stdlib_root, &canon, src);
        if let Some(surface) = session_cache.get_arity_surface(&key) {
            return surface;
        }
        let surface = self.compute_source_arity_surface(&canon, file_id, src);
        session_cache.store_arity_surface(key, surface.clone());
        surface
    }

    fn compute_source_arity_surface(
        &self,
        base: &PathBuf,
        file_id: FileId,
        src: &str,
    ) -> CachedAritySurface {
        // This scan reads only file-level dependency directives and declaration
        // heads. Full directive validation, import-clause visibility, and body
        // diagnostics remain the responsibility of the normal parser and
        // `process_directives` path; this artifact is only kind metadata for
        // NEPLg2.1 prefix type boundaries.
        let lex = lexer::lex(file_id, src);
        if lex
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            return CachedAritySurface {
                local_type_arity_hints: Vec::new(),
                edges: Vec::new(),
                default_prelude_path: self.resolve_path(base, "std/prelude_base"),
                no_prelude: false,
                implicit_default_prelude: false,
            };
        }

        let mut edges = Vec::new();
        let mut no_prelude = false;
        for token in &lex.tokens {
            match &token.kind {
                TokenKind::DirPrelude(path) => {
                    edges.push(SourceImportEdge {
                        kind: SourceImportEdgeKind::Prelude,
                        target_path: self.resolve_path(base, path),
                        visibility: Visibility::Private,
                        import_clause: None,
                        source_order: edges.len(),
                    });
                }
                TokenKind::DirNoPrelude => no_prelude = true,
                TokenKind::DirImport(text) => {
                    let (path, clause, visibility) = parser::parse_import_directive_parts(text);
                    if !path.is_empty() {
                        edges.push(SourceImportEdge {
                            kind: SourceImportEdgeKind::Import,
                            target_path: self.resolve_path(base, &path),
                            visibility,
                            import_clause: Some(clause),
                            source_order: edges.len(),
                        });
                    }
                }
                TokenKind::DirInclude(path) => {
                    edges.push(SourceImportEdge {
                        kind: SourceImportEdgeKind::Include,
                        target_path: self.resolve_path(base, path),
                        visibility: Visibility::Pub,
                        import_clause: None,
                        source_order: edges.len(),
                    });
                }
                _ => {}
            }
        }

        CachedAritySurface {
            local_type_arity_hints: parser::type_arity_hints_from_tokens(&lex.tokens),
            edges,
            default_prelude_path: self.resolve_path(base, "std/prelude_base"),
            no_prelude,
            implicit_default_prelude: true,
        }
    }

    fn process_directives(
        &self,
        base: PathBuf,
        module: Module,
        sm: &mut SourceMap,
        cache: &mut BTreeMap<PathBuf, Module>,
        processing: &mut BTreeSet<PathBuf>,
        imported_once: &mut BTreeSet<PathBuf>,
        shallow_type_arity_cache: &mut ShallowTypeArityHintCache,
        is_root: bool,
    ) -> Result<Module, LoaderError> {
        let mut directives = module.directives.clone();
        let mut items = Vec::new();
        let mut prelude_paths = Vec::new();
        let mut no_prelude = false;
        for d in &module.directives {
            match d {
                Directive::Prelude { path, .. } => prelude_paths.push(path.clone()),
                Directive::NoPrelude { .. } => no_prelude = true,
                _ => {}
            }
        }
        if is_root && !no_prelude && prelude_paths.is_empty() {
            prelude_paths.push(String::from("std/prelude_base"));
        }
        for path in prelude_paths {
            let target = self.resolve_path(&base, &path);
            if import_not_seen(imported_once, &target) {
                let imp_mod = self.load_file(
                    &target,
                    sm,
                    cache,
                    processing,
                    imported_once,
                    shallow_type_arity_cache,
                    false,
                )?;
                for d in imp_mod.directives.clone() {
                    if let Directive::Entry { .. } = d {
                        continue;
                    }
                    if let Directive::Target { .. } = d {
                        continue;
                    }
                    if let Directive::IndentWidth { .. } = d {
                        continue;
                    }
                    directives.push(d);
                }
                for it in imp_mod.root.items.clone() {
                    if let Stmt::Directive(Directive::Entry { .. }) = it {
                        continue;
                    }
                    if let Stmt::Directive(Directive::Target { .. }) = it {
                        continue;
                    }
                    if let Stmt::Directive(Directive::IndentWidth { .. }) = it {
                        continue;
                    }
                    items.push(it);
                }
            }
        }
        for stmt in module.root.items.clone() {
            match &stmt {
                Stmt::Directive(Directive::Import { path, .. }) => {
                    let target = self.resolve_path(&base, path);
                    if import_not_seen(imported_once, &target) {
                        let imp_mod = self.load_file(
                            &target,
                            sm,
                            cache,
                            processing,
                            imported_once,
                            shallow_type_arity_cache,
                            false,
                        )?;
                        // Propagate non-file-scoped directives (e.g., externs) so
                        // symbols declared in stdlib become visible to the parent
                        // module during later compilation phases.
                        for d in imp_mod.directives.clone() {
                            if let Directive::Entry { .. } = d {
                                continue;
                            }
                            if let Directive::Target { .. } = d {
                                continue;
                            }
                            if let Directive::IndentWidth { .. } = d {
                                continue;
                            }
                            directives.push(d);
                        }
                        // Do not propagate file-scoped directives like #entry/#target/#indent
                        for it in imp_mod.root.items.clone() {
                            if let Stmt::Directive(Directive::Entry { .. }) = it {
                                continue;
                            }
                            if let Stmt::Directive(Directive::Target { .. }) = it {
                                continue;
                            }
                            if let Stmt::Directive(Directive::IndentWidth { .. }) = it {
                                continue;
                            }
                            items.push(it);
                        }
                    }
                }
                Stmt::Directive(Directive::Include { path, .. }) => {
                    let target = self.resolve_path(&base, path);
                    let inc_mod = self.load_file(
                        &target,
                        sm,
                        cache,
                        processing,
                        imported_once,
                        shallow_type_arity_cache,
                        false,
                    )?;
                    // Propagate non-file-scoped directives from included modules as well.
                    for d in inc_mod.directives.clone() {
                        if let Directive::Entry { .. } = d {
                            continue;
                        }
                        if let Directive::Target { .. } = d {
                            continue;
                        }
                        if let Directive::IndentWidth { .. } = d {
                            continue;
                        }
                        directives.push(d);
                    }
                    for it in inc_mod.root.items.clone() {
                        if let Stmt::Directive(Directive::Entry { .. }) = it {
                            continue;
                        }
                        if let Stmt::Directive(Directive::Target { .. }) = it {
                            continue;
                        }
                        if let Stmt::Directive(Directive::IndentWidth { .. }) = it {
                            continue;
                        }
                        items.push(it);
                    }
                }
                _ => items.push(stmt),
            }
        }
        let mut module = module.clone();
        module.directives = directives;
        module.root.items = items;
        Ok(module)
    }

    fn process_directives_with(
        &self,
        base: PathBuf,
        module: Module,
        sm: &mut SourceMap,
        cache: &mut BTreeMap<PathBuf, Module>,
        processing: &mut BTreeSet<PathBuf>,
        imported_once: &mut BTreeSet<PathBuf>,
        shallow_type_arity_cache: &mut ShallowTypeArityHintCache,
        is_root: bool,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
        mut session_cache: Option<&mut LoaderSessionCache>,
    ) -> Result<Module, LoaderError> {
        let mut directives = module.directives.clone();
        let mut items = Vec::new();
        let mut prelude_paths = Vec::new();
        let mut no_prelude = false;
        for d in &module.directives {
            match d {
                Directive::Prelude { path, .. } => prelude_paths.push(path.clone()),
                Directive::NoPrelude { .. } => no_prelude = true,
                _ => {}
            }
        }
        if is_root && !no_prelude && prelude_paths.is_empty() {
            prelude_paths.push(String::from("std/prelude_base"));
        }
        for path in prelude_paths {
            let target = self.resolve_path(&base, &path);
            if import_not_seen(imported_once, &target) {
                let imp_mod = self.load_file_with(
                    &target,
                    sm,
                    cache,
                    processing,
                    imported_once,
                    shallow_type_arity_cache,
                    false,
                    provider,
                    session_cache.as_deref_mut(),
                )?;
                for d in imp_mod.directives.clone() {
                    if let Directive::Entry { .. } = d {
                        continue;
                    }
                    if let Directive::Target { .. } = d {
                        continue;
                    }
                    if let Directive::IndentWidth { .. } = d {
                        continue;
                    }
                    directives.push(d);
                }
                for it in imp_mod.root.items.clone() {
                    if let Stmt::Directive(Directive::Entry { .. }) = it {
                        continue;
                    }
                    if let Stmt::Directive(Directive::Target { .. }) = it {
                        continue;
                    }
                    if let Stmt::Directive(Directive::IndentWidth { .. }) = it {
                        continue;
                    }
                    items.push(it);
                }
            }
        }
        for stmt in module.root.items.clone() {
            match &stmt {
                Stmt::Directive(Directive::Import { path, .. }) => {
                    let target = self.resolve_path(&base, path);
                    if import_not_seen(imported_once, &target) {
                        let imp_mod = self.load_file_with(
                            &target,
                            sm,
                            cache,
                            processing,
                            imported_once,
                            shallow_type_arity_cache,
                            false,
                            provider,
                            session_cache.as_deref_mut(),
                        )?;
                        for d in imp_mod.directives.clone() {
                            if let Directive::Entry { .. } = d {
                                continue;
                            }
                            if let Directive::Target { .. } = d {
                                continue;
                            }
                            if let Directive::IndentWidth { .. } = d {
                                continue;
                            }
                            directives.push(d);
                        }
                        for it in imp_mod.root.items.clone() {
                            if let Stmt::Directive(Directive::Entry { .. }) = it {
                                continue;
                            }
                            if let Stmt::Directive(Directive::Target { .. }) = it {
                                continue;
                            }
                            if let Stmt::Directive(Directive::IndentWidth { .. }) = it {
                                continue;
                            }
                            items.push(it);
                        }
                    }
                }
                Stmt::Directive(Directive::Include { path, .. }) => {
                    let target = self.resolve_path(&base, path);
                    let inc_mod = self.load_file_with(
                        &target,
                        sm,
                        cache,
                        processing,
                        imported_once,
                        shallow_type_arity_cache,
                        false,
                        provider,
                        session_cache.as_deref_mut(),
                    )?;
                    for d in inc_mod.directives.clone() {
                        if let Directive::Entry { .. } = d {
                            continue;
                        }
                        if let Directive::Target { .. } = d {
                            continue;
                        }
                        if let Directive::IndentWidth { .. } = d {
                            continue;
                        }
                        directives.push(d);
                    }
                    for it in inc_mod.root.items.clone() {
                        if let Stmt::Directive(Directive::Entry { .. }) = it {
                            continue;
                        }
                        if let Stmt::Directive(Directive::Target { .. }) = it {
                            continue;
                        }
                        if let Stmt::Directive(Directive::IndentWidth { .. }) = it {
                            continue;
                        }
                        items.push(it);
                    }
                }
                _ => items.push(stmt),
            }
        }
        let mut module = module.clone();
        module.directives = directives;
        module.root.items = items;
        Ok(module)
    }

    fn parse_provider_module_with_session_cache(
        &self,
        canon: &PathBuf,
        file_id: FileId,
        src: String,
        type_arity_hints: Vec<(String, usize)>,
        public_surface_dependency: Option<&CachedAritySurface>,
        session_cache: Option<&mut LoaderSessionCache>,
    ) -> Result<(Module, SourceCapabilities), CoreError> {
        let Some(session_cache) = session_cache else {
            let module = self.parse_module_with_type_arity_hints(file_id, src, type_arity_hints)?;
            let capabilities = self.source_capabilities_for_module(canon, &module);
            return Ok((module, capabilities));
        };

        if !self.configured_stdlib_source_path(canon) {
            session_cache.record_parsed_module_bypass();
            session_cache.record_public_surface_hash_bypass();
            let module = self.parse_module_with_type_arity_hints(file_id, src, type_arity_hints)?;
            let capabilities = self.source_capabilities_for_module(canon, &module);
            return Ok((module, capabilities));
        }

        let key = session_cache.key_for(canon, &src, &type_arity_hints);
        if let Some(entry) = session_cache.get_parsed_module(&key) {
            let mut module = entry.module;
            remap_module_file_id(&mut module, CACHED_MODULE_FILE_ID, file_id);
            session_cache.record_public_surface_hash_hit(entry.public_surface_hash);
            return Ok((module, entry.capabilities));
        }

        let module = self.parse_module_with_type_arity_hints(file_id, src, type_arity_hints)?;
        let capabilities = self.source_capabilities_for_module(canon, &module);
        let public_surface_hash = module_public_surface_hash(&module, public_surface_dependency);
        let mut cached_module = module.clone();
        remap_module_file_id(&mut cached_module, file_id, CACHED_MODULE_FILE_ID);
        session_cache.record_public_surface_hash_store();
        session_cache.store_parsed_module(
            key,
            CachedParsedModule {
                module: cached_module,
                capabilities: capabilities.clone(),
                public_surface_hash,
            },
        );
        Ok((module, capabilities))
    }

    fn parse_module_with_type_arity_hints(
        &self,
        file_id: FileId,
        src: String,
        type_arity_hints: Vec<(String, usize)>,
    ) -> Result<Module, CoreError> {
        let lex = lexer::lex(file_id, &src);
        if lex
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
        {
            return Err(CoreError::from_diagnostics(lex.diagnostics));
        }
        let parse = parser::parse_tokens_with_type_arity_hints(file_id, lex, type_arity_hints);
        if parse
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
        {
            return Err(CoreError::from_diagnostics(parse.diagnostics));
        }
        if let Some(module) = parse.module {
            Ok(module)
        } else {
            Err(CoreError::from_diagnostics(parse.diagnostics))
        }
    }

    fn resolve_path(&self, base: &PathBuf, spec: &str) -> PathBuf {
        let is_std_import = !spec.starts_with('.') && !spec.starts_with('/');
        let mut p = if is_std_import {
            self.stdlib_root.join(spec)
        } else {
            base.parent()
                .map(|p| p.join(spec))
                .unwrap_or_else(|| PathBuf::from(spec))
        };
        if p.extension().is_none() {
            p = p.with_extension("nepl");
        }
        let p = canonicalize_path(&p);
        loader_log!(
            "[Loader] resolve_path: base={:?}, spec={:?} -> {:?}",
            base,
            spec,
            p
        );
        p
    }

    fn source_capabilities_for_module(
        &self,
        canon: &PathBuf,
        module: &Module,
    ) -> SourceCapabilities {
        if self.configured_stdlib_source_path(canon) {
            let mut capabilities = module_source_capabilities(module);
            capabilities
                .retain_use_sites(|site| self.source_capability_use_site_allowed(canon, site));
            capabilities
        } else {
            SourceCapabilities::none()
        }
    }

    fn source_capability_use_site_allowed(
        &self,
        canon: &PathBuf,
        site: &SourceCapabilityUseSite,
    ) -> bool {
        match site {
            SourceCapabilityUseSite::CompilerMemoryTypeDefinition { memory_type, .. } => {
                self.canonical_compiler_memory_type_definition_path(*memory_type) == *canon
            }
            _ => true,
        }
    }

    fn canonical_compiler_memory_type_definition_path(
        &self,
        memory_type: CompilerMemoryType,
    ) -> PathBuf {
        match memory_type {
            CompilerMemoryType::RawPointer | CompilerMemoryType::OwnerToken => {
                canonicalize_path(&self.stdlib_root.join("core").join("mem").join("types.nepl"))
            }
        }
    }

    fn configured_stdlib_source_path(&self, canon: &PathBuf) -> bool {
        let root = canonicalize_path(&self.stdlib_root);
        canon.starts_with(root)
    }
}

#[cfg(test)]
fn stdlib_path(root: &PathBuf, segments: &[&str]) -> PathBuf {
    segments
        .iter()
        .fold(root.clone(), |path, segment| path.join(segment))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_file_to_string(path: &PathBuf) -> Result<String, LoaderError> {
    fs::read_to_string(path).map_err(|e| LoaderError::Io(format!("{e}")))
}

#[cfg(target_arch = "wasm32")]
fn read_file_to_string(_path: &PathBuf) -> Result<String, LoaderError> {
    Err(LoaderError::Io(
        "filesystem access is not available on this target".into(),
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn canonicalize_path(path: &PathBuf) -> PathBuf {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
    normalize_path_lexically(&strip_windows_verbatim_prefix(canonical))
}

#[cfg(target_arch = "wasm32")]
fn canonicalize_path(path: &PathBuf) -> PathBuf {
    normalize_path_lexically(path)
}

/// Normalize path segments without filesystem access.
/// This keeps virtual paths stable across loader/provider boundaries.
fn normalize_path_lexically(path: &PathBuf) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(all(not(target_arch = "wasm32"), windows))]
fn strip_windows_verbatim_prefix(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{}", rest));
    }
    if let Some(rest) = text.strip_prefix(r"\\?\") {
        return PathBuf::from(rest);
    }
    path
}

#[cfg(all(not(target_arch = "wasm32"), not(windows)))]
fn strip_windows_verbatim_prefix(path: PathBuf) -> PathBuf {
    path
}

fn path_to_source_label(path: &PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

fn import_not_seen(imported_once: &mut BTreeSet<PathBuf>, target: &PathBuf) -> bool {
    imported_once.insert(canonicalize_path(target))
}

fn push_unique_canonical_path(
    paths: &mut Vec<PathBuf>,
    seen: &mut BTreeSet<PathBuf>,
    path: &PathBuf,
) {
    let canon = canonicalize_path(path);
    if seen.insert(canon.clone()) {
        paths.push(canon);
    }
}

/// Merge dependency arity hints while preserving the latest known declaration.
///
/// Import graphs can expose the same constructor through multiple facades. The
/// parser needs a single arity for boundary detection, and duplicate entries
/// with the same name should therefore refresh rather than accumulate.
fn push_loader_type_arity_hints(target: &mut Vec<(String, usize)>, source: Vec<(String, usize)>) {
    for (name, arity) in source {
        if let Some((_, existing_arity)) = target.iter_mut().rev().find(|(known, _)| known == &name)
        {
            *existing_arity = arity;
        } else {
            target.push((name, arity));
        }
    }
}

fn hash_type_arity_hints(type_arity_hints: &[(String, usize)]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for (name, arity) in type_arity_hints {
        fnv1a64_update(&mut hash, name.as_bytes());
        fnv1a64_update(&mut hash, &[0]);
        fnv1a64_update(&mut hash, arity.to_string().as_bytes());
        fnv1a64_update(&mut hash, &[0xff]);
    }
    hash
}

/// Hash the module surface that can affect downstream modules.
///
/// This is intentionally a stable frontend artifact, not a typed HIR cache. It
/// includes public declaration headers, logical import/prelude/include edges,
/// public re-export directives, and impl headers because those can affect name
/// lookup, kind boundaries, trait lookup, and later typed public-surface cache
/// invalidation. It excludes docs, spans, private function bodies, `SourceMap`,
/// `ImportResolution`, `TypeId`, Resource IR, and codegen fragments.
fn module_public_surface_hash(
    module: &Module,
    dependency_surface: Option<&CachedAritySurface>,
) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    fnv1a64_update(&mut hash, LOADER_SESSION_CACHE_VERSION.as_bytes());
    hash_str(&mut hash, "module-public-surface-v1");
    match dependency_surface {
        Some(surface) => {
            hash_u8(&mut hash, 1);
            hash_public_dependency_surface(&mut hash, surface);
        }
        None => hash_u8(&mut hash, 0),
    }
    for directive in &module.directives {
        hash_public_directive_surface(&mut hash, directive, dependency_surface.is_some());
    }
    for stmt in &module.root.items {
        hash_public_stmt_surface(&mut hash, stmt, module, dependency_surface.is_some());
    }
    hash
}

fn hash_public_dependency_surface(hash: &mut u64, surface: &CachedAritySurface) {
    hash_str(hash, "dependency-surface");
    hash_bool(hash, surface.no_prelude);
    hash_bool(hash, surface.implicit_default_prelude);
    hash_path(hash, &surface.default_prelude_path);
    let mut dependency_edges = surface.edges.iter().collect::<Vec<_>>();
    dependency_edges.sort_by_key(|edge| edge.source_order);
    hash_usize(hash, dependency_edges.len());
    for edge in dependency_edges {
        hash_source_import_edge(hash, edge);
    }
}

fn external_dependency_aggregate_public_surface_hash(path: &PathBuf) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    fnv1a64_update(&mut hash, LOADER_SESSION_CACHE_VERSION.as_bytes());
    hash_str(&mut hash, "external-dependency-public-surface-v1");
    hash_path(&mut hash, path);
    hash
}

fn cyclic_dependency_aggregate_public_surface_hash(path: &PathBuf, source: &str) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    fnv1a64_update(&mut hash, LOADER_SESSION_CACHE_VERSION.as_bytes());
    hash_str(&mut hash, "cyclic-dependency-public-surface-v1");
    hash_path(&mut hash, path);
    hash_u64(&mut hash, fnv1a64(source.as_bytes()));
    hash
}

fn hash_dependency_aggregate_public_surface_entries(hash: &mut u64, entries: &[(PathBuf, u64)]) {
    hash_usize(hash, entries.len());
    for (source_order, (path, dependency_hash)) in entries.iter().enumerate() {
        hash_usize(hash, source_order);
        hash_path(hash, path);
        hash_u64(hash, *dependency_hash);
    }
}

fn hash_public_directive_surface(
    hash: &mut u64,
    directive: &Directive,
    dependency_surface_hashed: bool,
) {
    match directive {
        Directive::Import {
            path, clause, vis, ..
        } if *vis == Visibility::Pub && !dependency_surface_hashed => {
            hash_str(hash, "directive.import.pub");
            hash_str(hash, path);
            hash_import_clause(hash, clause);
        }
        Directive::Include { path, .. } if !dependency_surface_hashed => {
            hash_str(hash, "directive.include.pub");
            hash_str(hash, path);
        }
        Directive::Extern {
            vis,
            module,
            name,
            func,
            signature,
            ..
        } if *vis == Visibility::Pub => {
            hash_str(hash, "directive.extern.pub");
            hash_str(hash, module);
            hash_str(hash, name);
            hash_str(hash, &func.name);
            hash_type_expr(hash, signature);
        }
        Directive::IfTarget { target, .. } => {
            hash_str(hash, "directive.if_target");
            hash_str(hash, target);
        }
        Directive::IfProfile { profile, .. } => {
            hash_str(hash, "directive.if_profile");
            hash_str(hash, profile);
        }
        Directive::NoPrelude { .. } => hash_str(hash, "directive.no_prelude"),
        Directive::Prelude { path, .. } if !dependency_surface_hashed => {
            hash_str(hash, "directive.prelude");
            hash_str(hash, path);
        }
        Directive::Entry { .. } | Directive::Target { .. } | Directive::IndentWidth { .. } => {}
        Directive::Import { .. }
        | Directive::Extern { .. }
        | Directive::Include { .. }
        | Directive::Prelude { .. } => {}
        Directive::Use { path, .. } => {
            hash_str(hash, "directive.use");
            hash_str(hash, path);
        }
    }
}

fn hash_public_stmt_surface(
    hash: &mut u64,
    stmt: &Stmt,
    module: &Module,
    dependency_surface_hashed: bool,
) {
    match stmt {
        Stmt::Directive(directive) => {
            hash_public_directive_surface(hash, directive, dependency_surface_hashed)
        }
        Stmt::FnDef(def) if def.vis == Visibility::Pub => hash_fn_def_signature(hash, def),
        Stmt::FnAlias(alias) if alias.vis == Visibility::Pub => hash_fn_alias(hash, alias, module),
        Stmt::StructDef(def) if def.vis == Visibility::Pub => hash_struct_def(hash, def),
        Stmt::EnumDef(def) if def.vis == Visibility::Pub => hash_enum_def(hash, def),
        Stmt::Trait(def) if def.vis == Visibility::Pub => hash_trait_def(hash, def),
        Stmt::Impl(def) => hash_impl_def(hash, def),
        Stmt::FnDef(_)
        | Stmt::FnAlias(_)
        | Stmt::StructDef(_)
        | Stmt::EnumDef(_)
        | Stmt::Trait(_)
        | Stmt::Expr(_)
        | Stmt::ExprSemi(_, _)
        | Stmt::Wasm(_)
        | Stmt::LlvmIr(_) => {}
    }
}

fn hash_fn_def_signature(hash: &mut u64, def: &FnDef) {
    hash_str(hash, "fn");
    hash_visibility(hash, def.vis);
    hash_str(hash, &def.name.name);
    hash_bool(hash, def.no_shadow);
    hash_type_params(hash, &def.type_params);
    hash_type_expr(hash, &def.signature);
    hash_usize(hash, def.params.len());
}

fn hash_fn_alias(hash: &mut u64, alias: &FnAlias, module: &Module) {
    hash_str(hash, "fn_alias");
    hash_visibility(hash, alias.vis);
    hash_str(hash, &alias.name.name);
    hash_bool(hash, alias.no_shadow);
    hash_str(hash, &alias.target.name);
    let matching_targets = module
        .root
        .items
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::FnDef(def) if def.name.name == alias.target.name => Some(def),
            _ => None,
        })
        .collect::<Vec<_>>();
    hash_usize(hash, matching_targets.len());
    for target in matching_targets {
        hash_fn_def_signature(hash, target);
    }
}

fn hash_struct_def(hash: &mut u64, def: &StructDef) {
    hash_str(hash, "struct");
    hash_str(hash, &def.name.name);
    hash_type_params(hash, &def.type_params);
    hash_usize(hash, def.fields.len());
    for (name, ty) in &def.fields {
        hash_str(hash, &name.name);
        hash_type_expr(hash, ty);
    }
}

fn hash_enum_def(hash: &mut u64, def: &EnumDef) {
    hash_str(hash, "enum");
    hash_str(hash, &def.name.name);
    hash_type_params(hash, &def.type_params);
    hash_usize(hash, def.variants.len());
    for variant in &def.variants {
        hash_str(hash, &variant.name.name);
        match &variant.payload {
            Some(payload) => {
                hash_u8(hash, 1);
                hash_type_expr(hash, payload);
            }
            None => hash_u8(hash, 0),
        }
    }
}

fn hash_trait_def(hash: &mut u64, def: &TraitDef) {
    hash_str(hash, "trait");
    hash_str(hash, &def.name.name);
    hash_type_params(hash, &def.type_params);
    hash_usize(hash, def.capabilities.len());
    for capability in &def.capabilities {
        hash_trait_capability(hash, capability);
    }
    hash_usize(hash, def.methods.len());
    for method in &def.methods {
        hash_fn_def_signature(hash, method);
    }
}

fn hash_impl_def(hash: &mut u64, def: &ImplDef) {
    hash_str(hash, "impl");
    hash_type_params(hash, &def.type_params);
    match &def.trait_ref {
        Some(trait_ref) => {
            hash_u8(hash, 1);
            hash_trait_ref(hash, trait_ref);
        }
        None => hash_u8(hash, 0),
    }
    hash_type_expr(hash, &def.target_ty);
    hash_usize(hash, def.methods.len());
    for method in &def.methods {
        hash_fn_def_signature(hash, method);
    }
}

fn hash_type_params(hash: &mut u64, params: &[TypeParam]) {
    hash_usize(hash, params.len());
    for param in params {
        hash_str(hash, &param.name.name);
        hash_usize(hash, param.bounds.len());
        for bound in &param.bounds {
            hash_trait_ref(hash, bound);
        }
    }
}

fn hash_trait_ref(hash: &mut u64, trait_ref: &TraitRef) {
    hash_str(hash, &trait_ref.name.name);
    hash_usize(hash, trait_ref.args.len());
    for arg in &trait_ref.args {
        hash_type_expr(hash, arg);
    }
}

fn hash_trait_capability(hash: &mut u64, capability: &TraitCapability) {
    match capability {
        TraitCapability::Copy => hash_str(hash, "copy"),
        TraitCapability::Clone => hash_str(hash, "clone"),
        TraitCapability::Drop => hash_str(hash, "drop"),
        TraitCapability::Unknown(name) => {
            hash_str(hash, "unknown");
            hash_str(hash, name);
        }
    }
}

fn hash_type_expr(hash: &mut u64, ty: &TypeExpr) {
    match ty.as_unspanned() {
        TypeExpr::Unit => hash_str(hash, "unit"),
        TypeExpr::I32 => hash_str(hash, "i32"),
        TypeExpr::U8 => hash_str(hash, "u8"),
        TypeExpr::F32 => hash_str(hash, "f32"),
        TypeExpr::Bool => hash_str(hash, "bool"),
        TypeExpr::Char => hash_str(hash, "char"),
        TypeExpr::Never => hash_str(hash, "never"),
        TypeExpr::Str => hash_str(hash, "str"),
        TypeExpr::Label(label) => {
            hash_str(hash, "label");
            match label {
                Some(label) => {
                    hash_u8(hash, 1);
                    hash_str(hash, label);
                }
                None => hash_u8(hash, 0),
            }
        }
        TypeExpr::Named(name) => {
            hash_str(hash, "named");
            hash_str(hash, name);
        }
        TypeExpr::Apply(base, args) => {
            hash_str(hash, "apply");
            hash_type_expr(hash, base);
            hash_usize(hash, args.len());
            for arg in args {
                hash_type_expr(hash, arg);
            }
        }
        TypeExpr::Boxed(inner) => {
            hash_str(hash, "boxed");
            hash_type_expr(hash, inner);
        }
        TypeExpr::Reference(inner, mutable) => {
            hash_str(hash, "ref");
            hash_bool(hash, *mutable);
            hash_type_expr(hash, inner);
        }
        TypeExpr::Tuple(items) => {
            hash_str(hash, "tuple");
            hash_usize(hash, items.len());
            for item in items {
                hash_type_expr(hash, item);
            }
        }
        TypeExpr::Function {
            params,
            result,
            effect,
        } => {
            hash_str(hash, "fn");
            hash_effect(hash, *effect);
            hash_usize(hash, params.len());
            for param in params {
                hash_type_expr(hash, param);
            }
            hash_type_expr(hash, result);
        }
        TypeExpr::Spanned(_, _) => unreachable!("as_unspanned removes span wrappers"),
    }
}

fn hash_effect(hash: &mut u64, effect: Effect) {
    match effect {
        Effect::Pure => hash_str(hash, "pure"),
        Effect::Impure => hash_str(hash, "impure"),
    }
}

fn hash_source_import_edge(hash: &mut u64, edge: &SourceImportEdge) {
    hash_u8(
        hash,
        match edge.kind {
            SourceImportEdgeKind::Prelude => 1,
            SourceImportEdgeKind::Import => 2,
            SourceImportEdgeKind::Include => 3,
        },
    );
    hash_path(hash, &edge.target_path);
    hash_visibility(hash, edge.visibility);
    match &edge.import_clause {
        Some(clause) => {
            hash_u8(hash, 1);
            hash_import_clause(hash, clause);
        }
        None => hash_u8(hash, 0),
    }
    hash_usize(hash, edge.source_order);
}

fn hash_import_clause(hash: &mut u64, clause: &ImportClause) {
    match clause {
        ImportClause::DefaultAlias => hash_u8(hash, 1),
        ImportClause::Alias(name) => {
            hash_u8(hash, 2);
            hash_str(hash, name);
        }
        ImportClause::Open => hash_u8(hash, 3),
        ImportClause::Selective(items) => {
            hash_u8(hash, 4);
            hash_usize(hash, items.len());
            for item in items {
                hash_str(hash, &item.name);
                match &item.alias {
                    Some(alias) => {
                        hash_u8(hash, 1);
                        hash_str(hash, alias);
                    }
                    None => hash_u8(hash, 0),
                }
                hash_bool(hash, item.glob);
            }
        }
        ImportClause::Merge => hash_u8(hash, 5),
    }
}

fn hash_visibility(hash: &mut u64, visibility: Visibility) {
    hash_u8(
        hash,
        match visibility {
            Visibility::Pub => 1,
            Visibility::Private => 2,
        },
    );
}

fn hash_path(hash: &mut u64, path: &PathBuf) {
    let canonical = canonicalize_path(path);
    hash_str(hash, &canonical.to_string_lossy());
}

fn hash_str(hash: &mut u64, value: &str) {
    fnv1a64_update(hash, value.as_bytes());
    fnv1a64_update(hash, &[0]);
}

fn hash_bool(hash: &mut u64, value: bool) {
    hash_u8(hash, if value { 1 } else { 0 });
}

fn hash_usize(hash: &mut u64, value: usize) {
    fnv1a64_update(hash, value.to_string().as_bytes());
    fnv1a64_update(hash, &[0]);
}

fn hash_u64(hash: &mut u64, value: u64) {
    fnv1a64_update(hash, &value.to_le_bytes());
    fnv1a64_update(hash, &[0]);
}

fn hash_u8(hash: &mut u64, value: u8) {
    fnv1a64_update(hash, &[value, 0xff]);
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    fnv1a64_update(&mut hash, bytes);
    hash
}

fn fnv1a64_update(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

#[cfg(test)]
mod tests {
    use super::*;

    use crate::effects::{PrivateCacheOp, RawBodyMemoryOp, RawMemoryOp, WasmRawBodyMemoryOp};
    use crate::resource_primitives::{
        CollectionSlotBorrowPrimitive, CollectionSlotLifecyclePrimitive,
    };
    use crate::source_map::{CompilerMemoryField, CompilerMemoryType, SourceCapabilityUseSite};
    use crate::span::Span;
    use alloc::vec;

    trait SourceCapabilitiesTestExt {
        fn allows_raw_memory_structural_boundary(&self) -> bool;
        fn allows_raw_address_view_boundary(&self) -> bool;
        fn allows_raw_address_alias_boundary(&self) -> bool;
        fn allows_owner_token_construct_boundary(&self) -> bool;
        fn allows_raw_memory_operation_boundary(&self, operation: RawMemoryOp) -> bool;
        fn allows_raw_body_memory_operation_boundary(&self, operation: RawBodyMemoryOp) -> bool;
        fn allows_owner_aggregate_constructor_boundary(&self, name: &str) -> bool;
        fn allows_owner_aggregate_field_boundary(&self) -> bool;
        fn allows_compiler_memory_field_boundary(&self, field: CompilerMemoryField) -> bool;
        fn allows_compiler_memory_type_definition(&self, memory_type: CompilerMemoryType) -> bool;
        fn allows_collection_slot_lifecycle_boundary(
            &self,
            primitive: CollectionSlotLifecyclePrimitive,
        ) -> bool;
        fn allows_collection_slot_borrow_boundary(
            &self,
            primitive: CollectionSlotBorrowPrimitive,
        ) -> bool;
        fn allows_private_cache_boundary(&self, operation: PrivateCacheOp) -> bool;
    }

    impl SourceCapabilitiesTestExt for SourceCapabilities {
        fn allows_raw_memory_structural_boundary(&self) -> bool {
            self.use_sites_for_tests().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::RawMemoryStructuralBoundary { .. }
                )
            })
        }

        fn allows_raw_address_view_boundary(&self) -> bool {
            self.use_sites_for_tests()
                .any(|site| matches!(site, SourceCapabilityUseSite::RawAddressViewBoundary { .. }))
        }

        fn allows_raw_address_alias_boundary(&self) -> bool {
            self.use_sites_for_tests().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::RawAddressAliasBoundary { .. }
                )
            })
        }

        fn allows_owner_token_construct_boundary(&self) -> bool {
            self.use_sites_for_tests().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::OwnerTokenConstructBoundary { .. }
                )
            })
        }

        fn allows_raw_memory_operation_boundary(&self, operation: RawMemoryOp) -> bool {
            self.use_sites_for_tests().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::RawMemoryOperationBoundary {
                        operation: site_op,
                        ..
                    } if *site_op == operation
                )
            })
        }

        fn allows_raw_body_memory_operation_boundary(&self, operation: RawBodyMemoryOp) -> bool {
            self.use_sites_for_tests().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::RawBodyMemoryOperationBoundary {
                        operation: site_op,
                        ..
                    } if *site_op == operation
                )
            })
        }

        fn allows_owner_aggregate_constructor_boundary(&self, name: &str) -> bool {
            self.use_sites_for_tests().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::OwnerAggregateConstructorBoundary {
                        name: site_name,
                        ..
                    } if site_name == name
                )
            })
        }

        fn allows_owner_aggregate_field_boundary(&self) -> bool {
            self.use_sites_for_tests().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::OwnerAggregateFieldBoundary { .. }
                )
            })
        }

        fn allows_compiler_memory_field_boundary(&self, field: CompilerMemoryField) -> bool {
            self.use_sites_for_tests().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::CompilerMemoryFieldBoundary {
                        field: site_field,
                        ..
                    } if *site_field == field
                )
            })
        }

        fn allows_compiler_memory_type_definition(&self, memory_type: CompilerMemoryType) -> bool {
            self.use_sites_for_tests().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::CompilerMemoryTypeDefinition {
                        memory_type: site_type,
                        ..
                    } if *site_type == memory_type
                )
            })
        }

        fn allows_collection_slot_lifecycle_boundary(
            &self,
            primitive: CollectionSlotLifecyclePrimitive,
        ) -> bool {
            self.use_sites_for_tests().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::CollectionSlotLifecycleBoundary {
                        primitive: site_primitive,
                        ..
                    } if *site_primitive == primitive
                )
            })
        }

        fn allows_collection_slot_borrow_boundary(
            &self,
            primitive: CollectionSlotBorrowPrimitive,
        ) -> bool {
            self.use_sites_for_tests().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::CollectionSlotBorrowBoundary {
                        primitive: site_primitive,
                        ..
                    } if *site_primitive == primitive
                )
            })
        }

        fn allows_private_cache_boundary(&self, operation: PrivateCacheOp) -> bool {
            self.use_sites_for_tests().any(|site| {
                matches!(
                    site,
                    SourceCapabilityUseSite::PrivateCacheBoundary {
                        operation: site_operation,
                        ..
                    } if *site_operation == operation
                )
            })
        }
    }

    fn test_loader() -> Loader {
        Loader::new(PathBuf::from("C:/nepl-test/stdlib"))
    }

    fn real_test_loader() -> Loader {
        Loader::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("stdlib"),
        )
    }

    fn path_from_segments(root: &str, segments: &[&str]) -> PathBuf {
        segments
            .iter()
            .fold(PathBuf::from(root), |path, segment| path.join(segment))
    }

    fn load_source_capabilities(loader: &Loader, path: PathBuf, src: &str) -> SourceCapabilities {
        let mut sm = SourceMap::new();
        let mut cache = BTreeMap::new();
        let mut processing = BTreeSet::new();
        let mut imported_once = BTreeSet::new();
        let mut shallow_type_arity_cache = BTreeMap::new();
        let _ = loader
            .load_from_contents(
                path,
                String::from(src),
                &mut sm,
                &mut cache,
                &mut processing,
                &mut imported_once,
                &mut shallow_type_arity_cache,
                false,
            )
            .expect("test source should parse");
        sm.capabilities(FileId(0))
    }

    #[test]
    fn imported_once_uses_canonical_paths() {
        let mut imported_once = BTreeSet::new();
        let direct = PathBuf::from("C:/nepl-test/stdlib/alloc/io/bytebuf.nepl");
        let via_parent = PathBuf::from("C:/nepl-test/nepl-core/../stdlib/alloc/io/bytebuf.nepl");

        assert!(import_not_seen(&mut imported_once, &via_parent));
        assert!(
            !import_not_seen(&mut imported_once, &direct),
            "same import reached through a lexical parent path must not be loaded twice"
        );
    }

    #[test]
    fn resolve_path_returns_canonical_loader_key() {
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let loader = Loader::new(stdlib_root.clone());
        let base = path_from_segments(
            "C:/nepl-test/stdlib/alloc/collections/vec",
            &["storage", "view.nepl"],
        );

        assert_eq!(
            loader.resolve_path(&base, "../types"),
            canonicalize_path(&stdlib_path(
                &stdlib_root,
                &["alloc", "collections", "vec", "types.nepl"],
            )),
            "relative imports must enter loader surfaces as canonical cache keys",
        );
        assert_eq!(
            loader.resolve_path(&base, "core/result"),
            canonicalize_path(&stdlib_path(&stdlib_root, &["core", "result.nepl"])),
            "stdlib imports must use the same canonical root as cache and import-once checks",
        );
    }

    #[test]
    fn arity_preload_does_not_cache_merged_diamond_import_modules() {
        let entry_path = canonicalize_path(&PathBuf::from("C:/nepl-test/user/main.nepl"));
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let a_path = canonicalize_path(&stdlib_path(&stdlib_root, &["a.nepl"]));
        let b_path = canonicalize_path(&stdlib_path(&stdlib_root, &["b.nepl"]));
        let c_path = canonicalize_path(&stdlib_path(&stdlib_root, &["c.nepl"]));
        let entry_source = String::from(
            "#no_prelude\n#import \"a\" as *\n#import \"b\" as *\nfn main %fn unit i32 \\unit:\n    0\n",
        );
        let mut sources = BTreeMap::new();
        sources.insert(
            a_path,
            String::from("#import \"c\" as *\npub fn a_value %fn unit i32 \\unit:\n    1\n"),
        );
        sources.insert(
            b_path,
            String::from("#import \"c\" as *\npub fn b_value %fn unit i32 \\unit:\n    2\n"),
        );
        sources.insert(
            c_path,
            String::from("pub struct Shared<.T>:\n    value %.T\n"),
        );

        let mut loader = Loader::new(stdlib_root);
        let mut provider = |path: &PathBuf| {
            sources
                .get(path)
                .cloned()
                .ok_or_else(|| LoaderError::Io(format!("missing test source: {:?}", path)))
        };
        let loaded = loader
            .load_inline_with_provider(entry_path, entry_source, &mut provider)
            .expect("diamond import graph should load through shallow arity preload");
        let shared_count = loaded
            .module
            .root
            .items
            .iter()
            .filter(|stmt| matches!(stmt, Stmt::StructDef(def) if def.name.name == "Shared"))
            .count();

        assert_eq!(
            shared_count, 1,
            "type-arity preload must not cache context-dependent merged modules before the real import-once pass",
        );
    }

    #[test]
    fn provider_session_cache_reuses_stdlib_parsed_modules_with_fresh_file_ids() {
        let entry_path = canonicalize_path(&PathBuf::from("C:/nepl-test/user/main.nepl"));
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let foo_path = canonicalize_path(&stdlib_path(&stdlib_root, &["foo.nepl"]));
        let entry_source = String::from(
            "#no_prelude\n#import \"foo\" as *\nfn main %fn unit i32 \\unit:\n    foo unit\n",
        );
        let foo_source = String::from("pub fn foo %fn unit i32 \\unit:\n    1\n");
        let mut sources = BTreeMap::new();
        sources.insert(foo_path, foo_source);
        let mut session_cache = LoaderSessionCache::new("test-stdlib");

        for _ in 0..2 {
            let mut loader = Loader::new(stdlib_root.clone());
            let mut provider = |path: &PathBuf| {
                sources
                    .get(path)
                    .cloned()
                    .ok_or_else(|| LoaderError::Io(format!("missing test source: {:?}", path)))
            };
            let loaded = loader
                .load_inline_with_provider_and_cache(
                    entry_path.clone(),
                    entry_source.clone(),
                    &mut provider,
                    &mut session_cache,
                )
                .expect("provider-backed load should parse");
            let foo_file_id = loaded
                .source_map
                .iter_paths()
                .find_map(|(file_id, path)| {
                    (path.to_string_lossy().ends_with("/foo.nepl")
                        || path.to_string_lossy().ends_with("\\foo.nepl"))
                    .then_some(file_id)
                })
                .expect("imported stdlib file should be present in the fresh source map");
            let foo_def = loaded
                .module
                .root
                .items
                .iter()
                .find_map(|stmt| match stmt {
                    Stmt::FnDef(def) if def.name.name == "foo" => Some(def),
                    _ => None,
                })
                .expect("imported stdlib function should be merged into the loaded module");
            assert_eq!(
                foo_def.name.span.file_id, foo_file_id,
                "cached stdlib AST spans must be projected to the current SourceMap file id",
            );
        }

        let stats = session_cache.stats();
        assert!(
            stats.parsed_module_stores >= 1,
            "the first stdlib load should populate the parsed-module cache",
        );
        assert!(
            stats.parsed_module_hits >= 1,
            "the second stdlib load should reuse the parsed-module cache",
        );
    }

    #[test]
    fn provider_session_cache_misses_when_stdlib_source_hash_changes() {
        let entry_path = canonicalize_path(&PathBuf::from("C:/nepl-test/user/main.nepl"));
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let foo_path = canonicalize_path(&stdlib_path(&stdlib_root, &["foo.nepl"]));
        let entry_source = String::from(
            "#no_prelude\n#import \"foo\" as *\nfn main %fn unit i32 \\unit:\n    foo unit\n",
        );
        let mut sources = BTreeMap::new();
        sources.insert(
            foo_path.clone(),
            String::from("pub fn foo %fn unit i32 \\unit:\n    1\n"),
        );
        let mut session_cache = LoaderSessionCache::new("test-stdlib");

        for body_value in ["1", "2"] {
            sources.insert(
                foo_path.clone(),
                format!("pub fn foo %fn unit i32 \\unit:\n    {body_value}\n"),
            );
            let mut loader = Loader::new(stdlib_root.clone());
            let mut provider = |path: &PathBuf| {
                sources
                    .get(path)
                    .cloned()
                    .ok_or_else(|| LoaderError::Io(format!("missing test source: {:?}", path)))
            };
            loader
                .load_inline_with_provider_and_cache(
                    entry_path.clone(),
                    entry_source.clone(),
                    &mut provider,
                    &mut session_cache,
                )
                .expect("provider-backed load should parse");
        }

        let stats = session_cache.stats();
        assert_eq!(
            stats.parsed_module_hits, 0,
            "changing stdlib source text for the same canonical path must not hit the old parsed module",
        );
        assert_eq!(
            stats.parsed_module_misses, 2,
            "each distinct stdlib source hash should create a separate parsed-module key",
        );
        assert_eq!(
            stats.parsed_module_stores, 2,
            "both source versions should be stored independently",
        );
    }

    #[test]
    fn provider_session_cache_misses_when_imported_type_arity_hints_change() {
        let entry_path = canonicalize_path(&PathBuf::from("C:/nepl-test/user/main.nepl"));
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let foo_path = canonicalize_path(&stdlib_path(&stdlib_root, &["foo.nepl"]));
        let defs_path = canonicalize_path(&stdlib_path(&stdlib_root, &["defs.nepl"]));
        let entry_source = String::from(
            "#no_prelude\n#import \"foo\" as *\nfn main %fn unit i32 \\unit:\n    foo unit\n",
        );
        let foo_source =
            String::from("#import \"defs\" as *\npub fn foo %fn unit i32 \\unit:\n    1\n");
        let mut sources = BTreeMap::new();
        sources.insert(foo_path.clone(), foo_source);
        sources.insert(defs_path.clone(), String::new());
        let mut session_cache = LoaderSessionCache::new("test-stdlib");

        for defs_source in [
            "pub struct Box<.T>:\n    value %.T\n",
            "pub struct Box<.T,.U>:\n    first %.T\n    second %.U\n",
        ] {
            sources.insert(defs_path.clone(), String::from(defs_source));
            let mut loader = Loader::new(stdlib_root.clone());
            let mut provider = |path: &PathBuf| {
                sources
                    .get(path)
                    .cloned()
                    .ok_or_else(|| LoaderError::Io(format!("missing test source: {:?}", path)))
            };
            loader
                .load_inline_with_provider_and_cache(
                    entry_path.clone(),
                    entry_source.clone(),
                    &mut provider,
                    &mut session_cache,
                )
                .expect("provider-backed load should parse");
        }

        let stats = session_cache.stats();
        assert_eq!(
            stats.parsed_module_hits, 0,
            "changing imported type arity metadata must not reuse a parsed module keyed with old parser boundary hints",
        );
        assert_eq!(
            stats.parsed_module_misses, 4,
            "both defs.nepl and foo.nepl should miss again when the imported public type arity changes",
        );
        assert_eq!(
            stats.parsed_module_stores, 4,
            "cache entries should be separated by imported type arity hint hash as well as source hash",
        );
    }

    #[test]
    fn provider_session_cache_reuses_source_arity_surfaces() {
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let foo_path = canonicalize_path(&stdlib_path(&stdlib_root, &["foo.nepl"]));
        let loader = Loader::new(stdlib_root.clone());
        let foo_source = "#import \"defs\" as *\n";
        let mut session_cache = LoaderSessionCache::new("test-stdlib");

        for _ in 0..2 {
            let paths = loader.type_arity_preload_paths_with_cache(
                &foo_path,
                FileId(0),
                foo_source,
                false,
                Some(&mut session_cache),
            );
            assert_eq!(
                paths,
                vec![canonicalize_path(&stdlib_path(
                    &stdlib_root,
                    &["defs.nepl"]
                ))],
                "arity preload must keep using the same directive semantics on cache hit",
            );
        }

        let stats = session_cache.stats();
        assert_eq!(
            stats.arity_surface_stores, 1,
            "the first source scan should store one arity surface artifact",
        );
        assert_eq!(
            stats.arity_surface_hits, 1,
            "the second source scan should reuse the path/source-hash keyed arity surface",
        );
    }

    #[test]
    fn provider_session_cache_does_not_store_user_source_arity_surfaces() {
        let entry_path = canonicalize_path(&PathBuf::from("C:/nepl-test/user/main.nepl"));
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let loader = Loader::new(stdlib_root.clone());
        let entry_source = "#no_prelude\n#import \"foo\" as *\n";
        let mut session_cache = LoaderSessionCache::new("test-stdlib");

        for _ in 0..2 {
            let paths = loader.type_arity_preload_paths_with_cache(
                &entry_path,
                FileId(0),
                entry_source,
                true,
                Some(&mut session_cache),
            );
            assert_eq!(
                paths,
                vec![canonicalize_path(&stdlib_path(&stdlib_root, &["foo.nepl"]))],
                "user source scans should still resolve stdlib import roots for prewarm",
            );
        }

        let stats = session_cache.stats();
        assert_eq!(
            stats.arity_surface_stores, 0,
            "long-lived LoaderSessionCache must not retain user-source arity surfaces",
        );
        assert_eq!(
            stats.arity_surface_hits, 0,
            "user source arity scans should be recomputed instead of becoming session hits",
        );
        assert_eq!(
            stats.arity_surface_bypasses, 2,
            "bypass stats should make the stdlib-only cache boundary observable",
        );
    }

    #[test]
    fn source_import_surface_preserves_clause_visibility_and_order() {
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let loader = Loader::new(stdlib_root.clone());
        let path = canonicalize_path(&stdlib_path(&stdlib_root, &["facade.nepl"]));
        let source = [
            "#prelude std/prelude_base",
            "#import pub \"types\" as { Box as PublicBox, Result::* }",
            "#include \"included\"",
            "",
        ]
        .join("\n");

        let surface = loader.compute_source_arity_surface(&path, FileId(0), &source);
        assert_eq!(
            surface.preload_paths(false),
            vec![
                canonicalize_path(&stdlib_path(&stdlib_root, &["std", "prelude_base.nepl"],)),
                canonicalize_path(&stdlib_path(&stdlib_root, &["types.nepl"])),
                canonicalize_path(&stdlib_path(&stdlib_root, &["included.nepl"])),
            ],
            "preload paths should be derived from source-order import edges",
        );
        assert_eq!(
            surface.public_reexport_paths(),
            vec![
                canonicalize_path(&stdlib_path(&stdlib_root, &["types.nepl"])),
                canonicalize_path(&stdlib_path(&stdlib_root, &["included.nepl"])),
            ],
            "public re-export recovery should use the same import surface edges",
        );

        let import_edge = surface
            .edges
            .iter()
            .find(|edge| edge.kind == SourceImportEdgeKind::Import)
            .expect("test source should contain one import edge");
        assert_eq!(import_edge.visibility, Visibility::Pub);
        assert_eq!(import_edge.source_order, 1);
        assert!(
            matches!(import_edge.import_clause, Some(ImportClause::Selective(_))),
            "logical import graph groundwork must preserve import clauses instead of reducing them to path-only edges",
        );
    }

    #[test]
    fn module_public_surface_hash_ignores_body_and_private_edits() {
        let loader = Loader::new(PathBuf::from("C:/nepl-test/stdlib"));
        let first = loader
            .parse_module_with_type_arity_hints(
                FileId(0),
                "fn helper %fn unit i32 \\unit:\n    1\npub fn api %fn unit i32 \\unit:\n    helper unit\n"
                    .to_string(),
                Vec::new(),
            )
            .expect("first module should parse");
        let body_edit = loader
            .parse_module_with_type_arity_hints(
                FileId(0),
                "fn helper %fn unit u8 \\unit:\n    2\npub fn api %fn unit i32 \\unit:\n    3\n"
                    .to_string(),
                Vec::new(),
            )
            .expect("body edit module should parse");

        assert_eq!(
            module_public_surface_hash(&first, None),
            module_public_surface_hash(&body_edit, None),
            "private helper signatures and public function bodies are not downstream public surface",
        );
    }

    #[test]
    fn module_public_surface_hash_tracks_public_signature_and_reexports() {
        let loader = Loader::new(PathBuf::from("C:/nepl-test/stdlib"));
        let first = loader
            .parse_module_with_type_arity_hints(
                FileId(0),
                "#import pub \"types\" as { Box as PublicBox }\npub fn api %fn unit i32 \\unit:\n    1\n"
                    .to_string(),
                Vec::new(),
            )
            .expect("first module should parse");
        let signature_edit = loader
            .parse_module_with_type_arity_hints(
                FileId(0),
                "#import pub \"types\" as { Box as PublicBox }\npub fn api %fn unit u8 \\unit:\n    1\n"
                    .to_string(),
                Vec::new(),
            )
            .expect("signature edit module should parse");
        let reexport_edit = loader
            .parse_module_with_type_arity_hints(
                FileId(0),
                "#import pub \"types\" as { Box as RenamedBox }\npub fn api %fn unit i32 \\unit:\n    1\n"
                    .to_string(),
                Vec::new(),
            )
            .expect("reexport edit module should parse");

        let first_hash = module_public_surface_hash(&first, None);
        assert_ne!(
            first_hash,
            module_public_surface_hash(&signature_edit, None),
            "public function signature changes must invalidate the public surface",
        );
        assert_ne!(
            first_hash,
            module_public_surface_hash(&reexport_edit, None),
            "public re-export clause changes must invalidate the public surface",
        );
    }

    #[test]
    fn module_public_surface_hash_tracks_lookup_context_and_alias_targets() {
        let loader = Loader::new(PathBuf::from("C:/nepl-test/stdlib"));
        let base = PathBuf::from("C:/nepl-test/stdlib/pkg/main.nepl");
        let first_src =
            "#import \"a\" as *\nfn helper %fn unit i32 \\unit:\n    1\npub fn api helper;\n";
        let body_edit_src =
            "#import \"a\" as *\nfn helper %fn unit i32 \\unit:\n    2\npub fn api helper;\n";
        let target_signature_edit_src =
            "#import \"a\" as *\nfn helper %fn unit u8 \\unit:\n    1\npub fn api helper;\n";
        let private_import_edit_src =
            "#import \"b\" as *\nfn helper %fn unit i32 \\unit:\n    1\npub fn api helper;\n";

        let first = loader
            .parse_module_with_type_arity_hints(FileId(0), first_src.to_string(), Vec::new())
            .expect("first module should parse");
        let body_edit = loader
            .parse_module_with_type_arity_hints(FileId(0), body_edit_src.to_string(), Vec::new())
            .expect("body edit module should parse");
        let target_signature_edit = loader
            .parse_module_with_type_arity_hints(
                FileId(0),
                target_signature_edit_src.to_string(),
                Vec::new(),
            )
            .expect("target signature edit module should parse");
        let private_import_edit = loader
            .parse_module_with_type_arity_hints(
                FileId(0),
                private_import_edit_src.to_string(),
                Vec::new(),
            )
            .expect("private import edit module should parse");

        let first_surface = loader.compute_source_arity_surface(&base, FileId(0), first_src);
        let body_edit_surface =
            loader.compute_source_arity_surface(&base, FileId(0), body_edit_src);
        let target_signature_edit_surface =
            loader.compute_source_arity_surface(&base, FileId(0), target_signature_edit_src);
        let private_import_edit_surface =
            loader.compute_source_arity_surface(&base, FileId(0), private_import_edit_src);

        let first_hash = module_public_surface_hash(&first, Some(&first_surface));
        assert_eq!(
            first_hash,
            module_public_surface_hash(&body_edit, Some(&body_edit_surface)),
            "public alias targets should ignore private target body edits",
        );
        assert_ne!(
            first_hash,
            module_public_surface_hash(
                &target_signature_edit,
                Some(&target_signature_edit_surface)
            ),
            "public alias targets must include local callable signatures exposed through the alias",
        );
        assert_ne!(
            first_hash,
            module_public_surface_hash(&private_import_edit, Some(&private_import_edit_surface)),
            "private import edges can affect public signature name resolution and must invalidate the surface",
        );
    }

    #[test]
    fn module_public_surface_hash_tracks_public_no_shadow_contract() {
        let loader = Loader::new(PathBuf::from("C:/nepl-test/stdlib"));
        let ordinary = loader
            .parse_module_with_type_arity_hints(
                FileId(0),
                "pub fn api %fn unit i32 \\unit:\n    1\n".to_string(),
                Vec::new(),
            )
            .expect("ordinary module should parse");
        let no_shadow = loader
            .parse_module_with_type_arity_hints(
                FileId(0),
                "pub fn noshadow api %fn unit i32 \\unit:\n    1\n".to_string(),
                Vec::new(),
            )
            .expect("noshadow module should parse");

        assert_ne!(
            module_public_surface_hash(&ordinary, None),
            module_public_surface_hash(&no_shadow, None),
            "noshadow participates in cross-file binding behavior and is part of the public contract",
        );
    }

    #[test]
    fn root_dependency_aggregate_public_surface_hash_tracks_reexported_stdlib_changes() {
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let loader = Loader::new(stdlib_root.clone());
        let entry_path = PathBuf::from("C:/nepl-test/user/main.nepl");
        let facade_path = canonicalize_path(&stdlib_path(&stdlib_root, &["facade.nepl"]));
        let types_path = canonicalize_path(&stdlib_path(&stdlib_root, &["types.nepl"]));
        let root_source =
            "#no_prelude\n#import \"facade\" as *\nfn main %fn unit i32 \\unit:\n    exported unit\n";
        let facade_source = "#import pub \"types\" as *\n";

        let mut cache = LoaderSessionCache::new("dependency-public-surface-test");
        let mut sources = BTreeMap::new();
        sources.insert(facade_path.clone(), facade_source.to_string());
        sources.insert(
            types_path.clone(),
            "pub fn exported %fn unit i32 \\unit:\n    1\n".to_string(),
        );
        let mut provider = |path: &PathBuf| {
            sources
                .get(&canonicalize_path(path))
                .cloned()
                .ok_or_else(|| LoaderError::Io(format!("missing test source: {}", path.display())))
        };
        let first_hash = loader
            .root_dependency_aggregate_public_surface_hash_for_source_with_cache(
                entry_path.clone(),
                root_source,
                &mut provider,
                &mut cache,
            )
            .expect("first dependency surface should hash");
        let after_first = cache.stats();
        assert!(
            after_first.dependency_aggregate_public_surface_hash_stores >= 2,
            "facade and re-exported dependency should store aggregate public-surface hashes",
        );

        sources.insert(
            types_path.clone(),
            "pub fn exported %fn unit i32 \\unit:\n    2\n".to_string(),
        );
        let mut provider = |path: &PathBuf| {
            sources
                .get(&canonicalize_path(path))
                .cloned()
                .ok_or_else(|| LoaderError::Io(format!("missing test source: {}", path.display())))
        };
        let body_edit_hash = loader
            .root_dependency_aggregate_public_surface_hash_for_source_with_cache(
                entry_path.clone(),
                root_source,
                &mut provider,
                &mut cache,
            )
            .expect("body edit dependency surface should hash");
        let after_body_edit = cache.stats();
        assert_eq!(
            first_hash, body_edit_hash,
            "dependency aggregate hash should ignore re-exported function body-only edits",
        );
        assert!(
            after_body_edit.dependency_aggregate_public_surface_hash_hits
                > after_first.dependency_aggregate_public_surface_hash_hits,
            "body-only edits should reuse aggregate public-surface entries keyed by the stable public surface",
        );

        sources.insert(
            types_path,
            "pub fn exported %fn unit u8 \\unit:\n    1\n".to_string(),
        );
        let mut provider = |path: &PathBuf| {
            sources
                .get(&canonicalize_path(path))
                .cloned()
                .ok_or_else(|| LoaderError::Io(format!("missing test source: {}", path.display())))
        };
        let signature_edit_hash = loader
            .root_dependency_aggregate_public_surface_hash_for_source_with_cache(
                entry_path,
                root_source,
                &mut provider,
                &mut cache,
            )
            .expect("signature edit dependency surface should hash");
        assert_ne!(
            first_hash, signature_edit_hash,
            "dependency aggregate hash must change when a re-exported public signature changes",
        );
    }

    #[test]
    fn root_dependency_aggregate_public_surface_hash_bypasses_non_stdlib_edges() {
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let loader = Loader::new(stdlib_root);
        let entry_path = PathBuf::from("C:/nepl-test/user/main.nepl");
        let root_source =
            "#no_prelude\n#import \"./helper\" as *\nfn main %fn unit i32 \\unit:\n    1\n";
        let mut cache = LoaderSessionCache::new("dependency-public-surface-test");
        let mut provider = |_path: &PathBuf| {
            Err(LoaderError::Io(
                "non-stdlib dependency should not be read by bundled stdlib aggregate hash".into(),
            ))
        };

        let _hash = loader
            .root_dependency_aggregate_public_surface_hash_for_source_with_cache(
                entry_path,
                root_source,
                &mut provider,
                &mut cache,
            )
            .expect("non-stdlib dependency edges should become conservative bypass hashes");
        assert!(
            cache
                .stats()
                .dependency_aggregate_public_surface_hash_bypasses
                >= 1,
            "user dependency edges are outside the bundled stdlib aggregate cache boundary",
        );
    }

    #[test]
    fn root_prewarm_surface_hash_ignores_body_edits_but_tracks_import_surface() {
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let loader = Loader::new(stdlib_root.clone());
        let entry_path = canonicalize_path(&PathBuf::from("C:/nepl-test/user/main.nepl"));
        let first_source =
            "#no_prelude\n#import \"foo\" as *\nfn main %fn unit i32 \\unit:\n    1\n";
        let body_edit_source =
            "#no_prelude\n#import \"foo\" as *\nfn main %fn unit i32 \\unit:\n    2\n";
        let import_edit_source =
            "#no_prelude\n#import \"bar\" as *\nfn main %fn unit i32 \\unit:\n    2\n";
        let clause_edit_source =
            "#no_prelude\n#import \"foo\" as { value as renamed }\nfn main %fn unit i32 \\unit:\n    2\n";

        let (first_hash, first_roots) =
            loader.root_prewarm_surface_for_source(entry_path.clone(), first_source);
        let (body_hash, body_roots) =
            loader.root_prewarm_surface_for_source(entry_path.clone(), body_edit_source);
        let (import_hash, import_roots) =
            loader.root_prewarm_surface_for_source(entry_path.clone(), import_edit_source);
        let (clause_hash, clause_roots) =
            loader.root_prewarm_surface_for_source(entry_path, clause_edit_source);

        assert_eq!(
            body_hash, first_hash,
            "body-only edits should reuse the same loader prewarm surface",
        );
        assert_eq!(
            body_roots, first_roots,
            "body-only edits must not change prewarm roots",
        );
        assert_ne!(
            import_hash, first_hash,
            "changing the imported module path must invalidate the prewarm surface",
        );
        assert_ne!(
            import_roots, first_roots,
            "changed import paths should produce different prewarm roots",
        );
        assert_ne!(
            clause_hash, first_hash,
            "the hash keeps import-clause changes visible for the future logical import graph",
        );
        assert_eq!(
            clause_roots, first_roots,
            "alias-only changes still warm the same loader roots even though the graph surface hash changes",
        );
    }

    #[test]
    fn root_prewarm_surface_hash_tracks_relative_import_resolution_and_lexer_errors() {
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let loader = Loader::new(stdlib_root);
        let left_entry = canonicalize_path(&PathBuf::from("C:/nepl-test/user/left/main.nepl"));
        let right_entry = canonicalize_path(&PathBuf::from("C:/nepl-test/user/right/main.nepl"));
        let relative_source = "#no_prelude\n#import \"./shared\" as *\n";
        let invalid_source = "fn main %fn unit i32 \\unit:\n    \"unterminated\n";
        let valid_no_prelude_source = "#no_prelude\n";

        let (left_hash, left_roots) =
            loader.root_prewarm_surface_for_source(left_entry, relative_source);
        let (right_hash, right_roots) =
            loader.root_prewarm_surface_for_source(right_entry, relative_source);
        let (invalid_hash, invalid_roots) = loader.root_prewarm_surface_for_source(
            PathBuf::from("C:/nepl-test/user/main.nepl"),
            invalid_source,
        );
        let (valid_no_prelude_hash, valid_no_prelude_roots) = loader
            .root_prewarm_surface_for_source(
                PathBuf::from("C:/nepl-test/user/main.nepl"),
                valid_no_prelude_source,
            );

        assert_ne!(
            left_hash, right_hash,
            "relative imports must hash the resolved target path, not only the literal spec",
        );
        assert_ne!(
            left_roots, right_roots,
            "entry paths in different directories should resolve relative imports to different roots",
        );
        assert_ne!(
            invalid_hash, valid_no_prelude_hash,
            "lexer-error surfaces must not collide with valid no-prelude surfaces",
        );
        assert!(invalid_roots.is_empty());
        assert!(valid_no_prelude_roots.is_empty());
    }

    #[test]
    fn source_arity_surface_keeps_root_default_prelude_out_of_non_root_loads() {
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let entry_path = canonicalize_path(&stdlib_path(&stdlib_root, &["root_like.nepl"]));
        let loader = Loader::new(stdlib_root.clone());
        let mut session_cache = LoaderSessionCache::new("test-stdlib");

        let root_paths = loader.type_arity_preload_paths_with_cache(
            &entry_path,
            FileId(0),
            "",
            true,
            Some(&mut session_cache),
        );
        let non_root_paths = loader.type_arity_preload_paths_with_cache(
            &entry_path,
            FileId(0),
            "",
            false,
            Some(&mut session_cache),
        );

        assert_eq!(
            root_paths,
            vec![canonicalize_path(&stdlib_path(
                &stdlib_root,
                &["std", "prelude_base.nepl"],
            ))],
            "root sources without explicit prelude should preload the default prelude",
        );
        assert!(
            non_root_paths.is_empty(),
            "the same cached surface must not inject the root-only default prelude into non-root modules",
        );
        assert_eq!(
            session_cache.stats().arity_surface_hits,
            1,
            "root and non-root path lists should be derived from one cached surface, not from stale path output",
        );
    }

    #[test]
    fn source_arity_surface_skips_default_prelude_when_lexer_errors() {
        let entry_path = canonicalize_path(&stdlib_path(
            &PathBuf::from("C:/nepl-test/stdlib"),
            &["broken.nepl"],
        ));
        let loader = test_loader();
        let mut session_cache = LoaderSessionCache::new("test-stdlib");

        let paths = loader.type_arity_preload_paths_with_cache(
            &entry_path,
            FileId(0),
            "#target wasm\nlet broken \"unterminated\n",
            true,
            Some(&mut session_cache),
        );

        assert!(
            paths.is_empty(),
            "lexer errors must not trigger default prelude preloading before the normal parser path reports the real diagnostic",
        );
        let paths_again = loader.type_arity_preload_paths_with_cache(
            &entry_path,
            FileId(0),
            "#target wasm\nlet broken \"unterminated\n",
            true,
            Some(&mut session_cache),
        );
        assert!(
            paths_again.is_empty(),
            "cached lexer-error surfaces must preserve the old no-preload behavior",
        );
        assert_eq!(
            session_cache.stats().arity_surface_hits,
            1,
            "the lexer-error surface is still a pure source-hash query and can be reused safely",
        );
    }

    #[test]
    fn provider_session_cache_can_prewarm_stdlib_loader_queries() {
        let entry_path = canonicalize_path(&PathBuf::from("C:/nepl-test/user/main.nepl"));
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let foo_path = canonicalize_path(&stdlib_path(&stdlib_root, &["foo.nepl"]));
        let defs_path = canonicalize_path(&stdlib_path(&stdlib_root, &["defs.nepl"]));
        let entry_source = String::from(
            "#no_prelude\n#import \"foo\" as *\nfn main %fn unit i32 \\unit:\n    foo unit\n",
        );
        let mut sources = BTreeMap::new();
        sources.insert(
            foo_path.clone(),
            String::from("#import \"defs\" as *\npub fn foo %fn unit Box i32 \\unit:\n    Box 1\n"),
        );
        sources.insert(
            defs_path,
            String::from("pub struct Box<.T>:\n    value %.T\n"),
        );
        let loader = Loader::new(stdlib_root.clone());
        let mut session_cache = LoaderSessionCache::new("test-stdlib");

        {
            let mut provider = |path: &PathBuf| {
                sources
                    .get(path)
                    .cloned()
                    .ok_or_else(|| LoaderError::Io(format!("missing test source: {:?}", path)))
            };
            let warmed = loader
                .prewarm_provider_cache_for_source(
                    entry_path.clone(),
                    &entry_source,
                    &mut provider,
                    &mut session_cache,
                )
                .expect("prewarm should load stdlib roots through the provider cache");
            assert_eq!(
                warmed, 1,
                "prewarm should count configured stdlib roots reached from the root source",
            );
        }

        let after_prewarm = session_cache.stats();
        assert!(
            after_prewarm.parsed_module_stores >= 2,
            "prewarm should store parsed modules for the root and its imported arity dependency",
        );
        assert!(
            after_prewarm.arity_surface_stores >= 2,
            "prewarm should store source arity surfaces for the same stdlib dependency graph",
        );
        assert!(
            after_prewarm.public_surface_hash_stores >= 2,
            "prewarm should compute stable public-surface hashes for parsed stdlib modules",
        );
        assert!(
            after_prewarm.arity_surface_bypasses >= 1,
            "the entry source itself is not a long-lived stdlib artifact and should bypass arity caching",
        );

        let mut provider = |path: &PathBuf| {
            sources
                .get(path)
                .cloned()
                .ok_or_else(|| LoaderError::Io(format!("missing test source: {:?}", path)))
        };
        let mut fresh_loader = Loader::new(stdlib_root);
        fresh_loader
            .load_inline_with_provider_and_cache(
                entry_path,
                entry_source,
                &mut provider,
                &mut session_cache,
            )
            .expect("compile-time load should reuse prewarmed stdlib artifacts");
        let after_load = session_cache.stats();
        assert!(
            after_load.parsed_module_hits > after_prewarm.parsed_module_hits,
            "a later load in the same session should hit the prewarmed parsed-module cache",
        );
        assert!(
            after_load.arity_surface_hits > after_prewarm.arity_surface_hits,
            "a later load in the same session should hit the prewarmed arity-surface cache",
        );
        assert!(
            after_load.public_surface_hash_hits > after_prewarm.public_surface_hash_hits,
            "a later load in the same session should observe prewarmed public-surface hashes",
        );
    }

    #[test]
    fn shallow_type_arity_cache_does_not_reuse_cycle_partial_hints() {
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let loader = Loader::new(stdlib_root.clone());
        let a_path = canonicalize_path(&stdlib_path(&stdlib_root, &["a.nepl"]));
        let b_path = canonicalize_path(&stdlib_path(&stdlib_root, &["b.nepl"]));
        let mut sources = BTreeMap::new();
        sources.insert(
            a_path.clone(),
            String::from("#import pub \"b\" as *\npub struct AType<.T>:\n    value %.T\n"),
        );
        sources.insert(
            b_path.clone(),
            String::from("#import pub \"a\" as *\npub struct BType<.T>:\n    value %.T\n"),
        );
        let mut session_cache = LoaderSessionCache::new("test-stdlib");
        let mut shallow_type_arity_cache = BTreeMap::new();

        {
            let mut provider = |path: &PathBuf| {
                sources
                    .get(path)
                    .cloned()
                    .ok_or_else(|| LoaderError::Io(format!("missing test source: {:?}", path)))
            };
            let mut visited = BTreeSet::new();
            let hints = loader
                .shallow_type_arity_hints_from_file_with(
                    &a_path,
                    FileId(0),
                    &mut visited,
                    &mut shallow_type_arity_cache,
                    &mut provider,
                    Some(&mut session_cache),
                )
                .expect("the first cycle traversal should return the whole reachable type surface");
            assert!(
                hints.hints.iter().any(|(name, _)| name == "AType")
                    && hints.hints.iter().any(|(name, _)| name == "BType"),
                "cycle recovery should keep every reachable type constructor visible to prefix type parsing",
            );
        }

        let mut provider = |path: &PathBuf| {
            sources
                .get(path)
                .cloned()
                .ok_or_else(|| LoaderError::Io(format!("missing test source: {:?}", path)))
        };
        let mut visited = BTreeSet::new();
        let hints = loader
            .shallow_type_arity_hints_from_file_with(
                &b_path,
                FileId(0),
                &mut visited,
                &mut shallow_type_arity_cache,
                &mut provider,
                Some(&mut session_cache),
            )
            .expect("the second cycle traversal should not reuse a partial cache entry");

        assert!(
            hints.hints.iter().any(|(name, _)| name == "AType")
                && hints.hints.iter().any(|(name, _)| name == "BType"),
            "cycle-dependent shallow arity results are intentionally not cached as complete aggregate surfaces",
        );
    }

    #[test]
    fn shallow_arity_surface_rechecks_public_reexport_dependency_source_hash() {
        let stdlib_root = PathBuf::from("C:/nepl-test/stdlib");
        let loader = Loader::new(stdlib_root.clone());
        let facade_path = canonicalize_path(&stdlib_path(&stdlib_root, &["facade.nepl"]));
        let types_path = canonicalize_path(&stdlib_path(&stdlib_root, &["types.nepl"]));
        let mut sources = BTreeMap::new();
        sources.insert(
            facade_path.clone(),
            String::from("#import pub \"types\" as *\n"),
        );
        let mut session_cache = LoaderSessionCache::new("test-stdlib");

        for (src, expected_arity) in [
            ("pub struct Box<.T>:\n    value %.T\n", 1),
            ("pub struct Box<.T,.U>:\n    first %.T\n    second %.U\n", 2),
        ] {
            sources.insert(types_path.clone(), String::from(src));
            let mut provider = |path: &PathBuf| {
                sources
                    .get(path)
                    .cloned()
                    .ok_or_else(|| LoaderError::Io(format!("missing test source: {:?}", path)))
            };
            let mut visited = BTreeSet::new();
            let mut shallow_type_arity_cache = BTreeMap::new();
            let hints = loader
                .shallow_type_arity_hints_from_file_with(
                    &facade_path,
                    FileId(0),
                    &mut visited,
                    &mut shallow_type_arity_cache,
                    &mut provider,
                    Some(&mut session_cache),
                )
                .expect("shallow public reexport arity discovery should read provider sources");
            assert!(
                hints
                    .hints
                    .iter()
                    .any(|(name, arity)| name == "Box" && *arity == expected_arity),
                "dependency source hash changes must be observed even when the facade surface is cached",
            );
        }

        let stats = session_cache.stats();
        assert!(
            stats.arity_surface_hits >= 1,
            "the unchanged facade should hit the arity surface cache on the second pass",
        );
        assert!(
            stats.arity_surface_stores >= 3,
            "the facade plus both dependency source versions should be stored under separate surface keys",
        );
    }

    #[test]
    fn raw_memory_boundary_uses_configured_stdlib_root_and_source_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["future", "raw_boundary.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn helper <(i32)->i32> (ptr):\n    load_i32 ptr\n",
        );
        assert!(
            capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "compiler-owned stdlib source with raw operation evidence must receive operation capability without a module allowlist"
        );
        assert!(
            !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Store),
            "raw operation evidence for load must not authorize store"
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary(),
            "raw operation evidence must not authorize restricted raw structure manipulation"
        );
    }

    #[test]
    fn collection_slot_lifecycle_boundary_uses_typed_source_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "vec", "slot_boundary.nepl"],
        ));
        let src = concat!(
            "fn helper <.T> <(MemPtr<.T>,i32)->()> (ptr, offset):\n",
            "    #intrinsic \"collection_slot_initialize_empty\" <.T> (ptr, offset)\n",
        );
        let capabilities = load_source_capabilities(&loader, path, src);
        let intrinsic_start = src
            .find("\"collection_slot_initialize_empty\"")
            .expect("collection slot lifecycle intrinsic") as u32;
        let intrinsic_span = Span::new(
            FileId(0),
            intrinsic_start,
            intrinsic_start + "\"collection_slot_initialize_empty\"".len() as u32,
        );
        let unrelated_span = Span::new(FileId(0), 0, "fn".len() as u32);

        assert!(capabilities.allows_collection_slot_lifecycle_boundary(
            CollectionSlotLifecyclePrimitive::InitializeEmpty
        ));
        assert!(
            capabilities.allows_collection_slot_lifecycle_boundary_at(
                CollectionSlotLifecyclePrimitive::InitializeEmpty,
                intrinsic_span
            ),
            "collection slot lifecycle authority must attach to the exact intrinsic use site"
        );
        assert!(
            !capabilities.allows_collection_slot_lifecycle_boundary_at(
                CollectionSlotLifecyclePrimitive::InitializeEmpty,
                unrelated_span
            ),
            "collection slot lifecycle authority must not become file-wide"
        );
        assert!(
            !capabilities.allows_collection_slot_lifecycle_boundary(
                CollectionSlotLifecyclePrimitive::StorageDealloc
            ),
            "one collection slot lifecycle primitive must not authorize another"
        );

        let user_capabilities = load_source_capabilities(
            &loader,
            canonicalize_path(&path_from_segments(
                "C:/nepl-test/user",
                &["slot_boundary.nepl"],
            )),
            src,
        );
        assert!(
            !user_capabilities.allows_collection_slot_lifecycle_boundary(
                CollectionSlotLifecyclePrimitive::InitializeEmpty
            ),
            "matching source text outside configured stdlib must not receive collection slot lifecycle authority"
        );
    }

    #[test]
    fn collection_slot_borrow_boundary_uses_typed_source_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "vec", "borrow_boundary.nepl"],
        ));
        let src = concat!(
            "fn helper <.T> <(&RegionToken<.T>,i32)->&.T> (storage, offset):\n",
            "    #intrinsic \"collection_slot_borrow_ref\" <.T> (storage, offset)\n",
        );
        let capabilities = load_source_capabilities(&loader, path, src);

        assert!(
            capabilities
                .allows_collection_slot_borrow_boundary(CollectionSlotBorrowPrimitive::BorrowRef),
            "private compiler-owned slot borrow helpers must receive exact borrow-ref authority"
        );
        assert!(
            !capabilities.allows_collection_slot_lifecycle_boundary(
                CollectionSlotLifecyclePrimitive::BorrowRead
            ),
            "borrow-ref materialization authority must not silently authorize unit BorrowRead intrinsics"
        );

        let user_capabilities = load_source_capabilities(
            &loader,
            canonicalize_path(&path_from_segments(
                "C:/nepl-test/user",
                &["borrow_boundary.nepl"],
            )),
            src,
        );
        assert!(
            !user_capabilities
                .allows_collection_slot_borrow_boundary(CollectionSlotBorrowPrimitive::BorrowRef),
            "matching source text outside configured stdlib must not receive slot borrow authority"
        );
    }

    #[test]
    fn private_cache_boundary_uses_configured_stdlib_source_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "memo", "private_cache_boundary.nepl"],
        ));
        let src = concat!(
            "fn helper <()->()> ():\n",
            "    #intrinsic \"private_cache_lookup\" <> ()\n",
        );
        let capabilities = load_source_capabilities(&loader, path, src);
        let intrinsic_name_start = src
            .find("\"private_cache_lookup\"")
            .expect("private cache intrinsic") as u32;
        let intrinsic_name_span = Span::new(
            FileId(0),
            intrinsic_name_start,
            intrinsic_name_start + "\"private_cache_lookup\"".len() as u32,
        );
        let intrinsic_start = src.find("#intrinsic").expect("private cache intrinsic") as u32;
        let intrinsic_end = src.rfind("()").expect("private cache intrinsic args") as u32 + 2;
        let intrinsic_span = Span::new(FileId(0), intrinsic_start, intrinsic_end);
        let unrelated_span = Span::new(FileId(0), 0, "fn".len() as u32);

        assert!(capabilities.allows_private_cache_boundary(PrivateCacheOp::Lookup));
        assert!(
            capabilities.allows_private_cache_boundary_at(
                PrivateCacheOp::Lookup,
                intrinsic_span
            ),
            "private cache authority must attach to the exact compiler-owned intrinsic use site"
        );
        assert!(
            !capabilities
                .allows_private_cache_boundary_at(PrivateCacheOp::Lookup, intrinsic_name_span),
            "private cache authority must use the Resource IR effect expression span, not the intrinsic-name token span"
        );
        assert!(
            !capabilities
                .allows_private_cache_boundary_at(PrivateCacheOp::Lookup, unrelated_span),
            "private cache authority must not become file-wide"
        );
        assert!(
            !capabilities.allows_private_cache_boundary(PrivateCacheOp::Insert),
            "one private cache operation must not authorize another"
        );

        let user_capabilities = load_source_capabilities(
            &loader,
            canonicalize_path(&path_from_segments(
                "C:/nepl-test/user",
                &["private_cache_boundary.nepl"],
            )),
            src,
        );
        assert!(
            !user_capabilities.allows_private_cache_boundary(PrivateCacheOp::Lookup),
            "matching source text outside configured stdlib must not receive private cache authority"
        );
    }

    #[test]
    fn collection_slot_lifecycle_boundary_is_internal_not_public_surface() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "vec", "slot_boundary.nepl"],
        ));
        let internal_src = concat!(
            "fn internal_slot <.T> <(MemPtr<.T>,i32)->()> (ptr, offset):\n",
            "    #intrinsic \"collection_slot_initialize_empty\" <.T> (ptr, offset)\n",
        );
        let public_src = concat!(
            "pub fn public_slot <.T> <(MemPtr<.T>,i32)->()> (ptr, offset):\n",
            "    #intrinsic \"collection_slot_initialize_empty\" <.T> (ptr, offset)\n",
        );
        let public_alias_src = concat!(
            "fn internal_slot <.T> <(MemPtr<.T>,i32)->()> (ptr, offset):\n",
            "    #intrinsic \"collection_slot_initialize_empty\" <.T> (ptr, offset)\n",
            "pub fn public_slot internal_slot;\n",
        );
        let public_alias_chain_src = concat!(
            "fn internal_slot <.T> <(MemPtr<.T>,i32)->()> (ptr, offset):\n",
            "    #intrinsic \"collection_slot_initialize_empty\" <.T> (ptr, offset)\n",
            "fn alias_slot internal_slot;\n",
            "pub fn public_slot alias_slot;\n",
        );
        let public_wrapper_src = concat!(
            "fn internal_slot <.T> <(MemPtr<.T>,i32)->()> (ptr, offset):\n",
            "    #intrinsic \"collection_slot_initialize_empty\" <.T> (ptr, offset)\n",
            "pub fn public_slot <.T> <(MemPtr<.T>,i32)->()> (ptr, offset):\n",
            "    internal_slot ptr offset\n",
        );
        let public_wrapper_chain_src = concat!(
            "fn internal_slot <.T> <(MemPtr<.T>,i32)->()> (ptr, offset):\n",
            "    #intrinsic \"collection_slot_initialize_empty\" <.T> (ptr, offset)\n",
            "fn wrapper_slot <.T> <(MemPtr<.T>,i32)->()> (ptr, offset):\n",
            "    internal_slot ptr offset\n",
            "pub fn public_slot <.T> <(MemPtr<.T>,i32)->()> (ptr, offset):\n",
            "    wrapper_slot ptr offset\n",
        );
        let public_raw_adapter_src = concat!(
            "fn internal_slot <.T> <(MemPtr<.T>,i32)->()> (ptr, offset):\n",
            "    #intrinsic \"collection_slot_initialize_empty\" <.T> (ptr, offset)\n",
            "pub fn public_slot <.T> <(MemPtr<.T>,i32)->()> (ptr, offset):\n",
            "    let copied_offset offset\n",
            "    internal_slot ptr copied_offset\n",
        );
        let encapsulated_public_api_src = concat!(
            "pub struct CollectionOwner<.T>:\n",
            "    raw <i32>\n",
            "fn internal_slot <.T> <(MemPtr<.T>,i32)->()> (ptr, offset):\n",
            "    #intrinsic \"collection_slot_initialize_empty\" <.T> (ptr, offset)\n",
            "pub fn push_like <.T> <(CollectionOwner<.T>)->CollectionOwner<.T>> (owner):\n",
            "    let ptr owner\n",
            "    internal_slot ptr 0\n",
            "    owner\n",
        );

        let internal_capabilities = load_source_capabilities(&loader, path.clone(), internal_src);
        assert!(
            internal_capabilities.allows_collection_slot_lifecycle_boundary(
                CollectionSlotLifecyclePrimitive::InitializeEmpty
            ),
            "private compiler-owned lowering helpers may carry collection slot lifecycle authority"
        );

        let public_capabilities = load_source_capabilities(&loader, path.clone(), public_src);
        assert!(
            !public_capabilities.allows_collection_slot_lifecycle_boundary(
                CollectionSlotLifecyclePrimitive::InitializeEmpty
            ),
            "public stdlib functions must not expose collection slot lifecycle intrinsic authority"
        );

        let alias_capabilities = load_source_capabilities(&loader, path.clone(), public_alias_src);
        assert!(
            !alias_capabilities.allows_collection_slot_lifecycle_boundary(
                CollectionSlotLifecyclePrimitive::InitializeEmpty
            ),
            "public aliases must not re-export internal collection slot lifecycle lowering helpers"
        );

        let alias_chain_capabilities =
            load_source_capabilities(&loader, path.clone(), public_alias_chain_src);
        assert!(
            !alias_chain_capabilities.allows_collection_slot_lifecycle_boundary(
                CollectionSlotLifecyclePrimitive::InitializeEmpty
            ),
            "public alias chains must not re-export internal collection slot lifecycle lowering helpers"
        );

        let wrapper_capabilities =
            load_source_capabilities(&loader, path.clone(), public_wrapper_src);
        assert!(
            !wrapper_capabilities.allows_collection_slot_lifecycle_boundary(
                CollectionSlotLifecyclePrimitive::InitializeEmpty
            ),
            "public wrappers must not indirectly expose collection slot lifecycle lowering helpers"
        );

        let wrapper_chain_capabilities =
            load_source_capabilities(&loader, path.clone(), public_wrapper_chain_src);
        assert!(
            !wrapper_chain_capabilities.allows_collection_slot_lifecycle_boundary(
                CollectionSlotLifecyclePrimitive::InitializeEmpty
            ),
            "public wrapper chains must not indirectly expose collection slot lifecycle lowering helpers"
        );

        let raw_adapter_capabilities =
            load_source_capabilities(&loader, path.clone(), public_raw_adapter_src);
        assert!(
            !raw_adapter_capabilities.allows_collection_slot_lifecycle_boundary(
                CollectionSlotLifecyclePrimitive::InitializeEmpty
            ),
            "public raw-pointer APIs must not expose collection slot lifecycle helpers through local adapters"
        );

        let encapsulated_public_api_capabilities =
            load_source_capabilities(&loader, path, encapsulated_public_api_src);
        assert!(
            encapsulated_public_api_capabilities.allows_collection_slot_lifecycle_boundary(
                CollectionSlotLifecyclePrimitive::InitializeEmpty
            ),
            "public collection APIs may use private lifecycle helpers internally when they do not re-export the raw marker boundary"
        );
    }

    #[test]
    fn raw_memory_boundary_requires_source_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "raw.nepl"],
        ));

        let safe =
            load_source_capabilities(&loader, path.clone(), "fn helper <()->i32> ():\n    1\n");
        assert!(
            !safe.allows_raw_memory_structural_boundary()
                && !safe.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "configured raw boundary candidate without raw evidence must not receive capability"
        );

        let raw = load_source_capabilities(
            &loader,
            path,
            "fn helper <(i32)->i32> (ptr):\n    load_i32 ptr\n",
        );
        assert!(
            raw.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "configured raw boundary candidate with raw helper evidence must receive matching operation capability"
        );
        assert!(!raw.allows_raw_memory_operation_boundary(RawMemoryOp::Store));
    }

    #[test]
    fn raw_memory_function_boundary_requires_matching_direct_operation_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["future", "raw_operation_mismatch.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!("fn load_i32 <(i32)->i32> (ptr):\n", "    store_i32 ptr 1\n",),
        );

        assert!(
            !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "a load-named raw boundary must not be proven by store-only source evidence"
        );
        assert!(
            capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Store),
            "the observed store operation itself remains a precise use-site proof"
        );
    }

    #[test]
    fn raw_memory_function_boundary_accepts_compatible_loop_operation_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["future", "raw_fill_loop.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "fn memset_u8 <(i32,i32,i32)->()> (ptr, len, value):\n",
                "    store_u8 ptr value\n",
            ),
        );

        assert!(
            capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::FillBytes),
            "byte-fill boundaries may be proven by byte-store source evidence"
        );
        assert!(
            !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Fill),
            "byte-store evidence must not prove typed i32 fill authority"
        );
    }

    #[test]
    fn raw_memory_function_boundary_requires_matching_raw_body_operation_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["future", "raw_body_operation_mismatch.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "fn load_i32 <(i32)->i32> (ptr):\n",
                "    #wasm:\n",
                "        local.get $ptr\n",
                "        i32.store\n",
            ),
        );

        assert!(
            capabilities.allows_raw_body_memory_operation_boundary(RawBodyMemoryOp::Wasm(
                WasmRawBodyMemoryOp::Store
            )),
            "the raw-body memory operation must still be recorded as typed evidence"
        );
        assert!(
            !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "a load-named raw boundary must not be proven by store-only raw-body evidence"
        );
    }

    #[test]
    fn raw_memory_function_boundary_accepts_matching_raw_body_operation_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["future", "raw_body_operation_match.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "fn store_i32 <(i32,i32)->()> (ptr, value):\n",
                "    #wasm:\n",
                "        local.get $ptr\n",
                "        local.get $value\n",
                "        i32.store\n",
            ),
        );

        assert!(
            capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Store),
            "store raw-body evidence must prove a store boundary"
        );
        assert!(
            !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "store raw-body evidence must not prove unrelated load authority"
        );
    }

    #[test]
    fn owner_aggregate_boundaries_require_matching_source_evidence() {
        let loader = real_test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "vec", "types.nepl"],
        ));

        let safe =
            load_source_capabilities(&loader, path.clone(), "fn helper <()->i32> ():\n    1\n");
        assert!(
            !safe.allows_owner_aggregate_constructor_boundary("Vec"),
            "configured stdlib source without constructor evidence must not receive owner aggregate constructor capability"
        );
        assert!(
            !safe.allows_owner_aggregate_field_boundary(),
            "configured stdlib source without field evidence must not receive owner aggregate field capability"
        );

        let aggregate = load_source_capabilities(
            &loader,
            path,
            concat!(
                "#import \"core/field\" as field\n\n",
                "fn helper <(OwnerBox<i32>)->i32> (v):\n",
                "    field::get v \"owner\"\n",
            ),
        );
        assert!(
            !aggregate.allows_owner_aggregate_constructor_boundary("Vec"),
            "field accessor evidence must not grant owner aggregate constructor capability"
        );
        assert!(
            aggregate.allows_owner_aggregate_field_boundary(),
            "configured stdlib source with field accessor evidence must receive owner aggregate field capability"
        );
        assert!(
            !aggregate.allows_compiler_memory_field_boundary(CompilerMemoryField::Raw)
                && !aggregate.allows_compiler_memory_field_boundary(CompilerMemoryField::Size),
            "ordinary owner aggregate field access must not prove compiler memory representation-field authority"
        );
        assert!(
            !aggregate.allows_raw_memory_structural_boundary()
                && !aggregate.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "owner aggregate manipulation is not raw memory operation authority"
        );
    }

    #[test]
    fn owner_aggregate_boundary_accepts_intrinsic_field_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "types.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "fn helper <.T> <(&RegionToken<.T>)->&i32> (token):\n",
                "    #intrinsic \"get_field_ref\" <> (token,\"raw\")\n",
            ),
        );
        assert!(
            capabilities.allows_owner_aggregate_field_boundary(),
            "compiler-owned intrinsic field reference must be owner aggregate field evidence"
        );
        assert!(
            capabilities.allows_compiler_memory_field_boundary(CompilerMemoryField::Raw),
            "compiler-owned intrinsic raw field reference must prove the exact compiler memory field"
        );
        assert!(
            !capabilities.allows_compiler_memory_field_boundary(CompilerMemoryField::Size),
            "compiler memory field source proof must remain tied to the observed field selector"
        );
        assert!(
            !capabilities.allows_owner_aggregate_constructor_boundary("RegionToken"),
            "intrinsic field evidence must not grant owner aggregate constructor capability"
        );
    }

    #[test]
    fn owner_aggregate_boundary_rejects_set_field_intrinsic_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "types.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "fn helper <.T> <(&RegionToken<.T>,i32)->()> (token,value):\n",
                "    #intrinsic \"set_field\" <> (token,\"raw\",value)\n",
            ),
        );
        assert!(
            !capabilities.allows_owner_aggregate_field_boundary(),
            "write intrinsics must not prove owner aggregate field read/reference boundary"
        );
        assert!(
            !capabilities.allows_compiler_memory_field_boundary(CompilerMemoryField::Raw),
            "write intrinsics must not prove compiler memory field read/reference boundary"
        );
    }

    #[test]
    fn compiler_memory_field_boundary_requires_representation_field_selector() {
        let loader = real_test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["future", "compiler_memory_field.nepl"],
        ));
        let raw_capabilities = load_source_capabilities(
            &loader,
            path.clone(),
            concat!(
                "#import \"core/field\" as *\n\n",
                "fn helper <.T> <(MemPtr<.T>)->i32> (ptr):\n",
                "    get ptr \"raw\"\n",
            ),
        );
        assert!(raw_capabilities.allows_owner_aggregate_field_boundary());
        assert!(raw_capabilities.allows_compiler_memory_field_boundary(CompilerMemoryField::Raw));
        assert!(!raw_capabilities.allows_compiler_memory_field_boundary(CompilerMemoryField::Size));

        let ordinary_capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "#import \"core/field\" as *\n\n",
                "fn helper <.T> <(PtrHolder<.T>)->MemPtr<.T>> (holder):\n",
                "    get holder \"ptr\"\n",
            ),
        );
        assert!(ordinary_capabilities.allows_owner_aggregate_field_boundary());
        assert!(
            !ordinary_capabilities.allows_compiler_memory_field_boundary(CompilerMemoryField::Raw)
                && !ordinary_capabilities
                    .allows_compiler_memory_field_boundary(CompilerMemoryField::Size),
            "field accessors whose selector is not a compiler memory representation field must not gain compiler memory field proof"
        );
    }

    #[test]
    fn owner_aggregate_boundary_accepts_constructor_syntax_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "owner_box", "construct.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "fn helper <(RegionToken<i32>)->OwnerBox<i32>> (region):\n",
                "    OwnerBox<i32> region\n",
            ),
        );
        assert!(
            capabilities.allows_owner_aggregate_constructor_boundary("OwnerBox"),
            "compiler-owned aggregate constructor syntax is constructor boundary evidence"
        );
        assert!(
            !capabilities.allows_owner_aggregate_constructor_boundary("Diag"),
            "constructor evidence for one aggregate name must not authorize another constructor"
        );
        assert!(
            !capabilities.allows_owner_aggregate_field_boundary(),
            "constructor evidence must not grant owner token field projection capability"
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "aggregate constructor syntax does not grant raw memory boundary capability"
        );
    }

    #[test]
    fn owner_aggregate_boundary_accepts_constructor_initializer_call_head() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "owner_box", "init.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "fn helper <(RegionToken<i32>)->OwnerBox<i32>> (region):\n",
                "    let boxed <OwnerBox<i32>> OwnerBox<i32> region;\n",
                "    boxed\n",
            ),
        );
        assert!(
            capabilities.allows_owner_aggregate_constructor_boundary("OwnerBox"),
            "constructor after let/type annotation is still prefix call-head evidence"
        );
        assert!(
            !capabilities.allows_owner_aggregate_constructor_boundary("RegionToken"),
            "initializer constructor evidence remains tied to the constructed aggregate name"
        );
    }

    #[test]
    fn owner_aggregate_boundary_accepts_nested_explicit_constructor_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "vec", "storage", "alloc.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "fn helper <.T> <(RegionToken<.T>)->Result<Vec<.T>, StdErrorKind>> (region):\n",
                "    Result::Ok<Vec<.T>, StdErrorKind> Vec<.T> (OwnedBuffer<.T> 0 0 1 (VecStorage<.T>::Owned region))\n",
            ),
        );
        assert!(
            capabilities.allows_owner_aggregate_constructor_boundary("Vec"),
            "explicit generic constructor nested as a Result payload must be owner aggregate constructor evidence"
        );
        assert!(
            capabilities.allows_owner_aggregate_constructor_boundary("OwnedBuffer"),
            "explicit generic constructor nested inside another constructor must also be evidence"
        );
        assert!(
            !capabilities.allows_owner_aggregate_constructor_boundary("Ok"),
            "qualified enum variant wrappers must not become owner aggregate constructor evidence"
        );
    }

    #[test]
    fn owner_aggregate_boundary_accepts_nested_nongeneric_constructor_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &[
                "alloc",
                "collections",
                "adjacency_matrix",
                "api",
                "create.nepl",
            ],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "fn helper <(i32,i32,Bits)->Result<AdjacencyMatrix, Diag>> (nverts,nbytes,bits):\n",
                "    ok<AdjacencyMatrix, Diag> AdjacencyMatrix nverts nbytes bits\n",
            ),
        );
        assert!(
            capabilities.allows_owner_aggregate_constructor_boundary("AdjacencyMatrix"),
            "nested non-generic owner aggregate constructors must be source evidence"
        );
        assert!(
            !capabilities.allows_owner_aggregate_constructor_boundary("Ok"),
            "qualified enum variant wrappers remain outside owner aggregate constructor evidence"
        );
    }

    #[test]
    fn owner_aggregate_boundary_accepts_field_initializer_call_head() {
        let loader = real_test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "vec", "access.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "#import \"core/field\" as *\n\n",
                "fn helper <(OwnerBox<i32>)->i32> (v):\n",
                "    let owner <i32> get v \"owner\";\n",
                "    owner\n",
            ),
        );
        assert!(
            capabilities.allows_owner_aggregate_field_boundary(),
            "field accessor after let/type annotation is still prefix call-head evidence"
        );
        assert!(
            !capabilities.allows_owner_aggregate_constructor_boundary("OwnerBox"),
            "field initializer evidence must not grant owner aggregate constructor capability"
        );
    }

    #[test]
    fn owner_aggregate_boundary_rejects_unrelated_get_call_head() {
        let loader = real_test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "vec", "query.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "#import \"alloc/collections/vec/query/get\" as *\n\n",
                "fn helper <()->i32> ():\n",
                "    get items 0\n",
            ),
        );
        assert!(
            !capabilities.allows_owner_aggregate_field_boundary(),
            "ordinary get helpers imported from non-field modules are not owner field evidence"
        );
    }

    #[test]
    fn owner_aggregate_boundary_accepts_field_alias_import_call_head() {
        let loader = real_test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "vec", "storage.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "#import \"core/field\" as f\n\n",
                "fn helper <(OwnerBox<i32>)->i32> (v):\n",
                "    f::get v \"owner\"\n",
            ),
        );
        assert!(
            capabilities.allows_owner_aggregate_field_boundary(),
            "core/field alias calls are owner aggregate field source evidence"
        );
    }

    #[test]
    fn owner_aggregate_boundary_accepts_field_merge_import_call_head() {
        let loader = real_test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "vec", "mutation.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "#import \"core/field\" as @merge\n\n",
                "fn helper <(OwnerBox<i32>)->i32> (v):\n",
                "    get v \"owner\"\n",
            ),
        );
        assert!(
            capabilities.allows_owner_aggregate_field_boundary(),
            "core/field merge imports expose field accessors as source evidence"
        );
    }

    #[test]
    fn owner_aggregate_boundary_accepts_same_module_struct_constructor_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "vec", "mutation", "push.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "pub struct VecReallocRegionError<.T>:\n",
                "    region <RegionToken<.T>>\n",
                "    kind <StdErrorKind>\n\n",
                "fn helper <.T> <(RegionToken<.T>)->VecReallocRegionError<.T>> (region):\n",
                "    VecReallocRegionError<.T> region StdErrorKind::OutOfMemory\n",
            ),
        );
        assert!(
            capabilities.allows_owner_aggregate_constructor_boundary("VecReallocRegionError"),
            "same-module struct definitions must not shadow their constructor evidence"
        );
        assert!(
            !capabilities.allows_owner_aggregate_constructor_boundary("RegionToken"),
            "owner aggregate constructor evidence remains tied to the constructed aggregate name"
        );
    }

    #[test]
    fn owner_aggregate_boundary_does_not_share_unrelated_constructor_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "diag", "safe_constructor_user.nepl"],
        ));
        let capabilities =
            load_source_capabilities(&loader, path, "fn helper <()->Diag> ():\n    Diag 1\n");
        assert!(
            capabilities.allows_owner_aggregate_constructor_boundary("Diag"),
            "compiler-owned constructor evidence must be attached to the observed constructor name"
        );
        assert!(
            !capabilities.allows_owner_aggregate_constructor_boundary("Vec"),
            "unrelated constructor evidence must not authorize owner-backed aggregate constructors"
        );
    }

    #[test]
    fn owner_aggregate_boundary_requires_constructor_call_head() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "diag", "argument_value_user.nepl"],
        ));
        let capabilities =
            load_source_capabilities(&loader, path, "fn helper <()->i32> ():\n    consume Diag\n");
        assert!(
            !capabilities.allows_owner_aggregate_constructor_boundary("Diag"),
            "uppercase values used as arguments are not constructor boundary evidence"
        );
    }

    #[test]
    fn owner_aggregate_boundary_ignores_qualified_enum_variant_constructors() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "result_user.nepl"],
        ));
        let capabilities =
            load_source_capabilities(&loader, path, "fn helper <()->i32> ():\n    Result::Ok 1\n");
        assert!(
            !capabilities.allows_owner_aggregate_constructor_boundary("Ok"),
            "qualified enum variants are not owner-backed aggregate constructor evidence"
        );
        assert!(
            !capabilities.allows_owner_aggregate_field_boundary(),
            "qualified enum variants are not owner-token field projection evidence"
        );
    }

    #[test]
    fn owner_aggregate_boundary_ignores_same_module_enum_variant_constructors() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "result_local_user.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "enum LocalResult:\n",
                "    Ok <i32>\n\n",
                "fn helper <()->LocalResult> ():\n",
                "    Ok 1\n",
            ),
        );
        assert!(
            !capabilities.allows_owner_aggregate_constructor_boundary("Ok"),
            "same-module enum variants are not owner-backed aggregate constructor evidence"
        );
        assert!(
            !capabilities.allows_owner_aggregate_field_boundary(),
            "same-module enum variants are not owner-token field projection evidence"
        );
    }

    #[test]
    fn owner_aggregate_boundary_rejects_user_source_even_with_evidence() {
        let loader = real_test_loader();
        let path = canonicalize_path(&path_from_segments("C:/nepl-test/user", &["vec_like.nepl"]));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "#import \"core/field\" as field\n\n",
                "fn helper <(OwnerBox<i32>)->i32> (v):\n",
                "    field::get v \"owner\"\n",
            ),
        );
        assert!(
            !capabilities.allows_owner_aggregate_constructor_boundary("Vec"),
            "aggregate constructor evidence outside configured stdlib must not grant capability"
        );
        assert!(
            !capabilities.allows_owner_aggregate_field_boundary(),
            "field accessor evidence outside configured stdlib must not grant capability"
        );
    }

    #[test]
    fn owner_aggregate_boundary_ignores_shadowed_helper_names() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "vec", "safe_shadow.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn get <()->i32> ():\n    1\n\nfn helper <()->i32> ():\n    get\n",
        );
        assert!(
            !capabilities.allows_owner_aggregate_constructor_boundary("Get"),
            "same-module safe helper names are not owner aggregate boundary evidence"
        );
        assert!(
            !capabilities.allows_owner_aggregate_field_boundary(),
            "same-module safe helper names are not owner field boundary evidence"
        );
    }

    #[test]
    fn compiler_memory_type_definitions_use_source_shape_not_raw_boundary_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "types.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "pub struct MemPtr<.T>:\n",
                "    raw <i32>\n\n",
                "pub struct RegionToken<.T>:\n",
                "    raw <i32>\n",
                "    size <i32>\n",
            ),
        );

        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "compiler-owned memory type definitions are not raw operation authority"
        );
        assert!(capabilities.allows_compiler_memory_type_definition(
            crate::source_map::CompilerMemoryType::RawPointer
        ));
        assert!(capabilities.allows_compiler_memory_type_definition(
            crate::source_map::CompilerMemoryType::OwnerToken
        ));
    }

    #[test]
    fn compiler_memory_type_definition_requires_canonical_core_mem_types_path() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "fake_types.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "pub struct MemPtr<.T>:\n",
                "    raw <i32>\n\n",
                "pub struct RegionToken<.T>:\n",
                "    raw <i32>\n",
                "    size <i32>\n",
            ),
        );

        assert!(!capabilities.allows_compiler_memory_type_definition(
            crate::source_map::CompilerMemoryType::RawPointer
        ));
        assert!(!capabilities.allows_compiler_memory_type_definition(
            crate::source_map::CompilerMemoryType::OwnerToken
        ));
    }

    #[test]
    fn actual_core_mem_types_expose_both_compiler_memory_type_capabilities() {
        let loader = real_test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "types.nepl"],
        ));
        let src = read_file_to_string(&path).expect("read core/mem/types.nepl");
        let capabilities = load_source_capabilities(&loader, path, &src);

        assert!(capabilities.allows_compiler_memory_type_definition(
            crate::source_map::CompilerMemoryType::RawPointer
        ));
        assert!(capabilities.allows_compiler_memory_type_definition(
            crate::source_map::CompilerMemoryType::OwnerToken
        ));
    }

    #[test]
    fn imported_region_token_span_keeps_owner_token_capability() {
        let mut loader = real_test_loader();
        let loaded = loader
            .load_inline(
                PathBuf::from("region_token_span_probe.nepl"),
                String::from("#import \"core/mem\" as *\n"),
            )
            .expect("load core/mem import");
        let region_token = loaded
            .module
            .root
            .items
            .iter()
            .find_map(|stmt| match stmt {
                Stmt::StructDef(def) if def.name.name == "RegionToken" => Some(def),
                _ => None,
            })
            .expect("RegionToken struct is imported");

        assert!(loaded
            .source_map
            .compiler_memory_type_definition_allowed_at(
                region_token.name.span,
                crate::source_map::CompilerMemoryType::OwnerToken,
            ));
    }

    #[test]
    fn compiler_memory_type_definition_requires_configured_stdlib_source_path() {
        let loader = test_loader();
        let path = canonicalize_path(&PathBuf::from("C:/nepl-user/types.nepl"));
        let capabilities =
            load_source_capabilities(&loader, path, "pub struct MemPtr<.T>:\n    raw <i32>\n");

        assert!(!capabilities.allows_compiler_memory_type_definition(
            crate::source_map::CompilerMemoryType::RawPointer
        ));
    }

    #[test]
    fn compiler_memory_type_definition_requires_exact_struct_shape() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "future_types.nepl"],
        ));
        let capabilities =
            load_source_capabilities(&loader, path, "pub struct MemPtr<.T>:\n    raw <u8>\n");

        assert!(!capabilities.allows_compiler_memory_type_definition(
            crate::source_map::CompilerMemoryType::RawPointer
        ));
    }

    #[test]
    fn raw_memory_boundary_ignores_shadowed_parameter_names() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["future", "safe_shadow.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn helper <(i32)->i32> (mem_ptr_addr):\n    mem_ptr_addr\n",
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "parameter names that match raw helpers are not source evidence"
        );
    }

    #[test]
    fn raw_memory_boundary_ignores_shadowed_qualified_parameter_names() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["future", "safe_qualified_shadow.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn helper <(i32)->i32> (raw):\n    raw::load_i32 1\n",
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "qualified raw helper-looking symbols with a shadowed qualifier are not source evidence"
        );
    }

    #[test]
    fn raw_memory_boundary_ignores_shadowed_local_names() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["future", "safe_local_shadow.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn helper <()->i32> ():\n    let alloc_raw <i32> 1;\n    alloc_raw\n",
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Alloc),
            "local names that match raw owner helpers are not source evidence"
        );
    }

    #[test]
    fn raw_memory_boundary_ignores_same_module_safe_helper_names() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["future", "safe_module_shadow.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn load_i32 <()->i32> ():\n    1\n\nfn helper <()->i32> ():\n    load_i32\n",
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "same-module safe definitions with raw helper names are not source evidence"
        );
    }

    #[test]
    fn raw_memory_boundary_ignores_impl_method_raw_helper_names() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["future", "safe_impl_method_shadow.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "impl i32:\n",
                "    fn load_i32 <()->i32> ():\n",
                "        load_i32\n",
            ),
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "impl method names that match raw helpers are not source evidence"
        );
    }

    #[test]
    fn raw_memory_boundary_accepts_same_name_raw_helper_wrapper_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "pointer", "scalar.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "pub fn load_u8 <(MemPtr<u8>)->Option<i32>> (p):\n    load_u8 mem_ptr_addr p\n",
        );
        assert!(
            capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::LoadU8),
            "a raw helper wrapper must be allowed to prove its own raw primitive call"
        );
        assert!(
            !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "load_u8 evidence must not grant generic typed load authority"
        );
    }

    #[test]
    fn raw_memory_boundary_accepts_raw_helper_call_in_constructor_payload() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "pointer", "scalar.nepl"],
        ));
        let src = "pub fn load_u8 <(MemPtr<u8>)->Option<i32>> (p):\n    let raw <i32> mem_ptr_addr p;\n    Option<i32>::Some load_u8 raw\n";
        let capabilities = load_source_capabilities(&loader, path, src);
        let call_start = src.rfind("load_u8 raw").expect("raw helper call") as u32;
        let call_span = Span::new(FileId(0), call_start, call_start + "load_u8".len() as u32);
        let unrelated_span = Span::new(FileId(0), call_span.end + 1, call_span.end + 8);
        assert!(
            capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::LoadU8),
            "a raw primitive call used as a constructor payload must remain source evidence"
        );
        assert!(
            capabilities.allows_raw_memory_operation_boundary_at(RawMemoryOp::LoadU8, call_span),
            "source proof must attach raw operation authority to the raw helper call site"
        );
        assert!(
            !capabilities
                .allows_raw_memory_operation_boundary_at(RawMemoryOp::LoadU8, unrelated_span),
            "raw operation authority must not spread to another span in the same file"
        );
        assert!(
            !capabilities.allows_raw_memory_operation_boundary_at(RawMemoryOp::Load, call_span),
            "load_u8 call-site evidence must stay distinct from generic typed load evidence"
        );
    }

    #[test]
    fn raw_memory_boundary_accepts_proven_top_level_raw_helper_call_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "allocator.nepl"],
        ));
        let src = concat!(
            "pub fn alloc_raw <(i32)->i32> (size):\n",
            "    mem_grow size\n",
            "\n",
            "pub fn __nepl_rt_alloc <(i32)->i32> (size):\n",
            "    alloc_raw size\n",
        );
        let capabilities = load_source_capabilities(&loader, path, src);
        let call_start = src.rfind("alloc_raw size").expect("raw helper call") as u32;
        let call_span = Span::new(FileId(0), call_start, call_start + "alloc_raw".len() as u32);
        assert!(
            capabilities.allows_raw_memory_operation_boundary_at(RawMemoryOp::Alloc, call_span),
            "a top-level raw helper call is evidence only after the target helper has source evidence"
        );
    }

    #[test]
    fn raw_memory_boundary_rejects_unproven_top_level_raw_helper_call_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "allocator.nepl"],
        ));
        let src = concat!(
            "pub fn alloc_raw <(i32)->i32> (_size):\n",
            "    0\n",
            "\n",
            "pub fn __nepl_rt_alloc <(i32)->i32> (size):\n",
            "    alloc_raw size\n",
        );
        let capabilities = load_source_capabilities(&loader, path, src);
        let call_start = src.rfind("alloc_raw size").expect("raw helper call") as u32;
        let call_span = Span::new(FileId(0), call_start, call_start + "alloc_raw".len() as u32);
        assert!(
            !capabilities.allows_raw_memory_operation_boundary_at(RawMemoryOp::Alloc, call_span),
            "a top-level raw helper name without source evidence must not prove a raw call site"
        );
    }

    #[test]
    fn raw_memory_boundary_rejects_local_shadow_inside_same_name_raw_helper() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "pointer", "scalar.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "pub fn load_u8 <()->i32> ():\n    let load_u8 <i32> 1;\n    load_u8 0\n",
        );
        assert!(
            !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "local shadowing inside a same-name raw helper must still reject raw evidence"
        );
    }

    #[test]
    fn raw_memory_boundary_requires_raw_operation_call_head() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["future", "raw_operation_argument.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn helper <()->i32> ():\n    consume load_i32\n",
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "raw helper values used as non-call arguments are not raw operation evidence"
        );
    }

    #[test]
    fn raw_memory_boundary_requires_raw_structural_call_head() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["future", "raw_structural_argument.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn helper <()->i32> ():\n    consume mem_ptr_addr\n",
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "raw address helper values used as non-call arguments are not structural boundary evidence"
        );
    }

    #[test]
    fn raw_memory_boundary_accepts_raw_helper_definition_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "allocator.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "pub fn alloc_raw <(i32)->i32> (size):\n",
                "    mem_grow size\n",
            ),
        );
        assert!(
            capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Alloc),
            "allocator growth evidence must grant allocation boundary capability"
        );
        assert!(
            capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::MemoryGrow),
            "raw helper implementation keeps exact operation evidence from its body"
        );
        assert!(!capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Store));
    }

    #[test]
    fn raw_memory_boundary_requires_composite_realloc_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "allocator.nepl"],
        ));
        let incomplete = load_source_capabilities(
            &loader,
            path.clone(),
            concat!(
                "pub fn alloc_raw <(i32)->i32> (size):\n",
                "    mem_grow size\n",
                "\n",
                "pub fn realloc_raw <(i32,i32,i32)->i32> (_ptr, _old_size, new_size):\n",
                "    alloc_raw new_size\n",
            ),
        );
        assert!(
            !incomplete.allows_raw_memory_operation_boundary(RawMemoryOp::Realloc),
            "realloc boundary must not be proven by allocation-only source evidence"
        );

        let complete = load_source_capabilities(
            &loader,
            path,
            concat!(
                "pub fn alloc_raw <(i32)->i32> (size):\n",
                "    mem_grow size\n",
                "\n",
                "pub fn dealloc_raw <(i32,i32)->()> (ptr, size):\n",
                "    store_i32 ptr size\n",
                "\n",
                "pub fn realloc_raw <(i32,i32,i32)->i32> (ptr, old_size, new_size):\n",
                "    let next <i32> alloc_raw new_size\n",
                "    dealloc_raw ptr old_size\n",
                "    next\n",
            ),
        );
        assert!(
            complete.allows_raw_memory_operation_boundary(RawMemoryOp::Realloc),
            "realloc boundary requires both allocation and deallocation source evidence"
        );
    }

    #[test]
    fn raw_memory_boundary_keeps_raw_helper_body_evidence_function_scoped() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "allocator.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "pub fn alloc_raw <(i32)->i32> (size):\n",
                "    fn nested <(i32)->i32> (ptr):\n",
                "        load_i32 ptr\n",
                "    size\n",
            ),
        );
        assert!(
            !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Alloc),
            "raw evidence in a nested function must not grant the outer raw helper name operation"
        );
        assert!(
            capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "nested raw helper body still contributes exact module-level operation evidence"
        );
        assert!(!capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Store));
    }

    #[test]
    fn raw_memory_boundary_does_not_promote_address_view_helper_to_operation_definition() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "pointer", "scalar.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn load_i32 <(MemPtr<i32>)->i32> (ptr):\n    mem_ptr_addr ptr\n",
        );
        assert!(
            capabilities.allows_raw_address_view_boundary(),
            "raw address helper evidence must still prove address-view use"
        );
        assert!(
            !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "raw address view evidence alone must not prove a raw memory load helper body"
        );
    }

    #[test]
    fn raw_memory_boundary_rejects_owner_constructor_helper_as_address_view_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "pointer", "region.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn helper <(MemPtr<i32>)->RegionToken<i32>> (ptr):\n    region_new ptr 4\n",
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && !capabilities.allows_raw_address_view_boundary()
                && !capabilities.allows_raw_address_alias_boundary()
                && capabilities.allows_owner_token_construct_boundary(),
            "owner-token helper calls prove owner-token construction authority, not representation, raw-address-view, or generic alias authority"
        );
    }

    #[test]
    fn raw_memory_boundary_accepts_restricted_constructor_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "internal.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn helper <(i32)->MemPtr<i32>> (raw):\n    MemPtr raw\n",
        );
        assert!(
            capabilities.allows_raw_memory_structural_boundary(),
            "compiler-owned restricted constructors are raw boundary evidence"
        );
        assert!(!capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load));
    }

    #[test]
    fn raw_memory_boundary_accepts_raw_address_helper_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "internal.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn helper <(MemPtr<i32>)->i32> (ptr):\n    mem_ptr_addr ptr\n",
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && capabilities.allows_raw_address_view_boundary(),
            "raw address helper calls prove raw-address-view use, not representation access"
        );
        assert!(!capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load));
    }

    #[test]
    fn raw_memory_boundary_accepts_raw_address_alias_helper_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "internal.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn helper <(i32)->MemPtr<i32>> (raw):\n    mem_ptr_wrap raw\n",
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && !capabilities.allows_raw_address_view_boundary()
                && capabilities.allows_raw_address_alias_boundary(),
            "raw address alias helpers prove alias authority, not view or representation access"
        );
        assert!(!capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load));
    }

    #[test]
    fn raw_memory_boundary_rejects_checked_owner_helper_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "string", "builder", "reserve.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn helper <(i32)->Result<RegionToken<u8>,str>> (n):\n    alloc_region<u8> n\n",
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && !capabilities.allows_raw_address_view_boundary()
                && !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Alloc),
            "checked owner helper calls are safe API usage, not raw boundary evidence"
        );
    }

    #[test]
    fn raw_memory_boundary_accepts_raw_address_intrinsic_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "string", "storage.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn helper <(i32)->str> (raw):\n    #intrinsic \"str_from_addr_unchecked\" <> (raw)\n",
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && capabilities.allows_raw_address_view_boundary(),
            "compiler-owned raw address intrinsics are raw-address-view evidence"
        );
    }

    #[test]
    fn raw_memory_boundary_rejects_user_source_even_with_raw_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&path_from_segments(
            "C:/nepl-test/user",
            &["core", "mem", "raw.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn helper <(i32)->i32> (ptr):\n    load_i32 ptr\n",
        );
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "raw helper evidence outside configured stdlib must not grant capability"
        );
    }

    #[test]
    fn raw_memory_boundary_rejects_candidate_suffix_without_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["core", "mem", "raw.nepl"],
        ));
        let capabilities =
            load_source_capabilities(&loader, path, "fn helper <()->i32> ():\n    1\n");
        assert!(
            !capabilities.allows_raw_memory_structural_boundary()
                && !capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "configured stdlib source without source evidence must not receive capability"
        );
    }

    #[test]
    fn raw_memory_boundary_rejects_same_suffix_outside_configured_stdlib() {
        let loader = test_loader();
        let path = canonicalize_path(&path_from_segments(
            "C:/nepl-test/stdlib2",
            &["core", "mem", "raw.nepl"],
        ));
        assert!(
            !loader.configured_stdlib_source_path(&path),
            "raw memory boundary must not be granted by prefix-like path text: {:?}",
            path
        );
    }

    #[cfg(windows)]
    #[test]
    fn configured_stdlib_source_path_accepts_virtual_child_under_existing_windows_root() {
        let loader = real_test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &[
                "alloc",
                "collections",
                "owner_box",
                "virtual_field_alias.nepl",
            ],
        ));
        assert!(
            !path.exists(),
            "this regression must use a virtual child path under an existing stdlib root"
        );
        assert!(
            loader.configured_stdlib_source_path(&path),
            "virtual stdlib child paths must compare equal to the configured stdlib root prefix: {:?}",
            path
        );
    }
}
