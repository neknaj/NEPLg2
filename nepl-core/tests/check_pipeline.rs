use nepl_core::compiler::prepare_module_for_codegen;
use nepl_core::diagnostic_codes::{DiagnosticCode, EffectDiagnosticCode};
use nepl_core::error::CoreError;
use nepl_core::resource::{ResourceDropElaborationFunction, ResourceDropElaborationPlan};
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
fn prepare_codegen_exposes_checked_resource_drop_elaboration_plan() {
    let module = parse_module(
        r#"#entry main
#indent 4
#target core
#no_prelude

struct Guard:
    id <i32>

fn ignore <.T> <(.T)->i32> (_value):
    1

fn main <()->i32> ():
    ignore<Guard> Guard 7
"#,
    );

    let prepared = prepare_module_for_codegen(
        &module,
        CompileTarget::Wasm,
        nepl_core::BuildProfile::detect(),
    )
    .expect("prepare codegen should expose checked Resource IR drop elaboration");
    let ignore_plan = prepared
        .resource_drop_elaboration_plan
        .functions
        .iter()
        .find(|function| function.origin_name == "ignore")
        .expect("monomorphized generic function should keep a source-origin drop plan");
    assert_ne!(
        ignore_plan.name, ignore_plan.origin_name,
        "prepared drop plan should be built from monomorphized Resource IR"
    );
    assert!(ignore_plan.auto_drops.iter().any(|drop| {
        drop.source_name == "_value"
            && matches!(
                drop.requirement,
                nepl_core::resource::ResourceDropRequirement::StateOnly
            )
    }));
}

#[test]
fn resource_drop_insertion_accepts_deep_prefix_chain_without_stack_overflow() {
    let source = deep_identity_source(1105).replace(
        "#target core\n\n",
        "#target core\n#import \"core/traits/drop\" as *\n\n",
    );
    let module = parse_module(&source);
    let mut tc = nepl_core::typecheck::typecheck(
        &module,
        CompileTarget::Wasm,
        nepl_core::BuildProfile::detect(),
        None,
    );
    let mut hir = tc.module.take().expect("typecheck should produce HIR");
    let plan = ResourceDropElaborationPlan {
        functions: hir
            .functions
            .iter()
            .map(|function| ResourceDropElaborationFunction {
                name: function.name.clone(),
                origin_name: function.origin_name.clone(),
                auto_drops: Vec::new(),
                drop_points: Vec::new(),
            })
            .collect(),
    };

    nepl_core::passes::insert_resource_drops(&mut hir, &mut tc.types, &plan)
        .expect("empty checked drop plan should still traverse deep HIR iteratively");
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
    let hir = tc.module.take().expect("typecheck should produce HIR");

    let (hir, unresolved) =
        nepl_core::monomorphize::monomorphize_with_unresolved_trait_calls(&mut tc.types, hir);
    assert!(unresolved.is_empty());
    assert!(!hir.functions.is_empty());
}

#[test]
fn resource_static_check_accepts_deep_prefix_chain_without_stack_overflow() {
    let module = parse_module(&deep_identity_source(1105));
    let mut tc = nepl_core::typecheck::typecheck(
        &module,
        CompileTarget::Wasm,
        nepl_core::BuildProfile::detect(),
        None,
    );
    let hir = tc.module.take().expect("typecheck should produce HIR");
    let (hir, unresolved) =
        nepl_core::monomorphize::monomorphize_with_unresolved_trait_calls(&mut tc.types, hir);
    assert!(unresolved.is_empty());

    let resource = nepl_core::resource::lower_hir_module(&hir, &tc.types);
    let coverage =
        nepl_core::resource::compare_hir_resource_lowering_typed(&hir, &resource, &tc.types);
    let initialized_moves =
        nepl_core::resource::check_resource_initialized_moves(&resource, &tc.types);
    let borrow_lifetimes =
        nepl_core::resource::check_resource_borrow_lifetimes(&resource, &tc.types);
    let effect_boundaries = nepl_core::resource::check_resource_effect_boundaries(&resource);
    let owner_obligations =
        nepl_core::resource::check_resource_owner_obligations(&resource, &tc.types);
    assert!(
        coverage.diagnostics.is_empty()
            && initialized_moves.diagnostics.is_empty()
            && borrow_lifetimes.diagnostics.is_empty()
            && effect_boundaries.diagnostics.is_empty()
            && owner_obligations.diagnostics.is_empty(),
        "resource static-check diagnostics: {coverage:#?} {initialized_moves:#?} {borrow_lifetimes:#?} {effect_boundaries:#?} {owner_obligations:#?}"
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

#[test]
fn check_module_runs_resource_static_safety_gates() {
    let module = parse_module(
        r#"#entry main
#indent 4
#target wasm

fn raw_store <(i32,i32)->()> (p, v):
    #wasm:
        local.get p
        local.get v
        i32.store

fn main <()->i32> ():
    raw_store 0 1
    0
"#,
    );

    let err = check_module(module, CompileOptions::default())
        .expect_err("check-only pipeline must run Resource IR effect gates");
    let CoreError::Diagnostics(diagnostics) = err else {
        panic!("expected diagnostics from Resource IR effect gate");
    };
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::Effect(EffectDiagnosticCode::PureCallsImpure)
        }),
        "expected pure-call-impure diagnostic from Resource IR check, got {diagnostics:?}"
    );
}
