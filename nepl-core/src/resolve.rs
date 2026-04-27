//! Name and import resolution support.
//!
//! `ImportResolution` is used by the main typecheck pipeline to enforce
//! `#import` visibility on the current flat loader representation.  The
//! ModuleGraph/DefId APIs below remain host-side utilities for module-scope
//! export resolution and the NEPLg3 split pipeline.

extern crate alloc;

use alloc::collections::btree_map::Entry;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::{Directive, ImportClause};
use crate::source_map::SourceMap;

#[cfg(not(target_os = "none"))]
use crate::ast::ImportItem;
#[cfg(not(target_os = "none"))]
use crate::ast::{EnumDef, FnAlias, FnDef, StructDef, Visibility};
#[cfg(not(target_os = "none"))]
use crate::diagnostic::Diagnostic;
#[cfg(not(target_os = "none"))]
use crate::diagnostic_ids::DiagnosticId;
#[cfg(not(target_os = "none"))]
use crate::module_graph::{ExportTable, ModuleGraph, ModuleId};

pub type QualifiedImportTargets = BTreeMap<u32, BTreeMap<String, BTreeSet<u32>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnqualifiedImportVisibility {
    Hidden,
    All,
    Selected(BTreeMap<String, String>),
}

pub type UnqualifiedImportVisibilityMap = BTreeMap<u32, BTreeMap<u32, UnqualifiedImportVisibility>>;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportResolution {
    qualified_targets: QualifiedImportTargets,
    unqualified_visibility: UnqualifiedImportVisibilityMap,
}

impl ImportResolution {
    pub fn from_module(
        module: &crate::ast::Module,
        source_map: Option<&SourceMap>,
    ) -> ImportResolution {
        ImportResolution {
            qualified_targets: build_qualified_import_targets(module, source_map),
            unqualified_visibility: build_unqualified_import_visibility(module, source_map),
        }
    }

    pub fn qualified_targets_for_alias(
        &self,
        source_file: u32,
        alias: &str,
    ) -> Option<&BTreeSet<u32>> {
        self.qualified_targets
            .get(&source_file)
            .and_then(|aliases| aliases.get(alias))
    }

    pub fn has_qualified_targets(&self) -> bool {
        !self.qualified_targets.is_empty()
    }

    pub fn unqualified_lookup_names(&self, source_file: u32, name: &str) -> Vec<String> {
        let mut names = Vec::new();
        names.push(name.to_string());
        if let Some(imports) = self.unqualified_visibility.get(&source_file) {
            for visibility in imports.values() {
                if let UnqualifiedImportVisibility::Selected(selected) = visibility {
                    if let Some(source_name) = selected.get(name) {
                        if !names.iter().any(|existing| existing == source_name) {
                            names.push(source_name.clone());
                        }
                    }
                }
            }
        }
        names
    }

    pub fn binding_is_visible_unqualified(
        &self,
        source_file: u32,
        name: &str,
        binding_file: u32,
        binding_name: &str,
    ) -> bool {
        if binding_file == source_file {
            return binding_name == name;
        }
        let Some(imports) = self.unqualified_visibility.get(&source_file) else {
            return true;
        };
        let Some(visibility) = imports.get(&binding_file) else {
            return false;
        };
        match visibility {
            UnqualifiedImportVisibility::Hidden => false,
            UnqualifiedImportVisibility::All => binding_name == name,
            UnqualifiedImportVisibility::Selected(selected) => selected
                .get(name)
                .map(|source_name| source_name == binding_name)
                .unwrap_or(false),
        }
    }
}

fn normalized_import_suffix(module: &str, ext: &str) -> String {
    let normalized = module.replace('\\', "/");
    let mut parts = Vec::new();
    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    let mut s = parts.join("/");
    if !s.starts_with('/') {
        s.insert(0, '/');
    }
    s.push_str(ext);
    s
}

fn import_target_files(source_map: &SourceMap, path: &str) -> BTreeSet<u32> {
    let suffixes = [
        normalized_import_suffix(path, ".nepl"),
        normalized_import_suffix(path, ".n.md"),
    ];
    source_map
        .iter_paths()
        .filter_map(|(file_id, path)| {
            let mut normalized = path.to_string_lossy().replace('\\', "/");
            if !normalized.starts_with('/') {
                normalized.insert(0, '/');
            }
            if suffixes.iter().any(|suffix| normalized.ends_with(suffix)) {
                Some(file_id.0)
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>()
}

fn import_clause_unqualified_visibility(clause: &ImportClause) -> UnqualifiedImportVisibility {
    match clause {
        ImportClause::DefaultAlias | ImportClause::Alias(_) => UnqualifiedImportVisibility::Hidden,
        ImportClause::Open | ImportClause::Merge => UnqualifiedImportVisibility::All,
        ImportClause::Selective(items) => {
            if items.iter().any(|item| item.glob) {
                return UnqualifiedImportVisibility::All;
            }
            let mut selected = BTreeMap::new();
            for item in items {
                selected.insert(
                    item.alias.clone().unwrap_or_else(|| item.name.clone()),
                    item.name.clone(),
                );
            }
            UnqualifiedImportVisibility::Selected(selected)
        }
    }
}

fn default_import_alias(path: &str) -> Option<String> {
    let file_name = path
        .rsplit(|ch| ch == '/' || ch == '\\')
        .next()
        .unwrap_or(path)
        .trim();
    if file_name.is_empty() {
        return None;
    }
    let stem = file_name
        .strip_suffix(".nepl")
        .or_else(|| file_name.strip_suffix(".n.md"))
        .unwrap_or(file_name);
    if stem.is_empty() {
        None
    } else {
        Some(stem.to_string())
    }
}

fn merge_unqualified_import_visibility(
    current: &mut UnqualifiedImportVisibility,
    next: UnqualifiedImportVisibility,
) -> bool {
    let before = current.clone();
    match next {
        UnqualifiedImportVisibility::Hidden => {}
        UnqualifiedImportVisibility::All => {
            if !matches!(&*current, UnqualifiedImportVisibility::All) {
                *current = UnqualifiedImportVisibility::All;
            }
        }
        UnqualifiedImportVisibility::Selected(next) => match current {
            UnqualifiedImportVisibility::Hidden => {
                *current = UnqualifiedImportVisibility::Selected(next);
            }
            UnqualifiedImportVisibility::Selected(current) => {
                current.extend(next);
            }
            UnqualifiedImportVisibility::All => {}
        },
    }
    *current != before
}

fn insert_unqualified_import_visibility(
    out: &mut UnqualifiedImportVisibilityMap,
    source_file: u32,
    target_files: BTreeSet<u32>,
    visibility: UnqualifiedImportVisibility,
) {
    if target_files.is_empty() {
        return;
    }
    let source_visibility = out.entry(source_file).or_insert_with(BTreeMap::new);
    for target_file in target_files {
        source_visibility
            .entry(target_file)
            .and_modify(|current| {
                let _ = merge_unqualified_import_visibility(current, visibility.clone());
            })
            .or_insert_with(|| visibility.clone());
    }
}

fn expand_unqualified_import_visibility(out: &mut UnqualifiedImportVisibilityMap) {
    let mut worklist = Vec::new();
    for (source_file, targets) in out.iter() {
        for (target_file, visibility) in targets {
            if *visibility == UnqualifiedImportVisibility::All {
                worklist.push((*source_file, *target_file));
            }
        }
    }
    while let Some((source_file, middle_file)) = worklist.pop() {
        let middle_targets = out
            .get(&middle_file)
            .map(|targets| {
                targets
                    .iter()
                    .map(|(target_file, visibility)| (*target_file, visibility.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for (target_file, next_visibility) in middle_targets {
            let source_visibility = out.entry(source_file).or_insert_with(BTreeMap::new);
            let changed = match source_visibility.entry(target_file) {
                Entry::Occupied(mut entry) => {
                    merge_unqualified_import_visibility(entry.get_mut(), next_visibility)
                }
                Entry::Vacant(entry) => {
                    entry.insert(next_visibility);
                    true
                }
            };
            if changed
                && source_visibility
                    .get(&target_file)
                    .is_some_and(|visibility| *visibility == UnqualifiedImportVisibility::All)
            {
                worklist.push((source_file, target_file));
            }
        }
    }
}

pub fn build_unqualified_import_visibility(
    module: &crate::ast::Module,
    source_map: Option<&SourceMap>,
) -> UnqualifiedImportVisibilityMap {
    let Some(source_map) = source_map else {
        return BTreeMap::new();
    };
    let mut directives: Vec<&Directive> = module.directives.iter().collect();
    for item in &module.root.items {
        if let crate::ast::Stmt::Directive(d) = item {
            directives.push(d);
        }
    }
    let mut out = BTreeMap::new();
    for (file_id, _) in source_map.iter_paths() {
        out.entry(file_id.0).or_insert_with(BTreeMap::new);
    }
    let root_file = source_map.iter_paths().next().map(|(file_id, _)| file_id.0);
    let mut root_has_no_prelude = false;
    let mut root_has_explicit_prelude = false;
    for directive in directives {
        match directive {
            Directive::Import {
                path, clause, span, ..
            } => {
                insert_unqualified_import_visibility(
                    &mut out,
                    span.file_id.0,
                    import_target_files(source_map, path),
                    import_clause_unqualified_visibility(clause),
                );
            }
            Directive::Include { path, span } => {
                insert_unqualified_import_visibility(
                    &mut out,
                    span.file_id.0,
                    import_target_files(source_map, path),
                    UnqualifiedImportVisibility::All,
                );
            }
            Directive::Prelude { path, span } => {
                if Some(span.file_id.0) == root_file {
                    root_has_explicit_prelude = true;
                }
                insert_unqualified_import_visibility(
                    &mut out,
                    span.file_id.0,
                    import_target_files(source_map, path),
                    UnqualifiedImportVisibility::All,
                );
            }
            Directive::NoPrelude { span } => {
                if Some(span.file_id.0) == root_file {
                    root_has_no_prelude = true;
                }
            }
            _ => {}
        }
    }
    if let Some(root_file) = root_file {
        if !root_has_no_prelude && !root_has_explicit_prelude {
            insert_unqualified_import_visibility(
                &mut out,
                root_file,
                import_target_files(source_map, "std/prelude_base"),
                UnqualifiedImportVisibility::All,
            );
        }
    }
    expand_unqualified_import_visibility(&mut out);
    out
}

pub fn build_qualified_import_targets(
    module: &crate::ast::Module,
    source_map: Option<&SourceMap>,
) -> QualifiedImportTargets {
    let Some(source_map) = source_map else {
        return BTreeMap::new();
    };
    let mut directives: Vec<&Directive> = module.directives.iter().collect();
    for item in &module.root.items {
        if let crate::ast::Stmt::Directive(d) = item {
            directives.push(d);
        }
    }
    let merge_import_targets = build_merge_import_targets(&directives, source_map);
    let mut out: QualifiedImportTargets = BTreeMap::new();
    for directive in directives {
        let Directive::Import {
            path, clause, span, ..
        } = directive
        else {
            continue;
        };
        let aliases: Vec<String> = match clause {
            ImportClause::DefaultAlias => default_import_alias(path)
                .map(|alias| vec![alias])
                .unwrap_or_default(),
            ImportClause::Alias(alias) => vec![alias.clone()],
            _ => Vec::new(),
        };
        if aliases.is_empty() {
            continue;
        }
        let target_files = expand_files_through_merge_imports(
            import_target_files(source_map, path),
            &merge_import_targets,
        );
        if target_files.is_empty() {
            continue;
        }
        let file_aliases = out.entry(span.file_id.0).or_default();
        for alias in aliases {
            file_aliases
                .entry(alias)
                .or_default()
                .extend(target_files.iter().copied());
        }
    }
    out
}

fn build_merge_import_targets(
    directives: &[&Directive],
    source_map: &SourceMap,
) -> BTreeMap<u32, BTreeSet<u32>> {
    let mut out: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    for directive in directives {
        let Directive::Import {
            path, clause, span, ..
        } = directive
        else {
            continue;
        };
        if !matches!(clause, ImportClause::Merge) {
            continue;
        }
        let target_files = import_target_files(source_map, path);
        if target_files.is_empty() {
            continue;
        }
        out.entry(span.file_id.0).or_default().extend(target_files);
    }
    out
}

fn expand_files_through_merge_imports(
    target_files: BTreeSet<u32>,
    merge_import_targets: &BTreeMap<u32, BTreeSet<u32>>,
) -> BTreeSet<u32> {
    let mut expanded = target_files;
    let mut stack = expanded.iter().copied().collect::<Vec<_>>();
    while let Some(file_id) = stack.pop() {
        let Some(merged_files) = merge_import_targets.get(&file_id) else {
            continue;
        };
        for merged_file in merged_files {
            if expanded.insert(*merged_file) {
                stack.push(*merged_file);
            }
        }
    }
    expanded
}

#[cfg(not(target_os = "none"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DefId(pub u32);

#[cfg(not(target_os = "none"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefKind {
    Function,
    Struct,
    Enum,
}

#[cfg(not(target_os = "none"))]
#[derive(Debug, Clone)]
pub struct DefInfo {
    pub id: DefId,
    pub kind: DefKind,
    pub module: ModuleId,
}

#[cfg(not(target_os = "none"))]
#[derive(Debug, Clone)]
pub struct DefTable {
    pub defs: BTreeMap<ModuleId, BTreeMap<String, DefInfo>>,
}

#[cfg(not(target_os = "none"))]
impl DefTable {
    pub fn new() -> Self {
        Self {
            defs: BTreeMap::new(),
        }
    }
}

/// Collect local public definitions and assign DefId.
#[cfg(not(target_os = "none"))]
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
#[cfg(not(target_os = "none"))]
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
#[cfg(not(target_os = "none"))]
#[derive(Debug, Clone)]
pub struct ImportScope {
    pub alias_map: BTreeMap<String, ModuleId>, // alias -> module
    pub open_modules: Vec<ModuleId>,           // as *
    pub selective: BTreeMap<String, DefInfo>,  // name -> def (selected)
}

#[cfg(not(target_os = "none"))]
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub id: ModuleId,
    pub exports: BTreeMap<String, DefInfo>,
    pub imports: ImportScope,
}

#[cfg(not(target_os = "none"))]
#[derive(Debug, Clone)]
pub struct ResolvedGraph {
    pub modules: BTreeMap<ModuleId, ResolvedModule>,
}

/// Build per-module import scopes using ExportTable results.
#[cfg(not(target_os = "none"))]
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

#[cfg(not(target_os = "none"))]
fn last_segment(path: &str) -> &str {
    path.rsplit(&['/', '\\'][..]).next().unwrap_or(path)
}

/// 可視シンボル表を生成し、曖昧な open import を検出する。
/// 優先順位: ローカル(pub)定義 > selective import > open import（最初の衝突で診断）。
#[cfg(not(target_os = "none"))]
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
