use std::path::PathBuf;

use nepl_core::ast::Stmt;
use nepl_core::loader::Loader;

fn stdlib_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../stdlib")
}

#[test]
fn slash_colon_doc_comment_attaches_to_defs() {
    let src = r#"#no_prelude
//: # VecLike
struct VecLike:
    len <i32>

//: value を返す
fn answer <()->i32> ():
    42
"#;
    let mut loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline(PathBuf::from("doc_test.nepl"), src.to_string())
        .expect("load");

    let mut found_struct = false;
    let mut found_fn = false;
    for item in &loaded.module.root.items {
        match item {
            Stmt::StructDef(def) if def.name.name == "VecLike" => {
                found_struct = true;
                assert_eq!(def.doc.as_deref(), Some("# VecLike"));
            }
            Stmt::FnDef(def) if def.name.name == "answer" => {
                found_fn = true;
                assert_eq!(def.doc.as_deref(), Some("value を返す"));
            }
            _ => {}
        }
    }

    assert!(found_struct);
    assert!(found_fn);
}

#[test]
fn stdlib_hashmap_struct_doc_is_attached_to_hashmap_struct() {
    let mut loader = Loader::new(stdlib_root());
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../stdlib/alloc/collections/hashmap.nepl");
    let loaded = loader.load(&path).expect("load stdlib hashmap");

    let hashmap = loaded
        .module
        .root
        .items
        .iter()
        .find_map(|item| match item {
            Stmt::StructDef(def) if def.name.name == "HashMap" => Some(def),
            _ => None,
        })
        .expect("HashMap struct");

    let doc = hashmap.doc.as_deref().expect("HashMap doc");
    assert!(doc.lines().any(|line| line == "## HashMap"));
    assert!(doc.contains("[要素数/ようそすう]"));
    assert!(doc.contains("tombstone 数"));
    assert!(doc.contains("storage"));
    assert!(doc.contains("hasher"));
    assert!(!doc.contains("# hashmap"));
}

#[test]
fn stdlib_fenwick_module_doc_is_separate_from_struct_doc() {
    let mut loader = Loader::new(stdlib_root());
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../stdlib/alloc/collections/fenwick.nepl");
    let loaded = loader.load(&path).expect("load stdlib fenwick");

    let module_doc = loaded.module.doc.as_deref().expect("module doc");
    assert!(module_doc.contains("# fenwick"));
    assert!(module_doc.contains("Fenwick Tree"));

    let fenwick = loaded
        .module
        .root
        .items
        .iter()
        .find_map(|item| match item {
            Stmt::StructDef(def) if def.name.name == "Fenwick" => Some(def),
            _ => None,
        })
        .expect("Fenwick struct");

    let struct_doc = fenwick.doc.as_deref().expect("Fenwick doc");
    assert!(struct_doc.contains("## Fenwick"));
    assert!(struct_doc.contains("所有者"));
}
