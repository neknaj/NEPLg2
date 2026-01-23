use crate::ast::{Directive, Module, Stmt};
use crate::diagnostic::Severity;
use crate::error::CoreError;
use crate::lexer;
use crate::parser;
use crate::span::FileId;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::result::Result;
use std::fs;
use std::path::PathBuf;
extern crate std;

/// Holds all loaded sources and their assigned FileId.
#[derive(Debug, Clone)]
pub struct SourceMap {
    files: Vec<(PathBuf, String)>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn path(&self, id: FileId) -> Option<&PathBuf> {
        self.files.get(id.0 as usize).map(|(p, _)| p)
    }

    /// Convert a byte offset to (line, column) 0-based.
    pub fn line_col(&self, id: FileId, byte: u32) -> Option<(usize, usize)> {
        let src = self.get(id)?;
        let mut line = 0;
        let mut col = 0;
        let mut count = 0;
        for ch in src.bytes() {
            if count as u32 == byte {
                return Some((line, col));
            }
            count += 1;
            if ch == b'\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        if count as u32 == byte {
            Some((line, col))
        } else {
            None
        }
    }

    pub fn line_str(&self, id: FileId, line: usize) -> Option<&str> {
        let src = self.get(id)?;
        src.lines().nth(line)
    }

    pub fn get(&self, id: FileId) -> Option<&str> {
        self.files.get(id.0 as usize).map(|(_, s)| s.as_str())
    }

    pub fn add(&mut self, path: PathBuf, src: String) -> FileId {
        let id = self.files.len() as u32;
        self.files.push((path, src));
        FileId(id)
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
}

impl Loader {
    pub fn new(stdlib_root: PathBuf) -> Self {
        Self { stdlib_root }
    }

    /// Load an already-provided source string as a pseudo file (for stdin use).
    pub fn load_inline(&self, path: PathBuf, src: String) -> Result<LoadResult, CoreError> {
        let mut sm = SourceMap::new();
        let mut cache: BTreeMap<PathBuf, Module> = BTreeMap::new();
        let mut processing: BTreeSet<PathBuf> = BTreeSet::new();
        let mut imported: BTreeSet<PathBuf> = BTreeSet::new();
        let module = self.load_from_contents(path, src, &mut sm, &mut cache, &mut processing, &mut imported)?;
        Ok(LoadResult { module, source_map: sm })
    }

    pub fn load(&self, entry: &PathBuf) -> Result<LoadResult, CoreError> {
        let mut sm = SourceMap::new();
        let mut cache: BTreeMap<PathBuf, Module> = BTreeMap::new();
        let mut processing: BTreeSet<PathBuf> = BTreeSet::new();
        let mut imported: BTreeSet<PathBuf> = BTreeSet::new();
        let module = self.load_file(entry, &mut sm, &mut cache, &mut processing, &mut imported)?;
        Ok(LoadResult { module, source_map: sm })
    }

    fn load_from_contents(
        &self,
        path: PathBuf,
        src: String,
        sm: &mut SourceMap,
        cache: &mut BTreeMap<PathBuf, Module>,
        processing: &mut BTreeSet<PathBuf>,
        imported_once: &mut BTreeSet<PathBuf>,
    ) -> Result<Module, CoreError> {
        // For pseudo files (stdin) canonicalize may fail; fall back to provided path.
        let canon = path
            .canonicalize()
            .unwrap_or(path.clone());
        if let Some(m) = cache.get(&canon) {
            return Ok(m.clone());
        }
        if !processing.insert(canon.clone()) {
            return Err(CoreError::Io(format!(
                "circular import/include detected at {:?}",
                canon
            )));
        }
        let file_id = sm.add(canon.clone(), src.clone());
        let module = self.parse_module(file_id, src)?;
        let module = self.process_directives(canon.clone(), module, sm, cache, processing, imported_once)?;
        processing.remove(&canon);
        cache.insert(canon.clone(), module.clone());
        Ok(module)
    }

    fn load_file(
        &self,
        path: &PathBuf,
        sm: &mut SourceMap,
        cache: &mut BTreeMap<PathBuf, Module>,
        processing: &mut BTreeSet<PathBuf>,
        imported_once: &mut BTreeSet<PathBuf>,
    ) -> Result<Module, CoreError> {
        let canon = path
            .canonicalize()
            .map_err(|e| CoreError::Io(e.to_string()))?;
        if let Some(m) = cache.get(&canon) {
            return Ok(m.clone());
        }
        if !processing.insert(canon.clone()) {
            return Err(CoreError::Io(format!(
                "circular import/include detected at {:?}",
                canon
            )));
        }
        let src = fs::read_to_string(&canon).map_err(|e| CoreError::Io(e.to_string()))?;
        let file_id = sm.add(canon.clone(), src.clone());
        let module = self.parse_module(file_id, src)?;
        let module = self.process_directives(canon.clone(), module, sm, cache, processing, imported_once)?;
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
    ) -> Result<Module, CoreError> {
        let mut directives = module.directives.clone();
        let mut items = Vec::new();
        for stmt in module.root.items.clone() {
            match &stmt {
                Stmt::Directive(Directive::Import { path, .. }) => {
                    let target = self.resolve_path(&base, path);
                    if imported_once.insert(target.clone()) {
                        let imp_mod =
                            self.load_file(&target, sm, cache, processing, imported_once)?;
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
                    let inc_mod = self.load_file(&target, sm, cache, processing, imported_once)?;
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
        if parse.module.is_none() {
            return Err(CoreError::from_diagnostics(parse.diagnostics));
        }
        Ok(parse.module.unwrap())
    }

    fn resolve_path(&self, base: &PathBuf, spec: &str) -> PathBuf {
        let mut p = if spec.starts_with("std/") {
            self.stdlib_root.join(spec)
        } else {
            base.parent()
                .map(|p| p.join(spec))
                .unwrap_or_else(|| PathBuf::from(spec))
        };
        if p.extension().is_none() {
            p = p.with_extension("nepl");
        }
        p
    }
}
