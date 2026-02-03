use nepl_core::ast::{Directive, ImportClause};
use nepl_core::diagnostic::Severity;
use nepl_core::lexer;
use nepl_core::module_graph::ModuleGraphBuilder;
use nepl_core::parser;
use nepl_core::resolve::{build_visible_map, collect_defs, compose_exports, resolve_imports};
use nepl_core::span::FileId;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn canonicalize_path(path: &PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.clone())
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
        lex.diagnostics.iter().all(|d| d.severity != Severity::Error),
        "unexpected lexer errors: {:?}",
        lex.diagnostics
    );
    let parse = parser::parse_tokens(FileId(0), lex);
    let module = parse.module.expect("module");
    assert!(
        parse.diagnostics.iter().all(|d| d.severity != Severity::Error),
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
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("ambiguous import")),
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
    fs::write(&pkg.join("util.nepl"), "pub fn util <() -> ()> ():\n    ()\n").unwrap();

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
