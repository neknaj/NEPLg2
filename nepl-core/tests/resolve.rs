use nepl_core::ast::{Directive, ImportClause};
use nepl_core::diagnostic::Severity;
use nepl_core::diagnostic_codes::{DiagnosticCode, ResolveDiagnosticCode};
use nepl_core::hir::{FuncRef, HirBody, HirExprKind, HirModule};
use nepl_core::lexer;
use nepl_core::loader::{Loader, LoaderError, SourceMap};
use nepl_core::module_graph::ModuleGraphBuilder;
use nepl_core::parser;
use nepl_core::resolve::{
    build_visible_map, collect_defs, compose_exports, resolve_imports, DefId, ImportResolution,
};
use nepl_core::span::{FileId, Span};
use nepl_core::{BuildProfile, CompileTarget};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn canonicalize_path(path: &PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.clone())
}

fn load_virtual_sources(
    main: &str,
    sources: &[(&str, &str)],
) -> (nepl_core::ast::Module, SourceMap) {
    let mut loader = Loader::new(PathBuf::from("virtual_std"));
    let mut provider = |path: &PathBuf| match path.file_name().and_then(|name| name.to_str()) {
        Some(file_name) => sources
            .iter()
            .find_map(|(name, source)| {
                if *name == file_name {
                    Some((*source).to_string())
                } else {
                    None
                }
            })
            .ok_or_else(|| LoaderError::Io(format!("missing virtual source: {:?}", path))),
        _ => Err(LoaderError::Io(format!(
            "missing virtual source: {:?}",
            path
        ))),
    };
    let loaded = loader
        .load_inline_with_provider(PathBuf::from("main.nepl"), main.to_string(), &mut provider)
        .expect("load virtual sources");
    (loaded.module, loaded.source_map)
}

fn source_file_id(source_map: &SourceMap, suffix: &str) -> u32 {
    source_map
        .iter_paths()
        .find_map(|(file_id, path)| {
            let normalized = path.to_string_lossy().replace('\\', "/");
            if normalized.ends_with(suffix) {
                Some(file_id.0)
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("source file not found: {}", suffix))
}

fn def_id_for_name(source_map: &SourceMap, file_id: u32, name: &str) -> DefId {
    let source = source_map.get(FileId(file_id)).expect("source text");
    let start = source
        .find(name)
        .unwrap_or_else(|| panic!("definition name not found: {}", name)) as u32;
    DefId::from_span(Span::new(FileId(file_id), start, start + name.len() as u32))
        .expect("source definition span")
}

fn typecheck_virtual(main: &str, sources: &[(&str, &str)]) -> (HirModule, SourceMap) {
    let (module, source_map) = load_virtual_sources(main, sources);
    let result = nepl_core::typecheck::typecheck(
        &module,
        CompileTarget::Wasm,
        BuildProfile::Debug,
        Some(&source_map),
    );
    assert!(
        result
            .diagnostics
            .iter()
            .all(|d| d.severity != Severity::Error),
        "unexpected typecheck errors: {:?}",
        result.diagnostics
    );
    (result.module.expect("hir module"), source_map)
}

fn entry_call_def_id(hir: &HirModule) -> Option<DefId> {
    let entry = hir.entry.as_deref().expect("resolved entry");
    let function = hir
        .functions
        .iter()
        .find(|f| f.name == entry)
        .unwrap_or_else(|| panic!("entry function not found: {}", entry));
    let HirBody::Block(block) = &function.body else {
        panic!("entry body is not a block")
    };
    let HirExprKind::Call { callee, .. } = &block.lines[0].expr.kind else {
        panic!("entry expression is not a call")
    };
    let FuncRef::User(_, _, def_id) = callee else {
        panic!("entry callee is not a user function")
    };
    *def_id
}

fn first_call_def_id_for_file(hir: &HirModule, file_id: u32) -> Option<DefId> {
    let function = hir
        .functions
        .iter()
        .find(|f| f.span.file_id.0 == file_id)
        .unwrap_or_else(|| panic!("function for file id {} not found", file_id));
    let HirBody::Block(block) = &function.body else {
        panic!("function body is not a block")
    };
    let HirExprKind::Call { callee, .. } = &block.lines[0].expr.kind else {
        panic!("function first expression is not a call")
    };
    let FuncRef::User(_, _, def_id) = callee else {
        panic!("callee is not a user function")
    };
    *def_id
}

#[test]
fn parse_prelude_directives() {
    let src = r#"
#prelude std/prelude_base
#no_prelude
#entry main
fn main <() -> i32> ():
    0
"#;
    let lex = lexer::lex(FileId(0), src);
    assert!(
        lex.diagnostics
            .iter()
            .all(|d| d.severity != Severity::Error),
        "unexpected lexer errors: {:?}",
        lex.diagnostics
    );
    let parse = parser::parse_tokens(FileId(0), lex);
    let module = parse.module.expect("module");
    assert!(
        parse
            .diagnostics
            .iter()
            .all(|d| d.severity != Severity::Error),
        "unexpected parser errors: {:?}",
        parse.diagnostics
    );
    let mut saw_prelude = false;
    let mut saw_no_prelude = false;
    for d in &module.directives {
        match d {
            Directive::Prelude { path, .. } => {
                assert_eq!(path, "std/prelude_base");
                saw_prelude = true;
            }
            Directive::NoPrelude { .. } => {
                saw_no_prelude = true;
            }
            _ => {}
        }
    }
    assert!(saw_prelude, "expected #prelude to be recorded");
    assert!(saw_no_prelude, "expected #no_prelude to be recorded");
}

#[test]
fn import_clause_merge_is_preserved() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("main.nepl");
    let part = dir.path().join("part.nepl");
    fs::write(
        &root,
        "#import \"./part\" as @merge\n#entry main\nfn main <() -> ()> ():\n    ()\n",
    )
    .unwrap();
    fs::write(&part, "fn helper <() -> ()> ():\n    ()\n").unwrap();

    let builder = ModuleGraphBuilder::new(dir.path().to_path_buf());
    let g = builder.build(&root).unwrap();
    let root_path = canonicalize_path(&root);
    let root_id = g.nodes.iter().find(|n| n.path == root_path).unwrap().id;
    let node = g.nodes.iter().find(|n| n.id == root_id).unwrap();
    assert_eq!(node.imports.len(), 1);
    assert!(matches!(node.imports[0].clause, ImportClause::Merge));
}

#[test]
fn import_resolution_filters_alias_selective_and_open_visibility() {
    const DEP: &str = r#"
#indent 4
#no_prelude

fn allowed <()->i32> ():
    41

fn hidden <()->i32> ():
    7
"#;
    let main = r#"
#entry main
#indent 4
#no_prelude

#import "dep" as dep
#import "dep" as { allowed as renamed }

fn main <()->i32> ():
    renamed
"#;
    let (module, source_map) = load_virtual_sources(main, &[("dep.nepl", DEP)]);
    let resolution = ImportResolution::from_module(&module, Some(&source_map));
    let main_file = source_file_id(&source_map, "main.nepl");
    let dep_file = source_file_id(&source_map, "dep.nepl");

    let alias_targets = resolution
        .qualified_targets_for_alias(main_file, "dep")
        .expect("default alias target");
    assert!(alias_targets.contains(&dep_file));
    assert_eq!(
        resolution.unqualified_lookup_names(main_file, "renamed"),
        vec![String::from("renamed"), String::from("allowed")]
    );
    assert!(resolution.binding_is_visible_unqualified(main_file, "renamed", dep_file, "allowed"));
    assert!(!resolution.binding_is_visible_unqualified(main_file, "allowed", dep_file, "allowed"));
    assert!(!resolution.binding_is_visible_unqualified(main_file, "hidden", dep_file, "hidden"));
}

#[test]
fn import_resolution_expands_selective_facade_reexport() {
    const DEP: &str = r#"
#indent 4
#no_prelude

pub fn allowed <()->i32> ():
    41

pub fn hidden <()->i32> ():
    7
"#;
    const FACADE: &str = r#"
#indent 4
#no_prelude

pub #import "dep" as @merge
"#;
    let main = r#"
#entry main
#indent 4
#no_prelude

#import "facade" as { allowed as renamed }

fn main <()->i32> ():
    renamed
"#;
    let (module, source_map) =
        load_virtual_sources(main, &[("dep.nepl", DEP), ("facade.nepl", FACADE)]);
    let resolution = ImportResolution::from_module(&module, Some(&source_map));
    let main_file = source_file_id(&source_map, "main.nepl");
    let dep_file = source_file_id(&source_map, "dep.nepl");

    assert_eq!(
        resolution.unqualified_lookup_names(main_file, "renamed"),
        vec![String::from("renamed"), String::from("allowed")]
    );
    assert!(resolution.binding_is_visible_unqualified(main_file, "renamed", dep_file, "allowed"));
    assert!(!resolution.binding_is_visible_unqualified(main_file, "hidden", dep_file, "hidden"));

    let result = nepl_core::typecheck::typecheck(
        &module,
        CompileTarget::Wasm,
        BuildProfile::Debug,
        Some(&source_map),
    );
    assert!(
        result
            .diagnostics
            .iter()
            .all(|d| d.severity != Severity::Error),
        "unexpected typecheck errors: {:?}",
        result.diagnostics
    );
    let hir = result.module.expect("hir module");
    let expected = def_id_for_name(&source_map, dep_file, "allowed");
    assert_eq!(entry_call_def_id(&hir), Some(expected));
}

#[test]
fn import_resolution_expands_qualified_merge_facade() {
    const DEP: &str = r#"
#indent 4
#no_prelude

fn allowed <()->i32> ():
    41
"#;
    const FACADE: &str = r#"
#indent 4
#no_prelude

#import "dep" as @merge
"#;
    let main = r#"
#entry main
#indent 4
#no_prelude

#import "facade" as facade

fn main <()->i32> ():
    facade::allowed
"#;
    let (module, source_map) =
        load_virtual_sources(main, &[("dep.nepl", DEP), ("facade.nepl", FACADE)]);
    let resolution = ImportResolution::from_module(&module, Some(&source_map));
    let main_file = source_file_id(&source_map, "main.nepl");
    let dep_file = source_file_id(&source_map, "dep.nepl");
    let facade_file = source_file_id(&source_map, "facade.nepl");

    let facade_targets = resolution
        .qualified_targets_for_alias(main_file, "facade")
        .expect("facade alias target");
    assert!(facade_targets.contains(&facade_file));
    assert!(facade_targets.contains(&dep_file));
}

#[test]
fn hir_user_call_keeps_def_id_for_qualified_import() {
    const DEP: &str = r#"
#indent 4
#no_prelude

fn allowed <()->i32> ():
    41
"#;
    let main = r#"
#entry main
#indent 4
#no_prelude

#import "dep" as dep

fn main <()->i32> ():
    dep::allowed
"#;
    let (hir, source_map) = typecheck_virtual(main, &[("dep.nepl", DEP)]);
    let dep_file = source_file_id(&source_map, "dep.nepl");
    let expected = def_id_for_name(&source_map, dep_file, "allowed");

    assert_eq!(entry_call_def_id(&hir), Some(expected));
}

#[test]
fn hir_user_call_keeps_local_def_id_when_open_import_is_shadowed() {
    const DEP: &str = r#"
#indent 4
#no_prelude

pub fn pick <()->i32> ():
    1
"#;
    let main = r#"
#entry main
#indent 4
#no_prelude

#import "dep" as *

fn pick <()->i32> ():
    2

fn main <()->i32> ():
    pick
"#;
    let (hir, source_map) = typecheck_virtual(main, &[("dep.nepl", DEP)]);
    let main_file = source_file_id(&source_map, "main.nepl");
    let expected = def_id_for_name(&source_map, main_file, "pick");

    assert_eq!(entry_call_def_id(&hir), Some(expected));
}

#[test]
fn facade_wrapper_can_call_same_named_alias_member() {
    const DEP: &str = r#"
#indent 4
#no_prelude

pub fn pick <()->i32> ():
    41
"#;
    const FACADE: &str = r#"
#indent 4
#no_prelude

#import "dep" as dep

pub fn pick <()->i32> ():
    dep::pick
"#;
    let main = r#"
#entry main
#indent 4
#no_prelude

#import "facade" as facade

fn main <()->i32> ():
    facade::pick
"#;
    let (hir, source_map) = typecheck_virtual(main, &[("dep.nepl", DEP), ("facade.nepl", FACADE)]);
    let dep_file = source_file_id(&source_map, "dep.nepl");
    let facade_file = source_file_id(&source_map, "facade.nepl");
    let dep_pick = def_id_for_name(&source_map, dep_file, "pick");
    let facade_pick = def_id_for_name(&source_map, facade_file, "pick");

    assert_eq!(
        first_call_def_id_for_file(&hir, facade_file),
        Some(dep_pick)
    );
    assert_eq!(entry_call_def_id(&hir), Some(facade_pick));
}

#[test]
fn resolve_import_alias_open_selective() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("main.nepl");
    let lib = dir.path().join("lib.nepl");
    let lib2 = dir.path().join("lib2.nepl");
    let lib3 = dir.path().join("lib3.nepl");
    fs::write(
        &root,
        "#import \"./lib\" as util\n#import \"./lib2\" as *\n#import \"./lib3\" as { foo as bar }\n#entry main\nfn main <() -> ()> ():\n    ()\n",
    )
    .unwrap();
    fs::write(&lib, "pub fn foo <() -> ()> ():\n    ()\n").unwrap();
    fs::write(&lib2, "pub fn baz <() -> ()> ():\n    ()\n").unwrap();
    fs::write(&lib3, "pub fn foo <() -> ()> ():\n    ()\n").unwrap();

    let builder = ModuleGraphBuilder::new(dir.path().to_path_buf());
    let g = builder.build(&root).unwrap();
    let defs = collect_defs(&g);
    let exports = ModuleGraphBuilder::build_exports(&g).unwrap();
    let export_defs = compose_exports(&defs, &exports);
    let resolved = resolve_imports(&g, &export_defs);

    let root_path = canonicalize_path(&root);
    let lib_path = canonicalize_path(&lib);
    let lib2_path = canonicalize_path(&lib2);
    let lib3_path = canonicalize_path(&lib3);
    let root_id = g.nodes.iter().find(|n| n.path == root_path).unwrap().id;
    let lib_id = g.nodes.iter().find(|n| n.path == lib_path).unwrap().id;
    let lib2_id = g.nodes.iter().find(|n| n.path == lib2_path).unwrap().id;
    let lib3_id = g.nodes.iter().find(|n| n.path == lib3_path).unwrap().id;

    let rm = resolved.modules.get(&root_id).unwrap();
    assert_eq!(rm.imports.alias_map.get("util"), Some(&lib_id));
    assert!(rm.imports.open_modules.contains(&lib2_id));
    let bar = rm.imports.selective.get("bar").unwrap();
    assert_eq!(bar.module, lib3_id);
}

#[test]
fn build_visible_map_reports_ambiguous_open() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("main.nepl");
    let a = dir.path().join("a.nepl");
    let b = dir.path().join("b.nepl");
    fs::write(
        &root,
        "#import \"./a\" as *\n#import \"./b\" as *\n#entry main\nfn main <() -> ()> ():\n    ()\n",
    )
    .unwrap();
    fs::write(&a, "pub fn foo <() -> ()> ():\n    ()\n").unwrap();
    fs::write(&b, "pub fn foo <() -> ()> ():\n    ()\n").unwrap();

    let builder = ModuleGraphBuilder::new(dir.path().to_path_buf());
    let g = builder.build(&root).unwrap();
    let defs = collect_defs(&g);
    let exports = ModuleGraphBuilder::build_exports(&g).unwrap();
    let export_defs = compose_exports(&defs, &exports);
    let resolved = resolve_imports(&g, &export_defs);
    let (_visible, diags) = build_visible_map(&defs, &resolved);

    let a_path = canonicalize_path(&a);
    let b_path = canonicalize_path(&b);
    let a_id = g.nodes.iter().find(|n| n.path == a_path).unwrap().id;
    let b_id = g.nodes.iter().find(|n| n.path == b_path).unwrap().id;
    let a_foo = defs.defs.get(&a_id).unwrap().get("foo").unwrap().id;
    let b_foo = defs.defs.get(&b_id).unwrap().get("foo").unwrap().id;
    assert_ne!(a_foo, b_foo);

    assert!(
        diags.iter().any(|d| d.code
            == Some(DiagnosticCode::Resolve(
                ResolveDiagnosticCode::ImportAmbiguous
            ))),
        "expected ambiguous import diagnostic, got {:?}",
        diags
    );
}

#[test]
fn selective_glob_opens_module() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("main.nepl");
    let lib = dir.path().join("lib.nepl");
    fs::write(
        &root,
        "#import \"./lib\" as { foo::* }\n#entry main\nfn main <() -> ()> ():\n    ()\n",
    )
    .unwrap();
    fs::write(&lib, "pub fn foo <() -> ()> ():\n    ()\n").unwrap();

    let builder = ModuleGraphBuilder::new(dir.path().to_path_buf());
    let g = builder.build(&root).unwrap();
    let defs = collect_defs(&g);
    let exports = ModuleGraphBuilder::build_exports(&g).unwrap();
    let export_defs = compose_exports(&defs, &exports);
    let resolved = resolve_imports(&g, &export_defs);
    let root_path = canonicalize_path(&root);
    let lib_path = canonicalize_path(&lib);
    let root_id = g.nodes.iter().find(|n| n.path == root_path).unwrap().id;
    let lib_id = g.nodes.iter().find(|n| n.path == lib_path).unwrap().id;
    let rm = resolved.modules.get(&root_id).unwrap();
    assert!(rm.imports.open_modules.contains(&lib_id));
}

#[test]
fn package_import_resolves_std() {
    let dir = tempdir().unwrap();
    let stdlib = dir.path().join("stdlib");
    let entry = dir.path().join("main.nepl");
    fs::create_dir_all(&stdlib).unwrap();
    fs::write(
        &entry,
        "#import \"std/util\" as *\n#entry main\nfn main <() -> ()> ():\n    ()\n",
    )
    .unwrap();
    fs::write(
        &stdlib.join("util.nepl"),
        "pub fn util <() -> ()> ():\n    ()\n",
    )
    .unwrap();

    let builder = ModuleGraphBuilder::new(stdlib.clone());
    let g = builder.build(&entry).unwrap();
    let std_node = g
        .nodes
        .iter()
        .find(|n| n.spec.package == "std" && n.spec.module == "util")
        .expect("std util module not found");
    let util_path = canonicalize_path(&stdlib.join("util.nepl"));
    assert_eq!(std_node.path, util_path);
}

#[test]
fn resolve_import_default_alias_from_nested_relative() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("main.nepl");
    let subdir = dir.path().join("lib");
    fs::create_dir_all(&subdir).unwrap();
    let lib = subdir.join("inner.nepl");
    fs::write(
        &root,
        "#import \"./lib/inner\"\n#entry main\nfn main <() -> ()> ():\n    ()\n",
    )
    .unwrap();
    fs::write(&lib, "pub fn foo <() -> ()> ():\n    ()\n").unwrap();

    let builder = ModuleGraphBuilder::new(dir.path().to_path_buf());
    let g = builder.build(&root).unwrap();
    let defs = collect_defs(&g);
    let exports = ModuleGraphBuilder::build_exports(&g).unwrap();
    let export_defs = compose_exports(&defs, &exports);
    let resolved = resolve_imports(&g, &export_defs);

    let root_path = canonicalize_path(&root);
    let lib_path = canonicalize_path(&lib);
    let root_id = g.nodes.iter().find(|n| n.path == root_path).unwrap().id;
    let lib_id = g.nodes.iter().find(|n| n.path == lib_path).unwrap().id;
    let rm = resolved.modules.get(&root_id).unwrap();
    assert_eq!(rm.imports.alias_map.get("inner"), Some(&lib_id));
}

#[test]
fn resolve_import_default_alias_from_package() {
    let dir = tempdir().unwrap();
    let stdlib = dir.path().join("stdlib");
    let pkg = dir.path().join("kp");
    let entry = dir.path().join("main.nepl");
    fs::create_dir_all(&stdlib).unwrap();
    fs::create_dir_all(&pkg).unwrap();
    fs::write(
        &entry,
        "#import \"kp/util\"\n#entry main\nfn main <() -> ()> ():\n    ()\n",
    )
    .unwrap();
    fs::write(
        &pkg.join("util.nepl"),
        "pub fn util <() -> ()> ():\n    ()\n",
    )
    .unwrap();

    let builder = ModuleGraphBuilder::new(stdlib.clone()).with_dep("kp", pkg.clone());
    let g = builder.build(&entry).unwrap();
    let defs = collect_defs(&g);
    let exports = ModuleGraphBuilder::build_exports(&g).unwrap();
    let export_defs = compose_exports(&defs, &exports);
    let resolved = resolve_imports(&g, &export_defs);

    let entry_path = canonicalize_path(&entry);
    let util_path = canonicalize_path(&pkg.join("util.nepl"));
    let root_id = g.nodes.iter().find(|n| n.path == entry_path).unwrap().id;
    let util_id = g.nodes.iter().find(|n| n.path == util_path).unwrap().id;
    let rm = resolved.modules.get(&root_id).unwrap();
    assert_eq!(rm.imports.alias_map.get("util"), Some(&util_id));
}

#[test]
fn selective_import_skips_missing_exports() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("main.nepl");
    let lib = dir.path().join("lib.nepl");
    fs::write(
        &root,
        "#import \"./lib\" as { foo, missing as miss }\n#entry main\nfn main <() -> ()> ():\n    ()\n",
    )
    .unwrap();
    fs::write(&lib, "pub fn foo <() -> ()> ():\n    ()\n").unwrap();

    let builder = ModuleGraphBuilder::new(dir.path().to_path_buf());
    let g = builder.build(&root).unwrap();
    let defs = collect_defs(&g);
    let exports = ModuleGraphBuilder::build_exports(&g).unwrap();
    let export_defs = compose_exports(&defs, &exports);
    let resolved = resolve_imports(&g, &export_defs);

    let root_path = canonicalize_path(&root);
    let root_id = g.nodes.iter().find(|n| n.path == root_path).unwrap().id;
    let rm = resolved.modules.get(&root_id).unwrap();
    assert!(rm.imports.selective.contains_key("foo"));
    assert!(!rm.imports.selective.contains_key("miss"));
}

#[test]
fn merge_import_is_treated_as_open() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("main.nepl");
    let lib = dir.path().join("lib.nepl");
    fs::write(
        &root,
        "#import \"./lib\" as @merge\n#entry main\nfn main <() -> ()> ():\n    ()\n",
    )
    .unwrap();
    fs::write(&lib, "pub fn foo <() -> ()> ():\n    ()\n").unwrap();

    let builder = ModuleGraphBuilder::new(dir.path().to_path_buf());
    let g = builder.build(&root).unwrap();
    let defs = collect_defs(&g);
    let exports = ModuleGraphBuilder::build_exports(&g).unwrap();
    let export_defs = compose_exports(&defs, &exports);
    let resolved = resolve_imports(&g, &export_defs);

    let root_path = canonicalize_path(&root);
    let lib_path = canonicalize_path(&lib);
    let root_id = g.nodes.iter().find(|n| n.path == root_path).unwrap().id;
    let lib_id = g.nodes.iter().find(|n| n.path == lib_path).unwrap().id;
    let rm = resolved.modules.get(&root_id).unwrap();
    assert!(rm.imports.open_modules.contains(&lib_id));
}

#[test]
fn build_visible_map_prefers_local_over_imports() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("main.nepl");
    let lib = dir.path().join("lib.nepl");
    let lib2 = dir.path().join("lib2.nepl");
    fs::write(
        &root,
        "#import \"./lib\" as { foo as foo_sel }\n#import \"./lib2\" as *\n#entry main\npub fn foo <() -> ()> ():\n    ()\nfn main <() -> ()> ():\n    ()\n",
    )
    .unwrap();
    fs::write(&lib, "pub fn foo <() -> ()> ():\n    ()\n").unwrap();
    fs::write(
        &lib2,
        "pub fn foo <() -> ()> ():\n    ()\npub fn bar <() -> ()> ():\n    ()\n",
    )
    .unwrap();

    let builder = ModuleGraphBuilder::new(dir.path().to_path_buf());
    let g = builder.build(&root).unwrap();
    let defs = collect_defs(&g);
    let exports = ModuleGraphBuilder::build_exports(&g).unwrap();
    let export_defs = compose_exports(&defs, &exports);
    let resolved = resolve_imports(&g, &export_defs);
    let (visible, _diags) = build_visible_map(&defs, &resolved);

    let root_path = canonicalize_path(&root);
    let lib_path = canonicalize_path(&lib);
    let lib2_path = canonicalize_path(&lib2);
    let root_id = g.nodes.iter().find(|n| n.path == root_path).unwrap().id;
    let lib_id = g.nodes.iter().find(|n| n.path == lib_path).unwrap().id;
    let lib2_id = g.nodes.iter().find(|n| n.path == lib2_path).unwrap().id;
    let vm = visible.get(&root_id).unwrap();
    let local = vm.get("foo").unwrap();
    let sel = vm.get("foo_sel").unwrap();
    let bar = vm.get("bar").unwrap();
    assert_eq!(local.module, root_id);
    assert_eq!(sel.module, lib_id);
    assert_eq!(bar.module, lib2_id);
}

#[test]
fn build_visible_map_prefers_selective_over_open() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("main.nepl");
    let lib = dir.path().join("lib.nepl");
    let lib2 = dir.path().join("lib2.nepl");
    fs::write(
        &root,
        "#import \"./lib\" as { foo }\n#import \"./lib2\" as *\n#entry main\nfn main <() -> ()> ():\n    ()\n",
    )
    .unwrap();
    fs::write(&lib, "pub fn foo <() -> ()> ():\n    ()\n").unwrap();
    fs::write(&lib2, "pub fn foo <() -> ()> ():\n    ()\n").unwrap();

    let builder = ModuleGraphBuilder::new(dir.path().to_path_buf());
    let g = builder.build(&root).unwrap();
    let defs = collect_defs(&g);
    let exports = ModuleGraphBuilder::build_exports(&g).unwrap();
    let export_defs = compose_exports(&defs, &exports);
    let resolved = resolve_imports(&g, &export_defs);
    let (visible, _diags) = build_visible_map(&defs, &resolved);

    let root_path = canonicalize_path(&root);
    let lib_path = canonicalize_path(&lib);
    let root_id = g.nodes.iter().find(|n| n.path == root_path).unwrap().id;
    let lib_id = g.nodes.iter().find(|n| n.path == lib_path).unwrap().id;
    let vm = visible.get(&root_id).unwrap();
    let foo = vm.get("foo").unwrap();
    assert_eq!(foo.module, lib_id);
}
