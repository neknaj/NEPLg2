use nepl_core::diagnostic::Severity;
use nepl_core::loader::Loader;
use nepl_core::typecheck;
use nepl_core::{BuildProfile, CompileTarget};
use std::path::PathBuf;

fn stdlib_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib")
}

#[test]
fn large_prefix_chain_typechecks_without_fixed_reduction_cap() {
    let call_count = 1105;
    let chain = std::iter::repeat("inc")
        .take(call_count)
        .collect::<Vec<_>>()
        .join(" ");
    let src = format!(
        r#"#entry main
#indent 4
#target core

fn inc <(i32)->i32> (x):
    x

fn main <()->i32> ():
    {chain} 0
"#
    );

    let mut loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline(PathBuf::from("call_reduction_large_prefix.nepl"), src)
        .expect("load");
    let checked = typecheck::typecheck(
        &loaded.module,
        CompileTarget::Wasm,
        BuildProfile::Debug,
        Some(&loaded.source_map),
    );
    let has_errors = checked
        .diagnostics
        .iter()
        .any(|d| matches!(d.severity, Severity::Error));
    assert!(
        !has_errors,
        "large prefix chain should typecheck: {:?}",
        checked.diagnostics
    );
    assert!(checked.module.is_some(), "typecheck should produce HIR");
}
