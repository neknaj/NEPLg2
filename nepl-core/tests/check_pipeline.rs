use nepl_core::compiler::prepare_module_for_codegen;
use nepl_core::error::CoreError;
use nepl_core::span::FileId;
use nepl_core::{check_module, compile_wasm, lexer, parser, CompileOptions, CompileTarget};

fn parse_module(source: &str) -> nepl_core::ast::Module {
    let lexed = lexer::lex(FileId(0), source);
    let parsed = parser::parse_tokens(FileId(0), lexed);
    assert!(
        parsed.diagnostics.is_empty(),
        "parser diagnostics: {:?}",
        parsed.diagnostics
    );
    parsed.module.expect("module should parse")
}

fn deep_identity_source(call_count: usize) -> String {
    let mut source = String::from(
        "#entry main\n#indent 4\n#target core\n\nfn inc <(i32)->i32> (x):\n    x\n\nfn main <()->i32> ():\n    ",
    );
    for _ in 0..call_count {
        source.push_str("inc ");
    }
    source.push_str("0\n");
    source
}

#[test]
fn check_module_accepts_deep_prefix_chain_without_codegen_stack_overflow() {
    let module = parse_module(&deep_identity_source(1105));

    check_module(module, CompileOptions::default())
        .expect("check-only pipeline should not enter recursive artifact generation");
}

#[test]
fn compile_wasm_accepts_deep_prefix_chain_without_codegen_stack_overflow() {
    let artifact = compile_wasm(
        FileId(0),
        &deep_identity_source(1105),
        CompileOptions::default(),
    )
    .expect("artifact pipeline should lower a deep prefix call chain without stack overflow");

    assert!(!artifact.wasm.is_empty());
}

#[test]
fn prepare_codegen_accepts_deep_prefix_chain_without_stack_overflow() {
    let module = parse_module(&deep_identity_source(1105));

    let prepared = prepare_module_for_codegen(
        &module,
        CompileTarget::Wasm,
        nepl_core::BuildProfile::detect(),
    )
    .expect("prepare codegen should not recurse through a deep prefix call chain");
    assert!(!prepared.hir_module.functions.is_empty());
}

#[test]
fn drop_insertion_accepts_deep_prefix_chain_without_stack_overflow() {
    let module = parse_module(&deep_identity_source(1105));
    let mut tc = nepl_core::typecheck::typecheck(
        &module,
        CompileTarget::Wasm,
        nepl_core::BuildProfile::detect(),
        None,
    );
    let mut hir = tc.module.take().expect("typecheck should produce HIR");

    nepl_core::passes::insert_drops(&mut hir, &mut tc.types);
}

#[test]
fn monomorphize_accepts_deep_prefix_chain_without_stack_overflow() {
    let module = parse_module(&deep_identity_source(1105));
    let mut tc = nepl_core::typecheck::typecheck(
        &module,
        CompileTarget::Wasm,
        nepl_core::BuildProfile::detect(),
        None,
    );
    let mut hir = tc.module.take().expect("typecheck should produce HIR");
    nepl_core::passes::insert_drops(&mut hir, &mut tc.types);

    let (hir, unresolved) =
        nepl_core::monomorphize::monomorphize_with_unresolved_trait_calls(&mut tc.types, hir);
    assert!(unresolved.is_empty());
    assert!(!hir.functions.is_empty());
}

#[test]
fn move_check_accepts_deep_prefix_chain_without_stack_overflow() {
    let module = parse_module(&deep_identity_source(1105));
    let mut tc = nepl_core::typecheck::typecheck(
        &module,
        CompileTarget::Wasm,
        nepl_core::BuildProfile::detect(),
        None,
    );
    let mut hir = tc.module.take().expect("typecheck should produce HIR");
    nepl_core::passes::insert_drops(&mut hir, &mut tc.types);
    let (hir, unresolved) =
        nepl_core::monomorphize::monomorphize_with_unresolved_trait_calls(&mut tc.types, hir);
    assert!(unresolved.is_empty());

    let diagnostics = nepl_core::passes::move_check::run(&hir, &tc.types);
    assert!(
        diagnostics.is_empty(),
        "move check diagnostics: {diagnostics:?}"
    );
}

#[test]
fn check_module_reports_type_errors() {
    let module = parse_module(
        "#entry main\n#indent 4\n#target core\n\nfn main <()->i32> ():\n    unknown_symbol\n",
    );

    let err = check_module(module, CompileOptions::default())
        .expect_err("typecheck diagnostics should fail check-only pipeline");
    assert!(matches!(err, CoreError::Diagnostics(_)));
}
