//! Name resolution scaffolding (Phase4: DefId 付与とエクスポート表からの引き当て).
//! まだ式中識別子の解決や HIR への反映は行わず、モジュール単位の定義テーブルを構築する。

#![allow(dead_code)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;

use crate::ast::{EnumDef, FnAlias, FnDef, StructDef, Visibility};
use crate::ast::{ImportClause, ImportItem};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::module_graph::{ExportTable, ModuleGraph, ModuleId};
use alloc::collections::BTreeSet;
use alloc::string::ToString;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DefId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefKind {
    Function,
    Struct,
    Enum,
}

#[derive(Debug, Clone)]
pub struct DefInfo {
    pub id: DefId,
    pub kind: DefKind,
    pub module: ModuleId,
}

#[derive(Debug, Clone)]
pub struct DefTable {
    pub defs: BTreeMap<ModuleId, BTreeMap<String, DefInfo>>,
}

impl DefTable {
    pub fn new() -> Self {
        Self {
            defs: BTreeMap::new(),
        }
    }
}

/// Collect local public definitions and assign DefId.
pub fn collect_defs(graph: &ModuleGraph) -> DefTable {
    let mut table = DefTable::new();
    let mut next_id: u32 = 0;
    for node in &graph.nodes {
        let mut map = BTreeMap::new();
        for stmt in &node.module.root.items {
            match stmt {
                crate::ast::Stmt::FnDef(FnDef { name, vis, .. }) if *vis == Visibility::Pub => {
                    let id = DefId(next_id);
                    next_id += 1;
                    map.insert(
                        name.name.clone(),
                        DefInfo {
                            id,
                            kind: DefKind::Function,
                            module: node.id,
                        },
                    );
                }
                crate::ast::Stmt::FnAlias(FnAlias { name, vis, .. }) if *vis == Visibility::Pub => {
                    let id = DefId(next_id);
                    next_id += 1;
                    map.insert(
                        name.name.clone(),
                        DefInfo {
                            id,
                            kind: DefKind::Function,
                            module: node.id,
                        },
                    );
                }
                crate::ast::Stmt::StructDef(StructDef { name, vis, .. })
                    if *vis == Visibility::Pub =>
                {
                    let id = DefId(next_id);
                    next_id += 1;
                    map.insert(
                        name.name.clone(),
                        DefInfo {
                            id,
                            kind: DefKind::Struct,
                            module: node.id,
                        },
                    );
                }
                crate::ast::Stmt::EnumDef(EnumDef { name, vis, .. }) if *vis == Visibility::Pub => {
                    let id = DefId(next_id);
                    next_id += 1;
                    map.insert(
                        name.name.clone(),
                        DefInfo {
                            id,
                            kind: DefKind::Enum,
                            module: node.id,
                        },
                    );
                }
                _ => {}
            }
        }
        table.defs.insert(node.id, map);
    }
    table
}

/// Compose DefTable with ExportTable to know which module exports which DefId.
pub fn compose_exports(
    defs: &DefTable,
    exports: &ExportTable,
) -> BTreeMap<ModuleId, BTreeMap<String, DefInfo>> {
    let mut out = BTreeMap::new();
    for (mid, map) in &exports.map {
        let mut exp_map = BTreeMap::new();
        for (name, entry) in map {
            if let Some(src_defs) = defs.defs.get(&entry.source) {
                if let Some(info) = src_defs.get(name) {
                    exp_map.insert(name.clone(), info.clone());
                }
            }
        }
        out.insert(*mid, exp_map);
    }
    out
}

/// 輸入スコープの展開結果
#[derive(Debug, Clone)]
pub struct ImportScope {
    pub alias_map: BTreeMap<String, ModuleId>, // alias -> module
    pub open_modules: Vec<ModuleId>,           // as *
    pub selective: BTreeMap<String, DefInfo>,  // name -> def (selected)
}

#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub id: ModuleId,
    pub exports: BTreeMap<String, DefInfo>,
    pub imports: ImportScope,
}

#[derive(Debug, Clone)]
pub struct ResolvedGraph {
    pub modules: BTreeMap<ModuleId, ResolvedModule>,
}

/// Build per-module import scopes using ExportTable results.
pub fn resolve_imports(
    graph: &ModuleGraph,
    exports: &BTreeMap<ModuleId, BTreeMap<String, DefInfo>>,
) -> ResolvedGraph {
    let mut modules = BTreeMap::new();
    for node in &graph.nodes {
        let mut alias_map = BTreeMap::new();
        let mut open_modules = Vec::new();
        let mut selective = BTreeMap::new();
        for dep in &node.deps {
            let dep_exports = exports.get(&dep.id);
            let default_alias = last_segment(&dep.spec.module);
            match &dep.clause {
                ImportClause::DefaultAlias => {
                    alias_map.insert(default_alias.to_string(), dep.id);
                }
                ImportClause::Alias(a) => {
                    alias_map.insert(a.clone(), dep.id);
                }
                ImportClause::Open => {
                    open_modules.push(dep.id);
                }
                ImportClause::Selective(list) => {
                    if let Some(e) = dep_exports {
                        for ImportItem { name, alias, glob } in list {
                            if *glob {
                                // name::* : open that namespace
                                open_modules.push(dep.id);
                                continue;
                            }
                            if let Some(def) = e.get(name) {
                                selective
                                    .insert(alias.clone().unwrap_or(name.clone()), def.clone());
                            }
                        }
                    }
                }
                ImportClause::Merge => {
                    // merge は open と同等に扱い、解決時に同一モジュール扱いにする
                    open_modules.push(dep.id);
                }
            }
        }
        let exports_map = exports.get(&node.id).cloned().unwrap_or_else(BTreeMap::new);
        modules.insert(
            node.id,
            ResolvedModule {
                id: node.id,
                exports: exports_map,
                imports: ImportScope {
                    alias_map,
                    open_modules,
                    selective,
                },
            },
        );
    }
    ResolvedGraph { modules }
}

fn last_segment(path: &str) -> &str {
    path.rsplit(&['/', '\\'][..]).next().unwrap_or(path)
}

/// 可視シンボル表を生成し、曖昧な open import を検出する。
/// 優先順位: ローカル(pub)定義 > selective import > open import（最初の衝突で診断）。
pub fn build_visible_map(
    defs: &DefTable,
    resolved: &ResolvedGraph,
) -> (
    BTreeMap<ModuleId, BTreeMap<String, DefInfo>>,
    Vec<Diagnostic>,
) {
    let mut out = BTreeMap::new();
    let mut diags = Vec::new();
    for (mid, rm) in &resolved.modules {
        let mut map = BTreeMap::new();
        // 1) ローカル公開定義
        if let Some(local) = defs.defs.get(mid) {
            for (n, info) in local {
                map.insert(n.clone(), info.clone());
            }
        }
        // 2) selective import
        for (n, info) in &rm.imports.selective {
            map.entry(n.clone()).or_insert_with(|| info.clone());
        }
        // 3) open import: 曖昧なら診断
        let mut seen_open: BTreeMap<String, DefInfo> = BTreeMap::new();
        let mut ambiguous: BTreeSet<String> = BTreeSet::new();
        for dep in &rm.imports.open_modules {
            if let Some(src) = resolved.modules.get(dep) {
                for (n, info) in &src.exports {
                    if seen_open.get(n).is_some() {
                        ambiguous.insert(n.clone());
                        // 既に map に入っていても診断のみ。優先順位は最初のもの。
                    } else {
                        seen_open.insert(n.clone(), info.clone());
                    }
                }
            }
        }
        for name in ambiguous {
            diags.push(
                Diagnostic::error(
                    alloc::format!(
                        "ambiguous import: `{}` is provided by multiple open imports",
                        name
                    ),
                    crate::span::Span::dummy(),
                )
                .with_id(DiagnosticId::AmbiguousImport),
            );
        }
        for (n, info) in seen_open {
            map.entry(n.clone()).or_insert(info);
        }
        out.insert(*mid, map);
    }
    (out, diags)
}
