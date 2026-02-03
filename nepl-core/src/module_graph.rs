#![allow(dead_code)]
//! Module graph construction (no name resolution yet).
//!
//! - ModuleSpec: package + module path (string form)
//! - ModuleGraph: DAG of modules with parsed AST
//! - Resolves imports to filesystem paths (stdlib/deps/relative)
//!
//! Phase 2: cycle検出と依存グラフ構築のみ。名前解決や再エクスポートは未実装。

extern crate alloc;
extern crate std;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::ast::{Directive, ImportClause, Module, Visibility};
use crate::diagnostic::Severity;
use crate::error::CoreError;
use crate::lexer;
use crate::parser;
use crate::span::FileId;

#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

/// 識別用ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleId(pub u32);

/// パッケージ名とモジュールパスの論理表現
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleSpec {
    pub package: String,
    pub module: String,
}

#[derive(Debug, Clone)]
pub struct ModuleNode {
    pub id: ModuleId,
    pub spec: ModuleSpec,
    pub path: PathBuf,
    pub module: Module,
    pub imports: Vec<ImportDecl>,
    pub deps: Vec<DepDecl>,
}

#[derive(Debug, Clone)]
pub struct ModuleGraph {
    pub nodes: Vec<ModuleNode>,
    pub topo: Vec<ModuleId>,
}

#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub spec: ModuleSpec,
    pub clause: ImportClause,
    pub vis: Visibility,
}

#[derive(Debug, Clone)]
pub struct DepDecl {
    pub spec: ModuleSpec,
    pub id: ModuleId,
    pub clause: ImportClause,
    pub vis: Visibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Function,
    Struct,
    Enum,
}

#[derive(Debug, Clone)]
pub struct ExportEntry {
    pub kind: ExportKind,
    pub source: ModuleId,
}

#[derive(Debug, Clone)]
pub struct ExportTable {
    pub map: BTreeMap<ModuleId, BTreeMap<String, ExportEntry>>,
}

#[derive(Debug)]
pub enum ModuleGraphError {
    Io(String),
    Parse(Vec<crate::diagnostic::Diagnostic>),
    MissingDependency(String),
    Cycle(Vec<PathBuf>),
    InvalidImport(String),
    DuplicateExport(String),
}

impl From<CoreError> for ModuleGraphError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::Diagnostics(diags) => ModuleGraphError::Parse(diags),
            _ => ModuleGraphError::InvalidImport(format!("{:?}", e)),
        }
    }
}

#[derive(Debug)]
pub struct ModuleGraphBuilder {
    stdlib_root: PathBuf,
    deps: BTreeMap<String, PathBuf>,
}

impl ModuleGraphBuilder {
    pub fn new(stdlib_root: PathBuf) -> Self {
        let mut deps = BTreeMap::new();
        deps.insert("std".to_string(), stdlib_root.clone());
        Self { stdlib_root, deps }
    }

    /// 追加依存を登録（pkg -> path）
    pub fn with_dep(mut self, name: &str, path: PathBuf) -> Self {
        self.deps.insert(name.to_string(), path);
        self
    }

    /// entry ファイルからグラフを構築する。
    pub fn build(&self, entry: &PathBuf) -> Result<ModuleGraph, ModuleGraphError> {
        let mut cache: BTreeMap<PathBuf, ModuleId> = BTreeMap::new();
        let mut nodes: Vec<ModuleNode> = Vec::new();
        let mut topo: Vec<ModuleId> = Vec::new();
        let mut stack: Vec<PathBuf> = Vec::new();

        let entry_path = canonicalize_path(entry);
        self.load_recursive(
            &entry_path,
            "local",
            &mut cache,
            &mut nodes,
            &mut topo,
            &mut stack,
        )?;

        Ok(ModuleGraph { nodes, topo })
    }

    /// Exports を構築する（Phase3）。pub 定義 + pub import の再エクスポートを集計。
    pub fn build_exports(graph: &ModuleGraph) -> Result<ExportTable, ModuleGraphError> {
        let mut table: BTreeMap<ModuleId, BTreeMap<String, ExportEntry>> = BTreeMap::new();
        // topo は子→親の順に積まれているので、その順で処理すれば依存先の export が先に揃う。
        for id in &graph.topo {
            let node = graph
                .nodes
                .iter()
                .find(|n| n.id == *id)
                .expect("node not found");
            let mut exports = BTreeMap::new();

            // ローカル pub 定義
            for stmt in &node.module.root.items {
                match stmt {
                    crate::ast::Stmt::FnDef(f) if f.vis == crate::ast::Visibility::Pub => {
                        Self::insert_export(
                            &mut exports,
                            &f.name.name,
                            ExportKind::Function,
                            node.id,
                        )?;
                    }
                    crate::ast::Stmt::StructDef(s) if s.vis == crate::ast::Visibility::Pub => {
                        Self::insert_export(
                            &mut exports,
                            &s.name.name,
                            ExportKind::Struct,
                            node.id,
                        )?;
                    }
                    crate::ast::Stmt::EnumDef(e) if e.vis == crate::ast::Visibility::Pub => {
                        Self::insert_export(
                            &mut exports,
                            &e.name.name,
                            ExportKind::Enum,
                            node.id,
                        )?;
                    }
                    _ => {}
                }
            }

            // pub import の再エクスポート
            for dep in &node.deps {
                let dep_id = dep.id;
                let vis = dep.vis;
                if vis != crate::ast::Visibility::Pub {
                    continue;
                }
                if let Some(child_exports) = table.get(&dep_id) {
                    match &dep.clause {
                        ImportClause::Selective(items) => {
                            let mut export_all = false;
                            for item in items {
                                if item.glob {
                                    export_all = true;
                                    continue;
                                }
                                if let Some(entry) = child_exports.get(&item.name) {
                                    let out_name = item.alias.as_ref().unwrap_or(&item.name);
                                    Self::insert_export(
                                        &mut exports,
                                        out_name,
                                        entry.kind,
                                        entry.source,
                                    )?;
                                }
                            }
                            if export_all {
                                for (name, entry) in child_exports {
                                    Self::insert_export(
                                        &mut exports,
                                        name,
                                        entry.kind,
                                        entry.source,
                                    )?;
                                }
                            }
                        }
                        _ => {
                            for (name, entry) in child_exports {
                                Self::insert_export(&mut exports, name, entry.kind, entry.source)?;
                            }
                        }
                    }
                }
            }

            table.insert(node.id, exports);
        }
        Ok(ExportTable { map: table })
    }

    fn insert_export(
        map: &mut BTreeMap<String, ExportEntry>,
        name: &str,
        kind: ExportKind,
        source: ModuleId,
    ) -> Result<(), ModuleGraphError> {
        if map.contains_key(name) {
            return Err(ModuleGraphError::DuplicateExport(name.to_string()));
        }
        map.insert(
            name.to_string(),
            ExportEntry {
                kind,
                source,
            },
        );
        Ok(())
    }

    fn load_recursive(
        &self,
        path: &PathBuf,
        package: &str,
        cache: &mut BTreeMap<PathBuf, ModuleId>,
        nodes: &mut Vec<ModuleNode>,
        topo: &mut Vec<ModuleId>,
        stack: &mut Vec<PathBuf>,
    ) -> Result<ModuleId, ModuleGraphError> {
        if let Some(id) = cache.get(path) {
            return Ok(*id);
        }
        if stack.contains(path) {
            let mut cycle = stack.clone();
            cycle.push(path.clone());
            return Err(ModuleGraphError::Cycle(cycle));
        }

        stack.push(path.clone());
        let src = read_file_to_string(path)?;
        let file_id = FileId(cache.len() as u32);
        let lex = lexer::lex(file_id, &src);
        if lex.diagnostics.iter().any(|d| d.severity == Severity::Error) {
            return Err(ModuleGraphError::Parse(lex.diagnostics));
        }
        let parse = parser::parse_tokens(file_id, lex);
        let module = match parse.module {
            Some(m) => m,
            None => return Err(ModuleGraphError::Parse(parse.diagnostics)),
        };
        if parse
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
        {
            return Err(ModuleGraphError::Parse(parse.diagnostics));
        }

        // collect imports
        let mut import_specs = Vec::new();
        for d in &module.directives {
            if let Directive::Import { path: p, clause, vis, .. } = d {
                let spec = self.resolve_import(p, package, path)?;
                import_specs.push(ImportDecl {
                    spec,
                    clause: clause.clone(),
                    vis: *vis,
                });
            }
        }

        // Load children
        let mut deps = Vec::new();
        for import in &import_specs {
            let target_path = self.to_path(&import.spec, path)?;
            let cid = self.load_recursive(
                &target_path,
                &import.spec.package,
                cache,
                nodes,
                topo,
                stack,
            )?;
            deps.push(DepDecl {
                spec: import.spec.clone(),
                id: cid,
                clause: import.clause.clone(),
                vis: import.vis,
            });
        }

        stack.pop();
        let id = ModuleId(nodes.len() as u32);
        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let spec = ModuleSpec {
            package: package.to_string(),
            module: module_name,
        };
        nodes.push(ModuleNode {
            id,
            spec,
            path: path.clone(),
            module,
            imports: import_specs,
            deps,
        });
        topo.push(id);
        cache.insert(path.clone(), id);
        Ok(id)
    }

    fn resolve_import(
        &self,
        spec: &str,
        current_pkg: &str,
        current_path: &PathBuf,
    ) -> Result<ModuleSpec, ModuleGraphError> {
        if spec.starts_with("./") || spec.starts_with("../") || spec.starts_with('/') {
            // relative
            let pkg = current_pkg.to_string();
            Ok(ModuleSpec {
                package: pkg,
                module: spec.to_string(),
            })
        } else {
            let mut parts = spec.splitn(2, '/');
            let pkg = parts.next().unwrap_or("").to_string();
            let rest = parts.next().unwrap_or("");
            if pkg.is_empty() || rest.is_empty() {
                return Err(ModuleGraphError::InvalidImport(format!(
                    "invalid import path `{}` at {}",
                    spec,
                    current_path.display()
                )));
            }
            Ok(ModuleSpec {
                package: pkg,
                module: rest.to_string(),
            })
        }
    }

    fn to_path(&self, spec: &ModuleSpec, from: &PathBuf) -> Result<PathBuf, ModuleGraphError> {
        let mut base = if spec.package == "local" {
            match from.parent() {
                Some(p) => p.to_path_buf(),
                None => PathBuf::from("."),
            }
        } else if let Some(dep) = self.deps.get(&spec.package) {
            dep.clone()
        } else {
            return Err(ModuleGraphError::MissingDependency(spec.package.clone()));
        };
        base = base.join(&spec.module);
        let mut p = base.clone();
        if p.extension().is_none() {
            p = p.with_extension("nepl");
        }
        Ok(canonicalize_path(&p))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn read_file_to_string(path: &PathBuf) -> Result<String, ModuleGraphError> {
    fs::read_to_string(path).map_err(|e| ModuleGraphError::Io(e.to_string()))
}

#[cfg(target_arch = "wasm32")]
fn read_file_to_string(_path: &PathBuf) -> Result<String, ModuleGraphError> {
    Err(ModuleGraphError::Io(
        "filesystem access is not available on this target".into(),
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn canonicalize_path(path: &PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.clone())
}

#[cfg(target_arch = "wasm32")]
fn canonicalize_path(path: &PathBuf) -> PathBuf {
    path.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn builds_simple_graph_and_exports() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("main.nepl");
        let dep = dir.path().join("lib.nepl");
        fs::write(
            &root,
            "pub #import \"./lib\" as *\nfn main <()*> ()> ():\n    ()\n",
        )
        .unwrap();
        fs::write(&dep, "pub fn foo <()*> ()> ():\n    ()\n").unwrap();

        let builder = ModuleGraphBuilder::new(dir.path().to_path_buf());
        let g = builder.build(&root).unwrap();
        let exports = ModuleGraphBuilder::build_exports(&g).unwrap();
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(g.topo.len(), 2);
        // main should re-export foo
        let root_path = canonicalize_path(&root);
        let root_id = g.nodes.iter().find(|n| n.path == root_path).unwrap().id;
        let root_exports = exports.map.get(&root_id).unwrap();
        assert!(root_exports.contains_key("foo"));
    }

    #[test]
    fn pub_selective_reexport_respects_alias() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("main.nepl");
        let dep = dir.path().join("lib.nepl");
        fs::write(
            &root,
            "pub #import \"./lib\" as { foo as bar }\nfn main <()*> ()> ():\n    ()\n",
        )
        .unwrap();
        fs::write(
            &dep,
            "pub fn foo <()*> ()> ():\n    ()\npub fn baz <()*> ()> ():\n    ()\n",
        )
        .unwrap();

        let builder = ModuleGraphBuilder::new(dir.path().to_path_buf());
        let g = builder.build(&root).unwrap();
        let exports = ModuleGraphBuilder::build_exports(&g).unwrap();
        let root_path = canonicalize_path(&root);
        let root_id = g.nodes.iter().find(|n| n.path == root_path).unwrap().id;
        let root_exports = exports.map.get(&root_id).unwrap();
        assert!(root_exports.contains_key("bar"));
        assert!(!root_exports.contains_key("foo"));
        assert!(!root_exports.contains_key("baz"));
    }
}
