use crate::ast::{Directive, Module, Stmt};
use crate::diagnostic::Severity;
use crate::error::CoreError;
use crate::lexer;
use crate::parser;
use crate::source_capability::{
    module_compiler_memory_type_definitions, module_has_owner_aggregate_field_evidence,
    module_has_raw_memory_boundary_evidence, module_owner_aggregate_constructor_evidence,
    module_raw_body_memory_operation_evidence, module_raw_memory_operation_evidence,
};
use crate::source_map::SourceCapability;
use crate::span::FileId;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::result::Result;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
use std::path::{Component, PathBuf};
extern crate std;

pub use crate::source_map::{SourceCapabilities, SourceMap, SourcePath};

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
            stdlib_root,
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
        let module = match self.load_from_contents(
            path,
            src,
            &mut sm,
            &mut cache,
            &mut processing,
            &mut imported,
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
        let module = match self.load_from_contents_with(
            path,
            src,
            &mut sm,
            &mut cache,
            &mut processing,
            &mut imported,
            true,
            provider,
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

    pub fn load(&mut self, entry: &PathBuf) -> Result<LoadResult, LoaderError> {
        let mut sm = SourceMap::new();
        let mut cache: BTreeMap<PathBuf, Module> = BTreeMap::new();
        let mut processing: BTreeSet<PathBuf> = BTreeSet::new();
        let mut imported: BTreeSet<PathBuf> = BTreeSet::new();
        let module = match self.load_file(
            entry,
            &mut sm,
            &mut cache,
            &mut processing,
            &mut imported,
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
        let module = self.parse_module(file_id, src)?;
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
        is_root: bool,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
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
        let module = self.parse_module(file_id, src)?;
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
            is_root,
            provider,
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
        let module = self.parse_module(file_id, src)?;
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
        is_root: bool,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
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
        let module = self.parse_module(file_id, src)?;
        sm.set_capabilities(
            file_id,
            self.source_capabilities_for_module(&canon, &module),
        );
        let module = self.process_directives_with(
            canon.clone(),
            module,
            sm,
            cache,
            processing,
            imported_once,
            is_root,
            provider,
        )?;
        processing.remove(&canon);
        cache.insert(canon.clone(), module.clone());
        Ok(module)
    }

    fn process_directives(
        &self,
        base: PathBuf,
        module: Module,
        sm: &mut SourceMap,
        cache: &mut BTreeMap<PathBuf, Module>,
        processing: &mut BTreeSet<PathBuf>,
        imported_once: &mut BTreeSet<PathBuf>,
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
                let imp_mod =
                    self.load_file(&target, sm, cache, processing, imported_once, false)?;
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
                        let imp_mod =
                            self.load_file(&target, sm, cache, processing, imported_once, false)?;
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
                    let inc_mod =
                        self.load_file(&target, sm, cache, processing, imported_once, false)?;
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
        is_root: bool,
        provider: &mut dyn FnMut(&PathBuf) -> Result<String, LoaderError>,
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
                    false,
                    provider,
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
                            false,
                            provider,
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
                        false,
                        provider,
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

    fn parse_module(&self, file_id: FileId, src: String) -> Result<Module, CoreError> {
        let lex = lexer::lex(file_id, &src);
        if lex
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
        {
            return Err(CoreError::from_diagnostics(lex.diagnostics));
        }
        let parse = parser::parse_tokens(file_id, lex);
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
        let mut capabilities = SourceCapabilities::none();
        if self.configured_stdlib_source_path(canon) {
            for name in module_owner_aggregate_constructor_evidence(module) {
                capabilities.insert(SourceCapability::OwnerAggregateConstructorBoundary(name));
            }
            if module_has_owner_aggregate_field_evidence(module) {
                capabilities.insert(SourceCapability::OwnerAggregateFieldBoundary);
            }
            if module_has_raw_memory_boundary_evidence(module) {
                capabilities.insert(SourceCapability::RawMemoryStructuralBoundary);
            }
            for operation in module_raw_memory_operation_evidence(module) {
                capabilities.insert(SourceCapability::RawMemoryOperationBoundary(operation));
            }
            for operation in module_raw_body_memory_operation_evidence(module) {
                capabilities.insert(SourceCapability::RawBodyMemoryOperationBoundary(operation));
            }
            for memory_type in module_compiler_memory_type_definitions(module) {
                capabilities.insert(SourceCapability::CompilerMemoryTypeDefinition(memory_type));
            }
        }
        capabilities
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
    match path.canonicalize() {
        Ok(p) => normalize_path_lexically(&p),
        Err(_) => normalize_path_lexically(path),
    }
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

fn path_to_source_label(path: &PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

fn import_not_seen(imported_once: &mut BTreeSet<PathBuf>, target: &PathBuf) -> bool {
    imported_once.insert(canonicalize_path(target))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::effects::RawMemoryOp;

    fn test_loader() -> Loader {
        Loader::new(PathBuf::from("C:/nepl-test/stdlib"))
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
        let _ = loader
            .load_from_contents(
                path,
                String::from(src),
                &mut sm,
                &mut cache,
                &mut processing,
                &mut imported_once,
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
    fn owner_aggregate_boundaries_require_matching_source_evidence() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "vec", "future_safe.nepl"],
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
            "fn helper <(OwnerBox<i32>)->i32> (v):\n    field::get v \"owner\"\n",
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
            !capabilities.allows_owner_aggregate_constructor_boundary("RegionToken"),
            "intrinsic field evidence must not grant owner aggregate constructor capability"
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
    fn owner_aggregate_boundary_accepts_field_initializer_call_head() {
        let loader = test_loader();
        let path = canonicalize_path(&stdlib_path(
            &loader.stdlib_root,
            &["alloc", "collections", "owner_box", "field_init.nepl"],
        ));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            concat!(
                "fn helper <(OwnerBox<i32>)->i32> (v):\n",
                "    let owner <i32> field::get v \"owner\";\n",
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
        let loader = test_loader();
        let path = canonicalize_path(&path_from_segments("C:/nepl-test/user", &["vec_like.nepl"]));
        let capabilities = load_source_capabilities(
            &loader,
            path,
            "fn helper <(OwnerBox<i32>)->i32> (v):\n    field::get v \"owner\"\n",
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
                "    let cur <i32> load_i32 0\n",
                "    add cur size\n",
            ),
        );
        assert!(
            capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Alloc),
            "raw helper implementation with raw body evidence must grant its operation capability"
        );
        assert!(
            capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Load),
            "raw helper implementation keeps exact operation evidence from its body"
        );
        assert!(!capabilities.allows_raw_memory_operation_boundary(RawMemoryOp::Store));
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
            capabilities.allows_raw_memory_structural_boundary(),
            "compiler-owned raw address identity helpers are raw boundary evidence"
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
            capabilities.allows_raw_memory_structural_boundary(),
            "compiler-owned raw address intrinsics are raw boundary evidence"
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
}
