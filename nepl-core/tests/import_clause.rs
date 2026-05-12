use nepl_core::diagnostic::Diagnostic;
use nepl_core::diagnostic_codes::DiagnosticCode;
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

const FACADE: &str = r#"
#indent 4
#no_prelude

#import "dep" as @merge
"#;

const PUB_OPEN_FACADE: &str = r#"
#indent 4
#no_prelude

pub #import "dep2" as *
"#;

const PUB_SELECTIVE_FACADE: &str = r#"
#indent 4
#no_prelude

pub #import "dep2" as { leaked as exposed }
"#;

fn compile_with_dep(main: &str) -> Result<(), CoreError> {
    compile_with_virtual_sources(
        main,
        &[
            ("dep.nepl", DEP),
            ("dep2.nepl", DEP2),
            ("facade.nepl", FACADE),
            ("pub_open_facade.nepl", PUB_OPEN_FACADE),
            ("pub_selective_facade.nepl", PUB_SELECTIVE_FACADE),
        ],
    )
}

fn compile_with_virtual_sources(main: &str, sources: &[(&str, &str)]) -> Result<(), CoreError> {
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
        diags.iter().any(|diag| diag.code
            == DiagnosticCode::Resolve(
                nepl_core::diagnostic_codes::ResolveDiagnosticCode::IdentifierUndefined
            )),
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
fn alias_import_follows_merge_facade_for_qualified_access() {
    let through_facade = r#"
#entry main
#indent 4
#no_prelude

#import "facade" as facade

fn main <()->i32> ():
    facade::allowed
"#;
    compile_with_dep(through_facade).expect("qualified facade import should expose merged symbols");

    let non_merge_transitive = r#"
#entry main
#indent 4
#no_prelude

#import "facade" as facade

fn main <()->i32> ():
    facade::leaked
"#;
    let diags = expect_compile_err(non_merge_transitive);
    assert_undefined_identifier(&diags);
}

#[test]
fn alias_import_follows_pub_open_reexports_for_qualified_access() {
    let reexported = r#"
#entry main
#indent 4
#no_prelude

#import "pub_open_facade" as facade

fn main <()->i32> ():
    facade::leaked
"#;
    compile_with_dep(reexported).expect("qualified alias import should expose pub open reexports");

    let private_transitive = r#"
#entry main
#indent 4
#no_prelude

#import "dep" as dep

fn main <()->i32> ():
    dep::leaked
"#;
    let diags = expect_compile_err(private_transitive);
    assert_undefined_identifier(&diags);
}

#[test]
fn alias_import_preserves_pub_selective_reexport_aliases() {
    let aliased = r#"
#entry main
#indent 4
#no_prelude

#import "pub_selective_facade" as facade

fn main <()->i32> ():
    facade::exposed
"#;
    compile_with_dep(aliased)
        .expect("qualified alias import should expose selected pub reexport aliases");

    let original = r#"
#entry main
#indent 4
#no_prelude

#import "pub_selective_facade" as facade

fn main <()->i32> ():
    facade::leaked
"#;
    let diags = expect_compile_err(original);
    assert_undefined_identifier(&diags);
}

#[test]
fn alias_qualified_call_survives_same_name_facade_wrapper() {
    let implementation = r#"
#indent 4
#no_prelude

fn scan <(i32)->i32> (x):
    x
"#;
    let facade = r#"
#indent 4
#no_prelude

#import "impls" as impls

fn scan <(i32)->i32> (x):
    impls::scan x
"#;
    let main = r#"
#entry main
#indent 4
#no_prelude

#import "facade_same" as *

fn main <()->i32> ():
    scan 40
"#;
    compile_with_virtual_sources(
        main,
        &[("impls.nepl", implementation), ("facade_same.nepl", facade)],
    )
    .expect("facade wrapper should keep alias-qualified access to same-name implementation");
}

#[test]
fn alias_qualified_enum_match_arm_uses_variant_member_tail() {
    let dependency = r#"
#indent 4
#no_prelude

enum E:
    A <i32>
    B

fn make <()->E> ():
    E::A 42
"#;
    let main = r#"
#entry main
#indent 4
#no_prelude

#import "enum_dep" as dep

fn main <()->i32> ():
    let value dep::make
    match value:
        dep::E::A v:
            v
        dep::E::B:
            0
"#;
    compile_with_virtual_sources(main, &[("enum_dep.nepl", dependency)])
        .expect("alias-qualified enum match arms should resolve by variant member tail");
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

#[test]
fn transitive_open_import_follows_long_chain() {
    const CHAIN_A: &str = r#"
#indent 4
#no_prelude

#import "chain_b" as *
"#;
    const CHAIN_B: &str = r#"
#indent 4
#no_prelude

#import "chain_c" as *
"#;
    const CHAIN_C: &str = r#"
#indent 4
#no_prelude

#import "chain_d" as *
"#;
    const CHAIN_D: &str = r#"
#indent 4
#no_prelude

fn leaf <()->i32> ():
    123
"#;
    let main = r#"
#entry main
#indent 4
#no_prelude

#import "chain_a" as *

fn main <()->i32> ():
    leaf
"#;
    compile_with_virtual_sources(
        main,
        &[
            ("chain_a.nepl", CHAIN_A),
            ("chain_b.nepl", CHAIN_B),
            ("chain_c.nepl", CHAIN_C),
            ("chain_d.nepl", CHAIN_D),
        ],
    )
    .expect("worklist visibility expansion should expose transitive open imports");
}

#[test]
fn transitive_open_import_preserves_selected_aliases() {
    const CHAIN_A: &str = r#"
#indent 4
#no_prelude

#import "chain_b" as *
"#;
    const CHAIN_B: &str = r#"
#indent 4
#no_prelude

#import "chain_c" as { leaf as renamed }
"#;
    const CHAIN_C: &str = r#"
#indent 4
#no_prelude

fn leaf <()->i32> ():
    77
"#;
    let aliased = r#"
#entry main
#indent 4
#no_prelude

#import "chain_a" as *

fn main <()->i32> ():
    renamed
"#;
    compile_with_virtual_sources(
        aliased,
        &[
            ("chain_a.nepl", CHAIN_A),
            ("chain_b.nepl", CHAIN_B),
            ("chain_c.nepl", CHAIN_C),
        ],
    )
    .expect("selected alias should propagate through open import edges");

    let original = r#"
#entry main
#indent 4
#no_prelude

#import "chain_a" as *

fn main <()->i32> ():
    leaf
"#;
    let Err(CoreError::Diagnostics(diags)) = compile_with_virtual_sources(
        original,
        &[
            ("chain_a.nepl", CHAIN_A),
            ("chain_b.nepl", CHAIN_B),
            ("chain_c.nepl", CHAIN_C),
        ],
    ) else {
        panic!("selected alias should not expose original source name");
    };
    assert_undefined_identifier(&diags);
}
