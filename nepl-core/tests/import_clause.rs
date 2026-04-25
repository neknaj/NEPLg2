use nepl_core::diagnostic::Diagnostic;
use nepl_core::diagnostic_ids::DiagnosticId;
use nepl_core::error::CoreError;
use nepl_core::loader::{Loader, LoaderError};
use nepl_core::{compile_module_with_source_map, CompileOptions, CompileTarget};
use std::path::PathBuf;

const DEP: &str = r#"
#indent 4
#no_prelude

#import "dep2" as *

fn allowed <()->i32> ():
    41

fn hidden <()->i32> ():
    7
"#;

const DEP2: &str = r#"
#indent 4
#no_prelude

fn leaked <()->i32> ():
    99
"#;

fn compile_with_dep(main: &str) -> Result<(), CoreError> {
    let mut loader = Loader::new(PathBuf::from("virtual_std"));
    let mut provider = |path: &PathBuf| match path.file_name().and_then(|name| name.to_str()) {
        Some("dep.nepl") => Ok(DEP.to_string()),
        Some("dep2.nepl") => Ok(DEP2.to_string()),
        _ => Err(LoaderError::Io(format!(
            "missing virtual source: {:?}",
            path
        ))),
    };
    let loaded = loader
        .load_inline_with_provider(PathBuf::from("main.nepl"), main.to_string(), &mut provider)
        .expect("load virtual sources");
    let module = loaded.module;
    let source_map = loaded.source_map;
    compile_module_with_source_map(
        module,
        Some(&source_map),
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    )
    .map(|_| ())
}

fn expect_compile_err(main: &str) -> Vec<Diagnostic> {
    let Err(CoreError::Diagnostics(diags)) = compile_with_dep(main) else {
        panic!("expected diagnostics");
    };
    diags
}

fn assert_undefined_identifier(diags: &[Diagnostic]) {
    assert!(
        diags
            .iter()
            .any(|diag| diag.id == Some(DiagnosticId::TypeUndefinedIdentifier)),
        "expected undefined identifier diagnostic, got {:?}",
        diags
    );
}

#[test]
fn alias_import_hides_unqualified_symbols_but_keeps_qualified_access() {
    let unqualified = r#"
#entry main
#indent 4
#no_prelude

#import "dep" as dep

fn main <()->i32> ():
    allowed
"#;
    let diags = expect_compile_err(unqualified);
    assert_undefined_identifier(&diags);

    let qualified = r#"
#entry main
#indent 4
#no_prelude

#import "dep" as dep

fn main <()->i32> ():
    dep::allowed
"#;
    compile_with_dep(qualified).expect("qualified alias import should compile");
}

#[test]
fn default_import_hides_unqualified_symbols_but_keeps_default_alias() {
    let unqualified = r#"
#entry main
#indent 4
#no_prelude

#import "./dep"

fn main <()->i32> ():
    allowed
"#;
    let diags = expect_compile_err(unqualified);
    assert_undefined_identifier(&diags);

    let qualified = r#"
#entry main
#indent 4
#no_prelude

#import "./dep"

fn main <()->i32> ():
    dep::allowed
"#;
    compile_with_dep(qualified).expect("default alias import should compile");
}

#[test]
fn selective_import_only_exposes_selected_symbols() {
    let allowed = r#"
#entry main
#indent 4
#no_prelude

#import "dep" as { allowed }

fn main <()->i32> ():
    allowed
"#;
    compile_with_dep(allowed).expect("selected import should compile");

    let hidden = r#"
#entry main
#indent 4
#no_prelude

#import "dep" as { allowed }

fn main <()->i32> ():
    hidden
"#;
    let diags = expect_compile_err(hidden);
    assert_undefined_identifier(&diags);
}

#[test]
fn selective_import_does_not_leak_transitive_open_imports() {
    let leaked = r#"
#entry main
#indent 4
#no_prelude

#import "dep" as { allowed }

fn main <()->i32> ():
    leaked
"#;
    let diags = expect_compile_err(leaked);
    assert_undefined_identifier(&diags);
}

#[test]
fn selective_import_alias_exposes_alias_only() {
    let aliased = r#"
#entry main
#indent 4
#no_prelude

#import "dep" as { allowed as renamed }

fn main <()->i32> ():
    renamed
"#;
    compile_with_dep(aliased).expect("selective alias import should compile");

    let original = r#"
#entry main
#indent 4
#no_prelude

#import "dep" as { allowed as renamed }

fn main <()->i32> ():
    allowed
"#;
    let diags = expect_compile_err(original);
    assert_undefined_identifier(&diags);
}

#[test]
fn selective_import_glob_follows_module_graph_open_semantics() {
    let src = r#"
#entry main
#indent 4
#no_prelude

#import "dep" as { allowed::* }

fn main <()->i32> ():
    hidden
"#;
    compile_with_dep(src).expect("selective glob import should expose imported symbols");
}

#[test]
fn open_import_exposes_unqualified_symbols() {
    let src = r#"
#entry main
#indent 4
#no_prelude

#import "dep" as *

fn main <()->i32> ():
    allowed
"#;
    compile_with_dep(src).expect("open import should expose unqualified symbols");
}
