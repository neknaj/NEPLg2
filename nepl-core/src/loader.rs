use crate::ast::{Directive, Module, Stmt};
use crate::diagnostic::Severity;
use crate::error::CoreError;
use crate::lexer;
use crate::parser;
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

const RAW_MEMORY_BOUNDARY_STDLIB_PATHS: &[&[&str]] = &[
    &["core", "mem.nepl"],
    &["alloc", "io.nepl"],
    &["alloc", "string.nepl"],
    &["alloc", "string", "access.nepl"],
    &["alloc", "string", "integer.nepl"],
    &["alloc", "string", "scanner.nepl"],
    &["alloc", "string", "storage.nepl"],
    &["alloc", "string", "utf8.nepl"],
    &["std", "text.nepl"],
    &["std", "streamio", "scanner", "state.nepl"],
];

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
            self.source_capabilities_for_path(&canon),
        );
        let module = self.parse_module(file_id, src)?;
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
            self.source_capabilities_for_path(&canon),
        );
        let module = self.parse_module(file_id, src)?;
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
            self.source_capabilities_for_path(&canon),
        );
        loader_log!("[Loader] Parsing module: {:?}", canon);
        let module = self.parse_module(file_id, src)?;
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
            self.source_capabilities_for_path(&canon),
        );
        let module = self.parse_module(file_id, src)?;
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
            if imported_once.insert(target.clone()) {
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
                    if imported_once.insert(target.clone()) {
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
            if imported_once.insert(target.clone()) {
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
                    if imported_once.insert(target.clone()) {
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

    fn source_capabilities_for_path(&self, canon: &PathBuf) -> SourceCapabilities {
        if self.configured_raw_memory_boundary_path(canon) {
            SourceCapabilities::raw_memory_boundary()
        } else {
            SourceCapabilities::none()
        }
    }

    fn configured_raw_memory_boundary_path(&self, canon: &PathBuf) -> bool {
        RAW_MEMORY_BOUNDARY_STDLIB_PATHS
            .iter()
            .map(|segments| stdlib_path(&self.stdlib_root, segments))
            .any(|path| *canon == canonicalize_path(&path))
    }
}

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
