use nepl_core::diagnostic::Severity;
use nepl_core::diagnostic_codes::{
    BackendDiagnosticCode, DiagnosticCode, EffectDiagnosticCode, LoaderDiagnosticCode,
    ResolveDiagnosticCode, TypeDiagnosticCode,
};
use nepl_core::error::CoreError;
use nepl_core::loader::Loader;
use nepl_core::span::FileId;
use nepl_core::{check_module, compile_wasm, BuildProfile, CompileOptions, CompileTarget};
mod harness;
use harness::{
    compile_src_with_options, run_main_i32, run_main_wasi_i32,
    run_main_wasi_i32_raw_memory_boundary,
};

fn compile_ok(src: &str) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    assert!(result.is_ok(), "expected success, got {:?}", result);
}

fn compile_err(src: &str) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    assert!(result.is_err(), "expected error, got {:?}", result);
}

fn compile_err_has_type_code(src: &str, code: TypeDiagnosticCode) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    let CoreError::Diagnostics(diags) = result.expect_err("expected diagnostics") else {
        panic!("expected diagnostics");
    };
    assert!(
        diags
            .iter()
            .any(|diag| diag.code == DiagnosticCode::Type(code)),
        "missing type diagnostic {:?}: {:?}",
        code,
        diags
    );
}

fn compile_err_has_effect_code(src: &str, code: EffectDiagnosticCode) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    let CoreError::Diagnostics(diags) = result.expect_err("expected diagnostics") else {
        panic!("expected diagnostics");
    };
    assert!(
        diags
            .iter()
            .any(|diag| diag.code == DiagnosticCode::Effect(code)),
        "missing effect diagnostic {:?}: {:?}",
        code,
        diags
    );
}

fn compile_err_type_code_count(src: &str, code: TypeDiagnosticCode) -> usize {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    let CoreError::Diagnostics(diags) = result.expect_err("expected diagnostics") else {
        panic!("expected diagnostics");
    };
    diags
        .iter()
        .filter(|diag| diag.code == DiagnosticCode::Type(code))
        .count()
}

fn compile_err_has_resolve_code(src: &str, code: ResolveDiagnosticCode) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    let CoreError::Diagnostics(diags) = result.expect_err("expected diagnostics") else {
        panic!("expected diagnostics");
    };
    assert!(
        diags
            .iter()
            .any(|diag| diag.code == DiagnosticCode::Resolve(code)),
        "missing resolve diagnostic {:?}: {:?}",
        code,
        diags
    );
}

fn compile_err_has_backend_code_with_options(
    src: &str,
    options: CompileOptions,
    code: BackendDiagnosticCode,
) {
    let result = compile_wasm(FileId(0), src, options);
    let CoreError::Diagnostics(diags) = result.expect_err("expected diagnostics") else {
        panic!("expected diagnostics");
    };
    assert!(
        diags
            .iter()
            .any(|diag| diag.code == DiagnosticCode::Backend(code)),
        "missing backend diagnostic {:?}: {:?}",
        code,
        diags
    );
}

fn compile_err_has_loader_code_with_options(
    src: &str,
    options: CompileOptions,
    code: LoaderDiagnosticCode,
) {
    let diags = compile_loader_diagnostics_with_options(src, options);
    assert!(
        diags
            .iter()
            .any(|diag| diag.code == DiagnosticCode::Loader(code)),
        "missing loader diagnostic {:?}: {:?}",
        code,
        diags
    );
}

fn compile_loader_diagnostics_with_options(
    src: &str,
    options: CompileOptions,
) -> Vec<nepl_core::diagnostic::Diagnostic> {
    let result = compile_wasm(FileId(0), src, options);
    let CoreError::Diagnostics(diags) = result.expect_err("expected diagnostics") else {
        panic!("expected diagnostics");
    };
    diags
}

fn compile_err_loader_code_count_with_options(
    src: &str,
    options: CompileOptions,
    code: LoaderDiagnosticCode,
) -> usize {
    compile_loader_diagnostics_with_options(src, options)
        .iter()
        .filter(|diag| diag.code == DiagnosticCode::Loader(code))
        .count()
}

fn compile_ok_target(src: &str, target: CompileTarget) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(target),
            verbose: false,
            profile: None,
        },
    );
    assert!(result.is_ok(), "expected success, got {:?}", result);
}

fn compile_err_target(src: &str, target: CompileTarget) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(target),
            verbose: false,
            profile: None,
        },
    );
    assert!(result.is_err(), "expected error, got {:?}", result);
}

fn compile_ok_profile(src: &str, profile: BuildProfile) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: Some(profile),
        },
    );
    assert!(result.is_ok(), "expected success, got {:?}", result);
}

fn compile_err_profile(src: &str, profile: BuildProfile) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: Some(profile),
        },
    );
    assert!(result.is_err(), "expected error, got {:?}", result);
}

fn load_inline_with_stdlib(src: &str) -> nepl_core::loader::LoadResult {
    let stdlib_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib");
    let mut loader = Loader::new(stdlib_root);
    loader
        .load_inline(std::path::PathBuf::from("test.nepl"), src.to_string())
        .expect("load")
}

#[test]
fn stdlib_reimported_definition_does_not_warn_same_signature_shadow() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    add 40 2
"#;
    let loaded = load_inline_with_stdlib(src);
    let checked = nepl_core::typecheck::typecheck(
        &loaded.module,
        CompileTarget::Wasm,
        BuildProfile::Debug,
        Some(&loaded.source_map),
    );
    assert!(
        checked
            .diagnostics
            .iter()
            .all(|diag| diag.severity != Severity::Error),
        "expected stdlib math import to typecheck, got {:?}",
        checked.diagnostics
    );
    assert!(
        checked.diagnostics.iter().all(|diag| {
            diag.code != DiagnosticCode::Resolve(ResolveDiagnosticCode::ShadowSameSignatureCallable)
        }),
        "same source definition must not be reported as callable shadowing: {:?}",
        checked.diagnostics
    );
}

#[test]
fn stdlib_overlapping_imports_do_not_reprocess_same_top_level_definitions() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/field" as field
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "std/stdio" as *

fn main <()*>()> ():
    println "";
"#;
    let loaded = load_inline_with_stdlib(src);
    let checked = nepl_core::typecheck::typecheck(
        &loaded.module,
        CompileTarget::Wasi,
        BuildProfile::Debug,
        Some(&loaded.source_map),
    );
    assert!(
        checked
            .diagnostics
            .iter()
            .all(|diag| diag.severity != Severity::Error),
        "expected overlapping stdlib imports to typecheck, got {:?}",
        checked.diagnostics
    );
    assert!(
        checked.diagnostics.iter().all(|diag| {
            diag.code != DiagnosticCode::Resolve(ResolveDiagnosticCode::ItemNameConflict)
                && diag.code
                    != DiagnosticCode::Type(TypeDiagnosticCode::ImplDuplicateForTraitTarget)
        }),
        "same imported definitions must not be reprocessed as duplicate items or impls: {:?}",
        checked.diagnostics
    );
}

#[test]
fn llvm_target_in_wasm_pipeline_has_backend_code() {
    let src = r#"
#entry main

fn main <() -> i32> ():
    0
"#;
    compile_err_has_backend_code_with_options(
        src,
        CompileOptions {
            target: Some(CompileTarget::Llvm),
            verbose: false,
            profile: None,
        },
        BackendDiagnosticCode::TargetRequiresCli,
    );
}

#[test]
fn llvm_mem_bulk_copy_stdlib_lowers_to_intrinsics() {
    let src = r#"
#entry main
#indent 4
#target llvm

#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn main <()->i32> ():
    mem_copy 16 24 4
    mem_move 32 16 4
    0
"#;
    let loaded = load_inline_with_stdlib(src);
    let ll = nepl_core::codegen_llvm::emit_ll_from_module_for_target_with_source_map(
        &loaded.module,
        CompileTarget::Llvm,
        BuildProfile::Debug,
        false,
        Some(&loaded.source_map),
    )
    .expect("stdlib mem bulk copy should emit LLVM IR without clang");
    assert!(ll.contains("declare void @llvm.memcpy.p0.p0.i32"));
    assert!(ll.contains("declare void @llvm.memmove.p0.p0.i32"));
    assert!(ll.contains("define void @mem_copy(i32 %dst, i32 %src, i32 %len)"));
    assert!(ll.contains("define void @mem_move(i32 %dst, i32 %src, i32 %len)"));
    assert!(ll.contains("call void @llvm.memcpy.p0.p0.i32"));
    assert!(ll.contains("call void @llvm.memmove.p0.p0.i32"));
}

#[test]
fn llvm_scalar_intrinsics_use_shared_backend_lowering() {
    let src = r#"
#entry main
#indent 4
#target llvm

fn to_f32 <(i32)->f32> (v):
    #intrinsic "i32_to_f32" <> (v)

fn bits_to_f32 <(i32)->f32> (v):
    #intrinsic "reinterpret_i32_f32" <> (v)

fn f32_bits <(f32)->i32> (v):
    #intrinsic "reinterpret_f32_i32" <> (v)

fn main <()->i32> ():
    let f <f32> to_f32 7;
    let bits <f32> bits_to_f32 1065353216;
    let a <i32> f32_bits f;
    let b <i32> f32_bits bits;
    a
"#;
    let loaded = load_inline_with_stdlib(src);
    let ll = nepl_core::codegen_llvm::emit_ll_from_module_for_target_with_source_map(
        &loaded.module,
        CompileTarget::Llvm,
        BuildProfile::Debug,
        false,
        Some(&loaded.source_map),
    )
    .expect("shared scalar intrinsic backend lowering should emit LLVM IR");
    assert!(ll.contains("sitofp i32"));
    assert!(ll.contains("bitcast i32"));
    assert!(ll.contains("bitcast float"));
    assert!(!ll.contains("i32_to_f32"));
    assert!(!ll.contains("reinterpret_i32_f32"));
    assert!(!ll.contains("reinterpret_f32_i32"));
}

#[test]
fn llvm_allocator_helper_is_emitted_for_codegen_inserted_alloc() {
    let src = r#"
#entry main
#indent 4
#target llvm

#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct Pair:
    left <i32>
    right <i32>

fn main <()->i32> ():
    let p <Pair> Pair 1 2
    0
"#;
    let loaded = load_inline_with_stdlib(src);
    let ll = nepl_core::codegen_llvm::emit_ll_from_module_for_target_with_source_map(
        &loaded.module,
        CompileTarget::Llvm,
        BuildProfile::Debug,
        false,
        Some(&loaded.source_map),
    )
    .expect("codegen-inserted aggregate allocation should emit allocator helpers");
    assert!(ll.contains("call i32 @\"__nepl_rt_alloc"));
    assert!(ll
        .lines()
        .any(|line| line.starts_with("define i32 @") && line.contains("__nepl_rt_alloc")));
    assert!(ll.contains("load_i32"));
    assert!(ll.contains("store_i32"));
}

#[test]
fn llvm_hashmap_string_key_preserves_explicit_hasher_type_args() {
    let src = r#"
#entry main
#indent 4
#target llvm

#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/field" as field
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/traits/copy" as *
#import "core/traits/hash" as *
#import "core/traits/hash_key" as *

fn must_hms <(Result<HashMap<str,i32,DefaultHash32>, Diag>)*>HashMap<str,i32,DefaultHash32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

struct ModKey:
    raw <i32>

impl HashKey for ModKey:
    fn eq <(ModKey,ModKey)->bool> (a, b):
        eq field::get a "raw" field::get b "raw"

    fn hash32 <(ModKey)->i32> (self):
        rem_s field::get self "raw" 17

impl Clone for ModKey:
    fn clone <(&ModKey)->ModKey> (self):
        *self

impl Copy for ModKey:
    fn copy_mark <(ModKey)->ModKey> (self):
        self

struct ModHasher:
    tag <()>

impl Clone for ModHasher:
    fn clone <(&ModHasher)->ModHasher> (self):
        *self

impl Copy for ModHasher:
    fn copy_mark <(ModHasher)->ModHasher> (self):
        self

impl Hasher<ModKey> for ModHasher:
    fn hash32 <(ModHasher,ModKey)->i32> (_h, key):
        rem_s field::get key "raw" 7

fn must_hmk <(Result<HashMap<ModKey,i32,ModHasher>, Diag>)*>HashMap<ModKey,i32,ModHasher>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let hms <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let hms <HashMap<str,i32,DefaultHash32>> must_hms insert hms "key" 7;
    let a <i32> match get &hms "key":
        Option::Some v:
            v
        Option::None:
            0
    free hms;
    let hmk <HashMap<ModKey,i32,ModHasher>> must_hmk new ModHasher;
    let hmk <HashMap<ModKey,i32,ModHasher>> must_hmk insert hmk (ModKey 10) 3;
    let b <i32> match get &hmk (ModKey 10):
        Option::Some v:
            v
        Option::None:
            0
    free hmk;
    add a b
"#;
    let loaded = load_inline_with_stdlib(src);
    let ll = nepl_core::codegen_llvm::emit_ll_from_module_for_target_with_source_map(
        &loaded.module,
        CompileTarget::Llvm,
        BuildProfile::Debug,
        false,
        Some(&loaded.source_map),
    )
    .expect("explicit HashMap hasher type argument should survive LLVM monomorphize");
    assert!(!ll.contains("hash_with__H_K__i32__pure_str_Option_T_i32"));
    assert!(!ll.contains("hash_with__H_K__i32__pure_str_ModHasher"));
    assert!(ll.contains("hash_with__H_K__i32__pure_ModKey_ModHasher"));
    assert!(ll.contains("DefaultHash32"));
}

#[test]
fn llvm_unit_locals_and_payload_binds_remain_in_scope() {
    let src = r#"
#entry main
#indent 4
#target llvm

#import "core/result" as *
#import "core/traits/copy" as *

fn unwrap_unit <(Result<(),i32>)->()> (r):
    match r:
        Result::Ok v:
            v
        Result::Err _e:
            #intrinsic "unreachable" <> ()

fn main <()->i32> ():
    let u <()> ()
    let a <()> u
    let b <()> u
    let r <Result<(),i32>> Result<(),i32>::Ok b
    unwrap_unit r
    0
"#;
    let loaded = load_inline_with_stdlib(src);
    let ll = nepl_core::codegen_llvm::emit_ll_from_module_for_target_with_source_map(
        &loaded.module,
        CompileTarget::Llvm,
        BuildProfile::Debug,
        false,
        Some(&loaded.source_map),
    )
    .expect("unit locals and unit payload binds should remain visible to LLVM lowering");
    assert!(ll.contains("define i32"));
}

#[test]
fn llvm_match_i32_literal_lowers_to_switch() {
    let src = r#"
#entry main
#indent 4
#target llvm

fn main <()->i32> ():
    let x <i32> 92
    match x:
        34:
            1
        92:
            2
        _:
            3
"#;
    let loaded = load_inline_with_stdlib(src);
    let ll = nepl_core::codegen_llvm::emit_ll_from_module_for_target_with_source_map(
        &loaded.module,
        CompileTarget::Llvm,
        BuildProfile::Debug,
        false,
        Some(&loaded.source_map),
    )
    .expect("i32 literal match should emit LLVM IR");
    assert!(ll.contains("switch i32"));
    assert!(ll.contains("i32 92, label"));
}

#[test]
fn llvm_reference_scalar_addr_of_and_deref_lowers() {
    let src = r#"
#entry main
#indent 4
#target llvm

fn deref_i32 <(&i32)->i32> (x):
    *x

fn main <()->i32> ():
    let a <i32> 6
    deref_i32 &a
"#;
    let loaded = load_inline_with_stdlib(src);
    let ll = nepl_core::codegen_llvm::emit_ll_from_module_for_target_with_source_map(
        &loaded.module,
        CompileTarget::Llvm,
        BuildProfile::Debug,
        false,
        Some(&loaded.source_map),
    )
    .expect("scalar references should emit LLVM IR without unsupported AddrOf/Deref");
    assert!(ll.contains("store i32"));
    assert!(ll.contains("load i32"));
}

#[test]
fn llvm_reference_aggregate_addr_of_lowers() {
    let src = r#"
#entry main
#indent 4
#target llvm

struct Pair:
    left <i32>
    right <i32>

fn observe_pair <(&Pair)->i32> (p):
    0

fn main <()->i32> ():
    let p <Pair> Pair 1 2
    observe_pair &p
"#;
    let loaded = load_inline_with_stdlib(src);
    let ll = nepl_core::codegen_llvm::emit_ll_from_module_for_target_with_source_map(
        &loaded.module,
        CompileTarget::Llvm,
        BuildProfile::Debug,
        false,
        Some(&loaded.source_map),
    )
    .expect("aggregate address-of should emit LLVM IR without unsupported AddrOf");
    assert!(ll.contains("define i32 @\"observe_pair\"(i32 %p0)"));
    assert!(ll.contains("call i32"));
    assert!(ll.contains("observe_pair"));
}

#[test]
fn compiles_literal_main() {
    let src = r#"
#entry main
fn main <() -> i32> ():
    #import "core/math" as *
    1
"#;
    compile_ok(src);
}

#[test]
fn compiles_add_block_expression() {
    let src = r#"
#entry main
#indent 4

#if[target=wasm]
fn add <(i32, i32) -> i32> (a, b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <() -> i32> ():
    #import "core/math" as *
    add 1:
        add 2 3
"#;
    compile_ok(src);
}

#[test]
fn set_type_mismatch_is_error() {
    let src = r#"
#entry main
fn main <() -> ()> ():
    let mut x <i32> 0;
    set x ();
"#;
    compile_err(src);
}

#[test]
fn set_immutable_variable_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->()> ():
    let x <i32> 0;
    set x 1;
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::MutationImmutable);
}

#[test]
fn set_undefined_variable_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->()> ():
    set x 1;
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::VariableUndefined);
}

#[test]
fn let_noshadow_shadow_has_resolve_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let noshadow x <i32> 1;
    let x <i32> 2;
    x
"#;
    compile_err_has_resolve_code(src, ResolveDiagnosticCode::ShadowNoShadowViolation);
}

#[test]
fn unknown_intrinsic_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->()> ():
    #intrinsic "rv_core_007_unknown" <> ()
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::IntrinsicUnknown);
}

#[test]
fn intrinsic_arg_type_mismatch_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->f32> ():
    #intrinsic "i32_to_f32" <> (true)
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::IntrinsicArgTypeMismatch);
}

#[test]
fn callsite_span_type_arg_arity_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->()> ():
    #intrinsic "callsite_span" <> ()
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::IntrinsicTypeArgArityMismatch);
}

#[test]
fn field_accessor_intrinsic_arg_arity_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

struct Pair:
    x <i32>
    y <i32>

fn main <()->()> ():
    let p Pair 1 2;
    #intrinsic "set_field" <> (p,"x")
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::IntrinsicArgArityMismatch);
}

#[test]
fn invalid_integer_literal_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    999999999999999999999999999999
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::LiteralIntInvalid);
}

#[test]
fn invalid_ast_char_literal_has_type_code() {
    let span = nepl_core::span::Span::dummy();
    let main = nepl_core::ast::Ident {
        name: "main".to_string(),
        span,
    };
    let module = nepl_core::ast::Module {
        doc: None,
        indent_width: 4,
        directives: vec![nepl_core::ast::Directive::Entry { name: main.clone() }],
        root: nepl_core::ast::Block {
            span,
            items: vec![nepl_core::ast::Stmt::FnDef(nepl_core::ast::FnDef {
                doc: None,
                vis: nepl_core::ast::Visibility::Private,
                name: main,
                no_shadow: false,
                type_params: Vec::new(),
                signature: nepl_core::ast::TypeExpr::Function {
                    params: Vec::new(),
                    result: Box::new(nepl_core::ast::TypeExpr::Char),
                    effect: nepl_core::ast::Effect::Pure,
                },
                params: Vec::new(),
                body: nepl_core::ast::FnBody::Parsed(nepl_core::ast::Block {
                    span,
                    items: vec![nepl_core::ast::Stmt::Expr(nepl_core::ast::PrefixExpr {
                        items: vec![nepl_core::ast::PrefixItem::Literal(
                            nepl_core::ast::Literal::Char(i32::MAX as u32 + 1),
                            span,
                        )],
                        trailing_semis: 0,
                        trailing_semi_span: None,
                        span,
                    })],
                }),
            })],
        },
    };
    let result = check_module(
        module,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    let CoreError::Diagnostics(diags) = result.expect_err("invalid char literal should fail")
    else {
        panic!("expected diagnostics");
    };
    assert!(
        diags.iter().any(
            |diag| diag.code == DiagnosticCode::Type(TypeDiagnosticCode::LiteralCharOutOfRange)
        ),
        "missing char literal diagnostic: {:?}",
        diags
    );
}

#[test]
fn nested_generic_function_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    fn id <.T> <(.T)->.T> (x):
        x
    id<i32> 1
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::NestedGenericFunctionUnsupported);
}

#[test]
fn nested_raw_block_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    block:
        #wasm:
            i32.const 1
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::RawBlockInvalidPlacement);
}

#[test]
fn pure_cannot_call_impure() {
    let src = r#"
#entry main
#indent 4

fn imp <(i32) *> i32> (x):
    #import "core/math" as *
    add x 1

fn pure <(i32) -> i32> (x):
    imp x

fn main <() -> i32> ():
    pure 1
"#;
    compile_err(src);
}

#[test]
fn iftarget_non_wasm_is_skipped() {
    let src = r#"
#entry main

#if[target=llvm]
fn bad <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    1
"#;
    compile_ok(src);
}

#[test]
fn invalid_iftarget_is_diagnostic() {
    let src = r#"
#entry main

#if[target=unknown_target]
fn bad <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    1
"#;
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    let CoreError::Diagnostics(diags) = result.expect_err("unknown target gate should fail") else {
        panic!("expected diagnostics");
    };
    assert!(
        diags.iter().any(|diag| diag.code
            == DiagnosticCode::Loader(
                nepl_core::diagnostic_codes::LoaderDiagnosticCode::ConditionalGateInvalid
            )),
        "missing invalid conditional gate diagnostic: {:?}",
        diags
    );
}

#[test]
fn invalid_ifprofile_is_diagnostic() {
    let src = r#"
#entry main

#if[profile=staging]
fn bad <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    1
"#;
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: Some(BuildProfile::Debug),
        },
    );
    let CoreError::Diagnostics(diags) = result.expect_err("unknown profile gate should fail")
    else {
        panic!("expected diagnostics");
    };
    assert!(
        diags.iter().any(|diag| diag.code
            == DiagnosticCode::Loader(
                nepl_core::diagnostic_codes::LoaderDiagnosticCode::ConditionalGateInvalid
            )),
        "missing invalid conditional gate diagnostic: {:?}",
        diags
    );
}

#[test]
fn invalid_iftarget_in_nested_block_is_diagnostic() {
    let src = r#"
#entry main
#indent 4

fn main <() -> i32> ():
    if true:
        then:
            #if[target=unknown_target]
            unknown_symbol
            1
        else:
            0
"#;
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    let CoreError::Diagnostics(diags) = result.expect_err("nested unknown target gate should fail")
    else {
        panic!("expected diagnostics");
    };
    assert!(
        diags.iter().any(|diag| diag.code
            == DiagnosticCode::Loader(
                nepl_core::diagnostic_codes::LoaderDiagnosticCode::ConditionalGateInvalid
            )),
        "missing invalid conditional gate diagnostic: {:?}",
        diags
    );
}

#[test]
fn ifprofile_debug_gate() {
    let src = r#"
#entry main

#if[profile=debug]
fn only_debug <() -> i32> ():
    123

fn main <() -> i32> ():
    only_debug
"#;
    compile_ok_profile(src, BuildProfile::Debug);
    compile_err_profile(src, BuildProfile::Release);
}

#[test]
fn ifprofile_release_skips_in_debug() {
    let src = r#"
#entry main

#if[profile=release]
fn only_release <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    0
"#;
    compile_ok_profile(src, BuildProfile::Debug);
    compile_err_profile(src, BuildProfile::Release);
}

#[test]
fn wasm_stack_mismatch_is_error() {
    let src = r#"
#entry main

#if[target=wasm]
fn add_one <(i32)->i32> (a):
    #wasm:
        local.get $a
        // missing value for add
        i32.add

fn main <() -> i32> ():
    #import "core/math" as *
    add_one 1
"#;
    compile_err(src);
}

#[test]
fn wasi_allows_wasm_gate() {
    let src = r#"
#entry main

#if[target=wasm]
fn only_wasm <() -> i32> ():
    123

fn main <() -> i32> ():
    only_wasm
"#;
    compile_ok_target(src, CompileTarget::Wasi);
}

#[test]
fn wasm_skips_wasi_gate() {
    let src = r#"
#entry main

#if[target=wasi]
fn only_wasi <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    0
"#;
    compile_ok_target(src, CompileTarget::Wasm);
    compile_err_target(src, CompileTarget::Wasi);
}

#[test]
fn import_and_prelude_directives_are_accepted() {
    let src = r#"
#entry main
#prelude std/prelude_base
#no_prelude
#import "core/math" as { add as plus, math::* }
#import "./part" as @merge

fn main <() -> i32> ():
    0
"#;
    compile_ok(src);
}

#[test]
fn string_literal_compiles() {
    let src = r#"
#entry main
#indent 4
#extern "env" "print_str" fn print <(str)*>()>

fn main <()*> ()> ():
    print "hello";
    ()
"#;
    compile_ok(src);
}

#[test]
fn pipe_injects_first_arg() {
    let src = r#"
#entry main
#indent 4

#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    add 1 add 2 3 |> add 4
"#;
    compile_ok(src);
}

#[test]
fn pipe_requires_callable_target() {
    let src = r#"
#entry main
#indent 4

fn main <()->i32> ():
    1 |> 2
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::PipeInvalid);
}

#[test]
fn pipe_with_type_annotation_is_ok() {
    let src = r#"
#entry main
#indent 4

#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    1 |> <i32> add 4
"#;
    compile_ok(src);
}

#[test]
fn pipe_with_double_type_annotation_is_ok() {
    let src = r#"
#entry main
#indent 4

#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    1 |> <i32> <i32> add 4
"#;
    compile_ok(src);
}

#[test]
fn pipe_target_missing_after_annotation_is_error() {
    let src = r#"
#entry main
#indent 4

fn main <()->i32> ():
    1 |> <i32> 2
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::PipeInvalid);
}

#[test]
fn wasi_import_rejected_on_wasm_target() {
    let src = r#"
#entry main
#indent 4
#extern "wasi_snapshot_preview1" "fd_write" fn fd_write <(i32,i32,i32,i32)->i32>
fn main <()->()> ():
    ()
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::ExternWasiTargetMismatch);
}

#[test]
fn extern_signature_not_function_has_type_code() {
    let entry_span = nepl_core::span::Span {
        file_id: FileId(0),
        start: 0,
        end: 1,
    };
    let extern_span = nepl_core::span::Span {
        file_id: FileId(0),
        start: 2,
        end: 3,
    };
    let main = nepl_core::ast::Ident {
        name: "main".to_string(),
        span: entry_span,
    };
    let bad = nepl_core::ast::Ident {
        name: "bad".to_string(),
        span: extern_span,
    };
    let module = nepl_core::ast::Module {
        doc: None,
        indent_width: 4,
        directives: vec![
            nepl_core::ast::Directive::Entry { name: main.clone() },
            nepl_core::ast::Directive::Extern {
                vis: nepl_core::ast::Visibility::Private,
                module: "env".to_string(),
                name: "bad".to_string(),
                func: bad,
                signature: nepl_core::ast::TypeExpr::I32.with_span(extern_span),
                span: extern_span,
            },
        ],
        root: nepl_core::ast::Block {
            span: entry_span,
            items: vec![nepl_core::ast::Stmt::FnDef(nepl_core::ast::FnDef {
                doc: None,
                vis: nepl_core::ast::Visibility::Private,
                name: main,
                no_shadow: false,
                type_params: Vec::new(),
                signature: nepl_core::ast::TypeExpr::Function {
                    params: Vec::new(),
                    result: Box::new(nepl_core::ast::TypeExpr::Unit),
                    effect: nepl_core::ast::Effect::Pure,
                },
                params: Vec::new(),
                body: nepl_core::ast::FnBody::Parsed(nepl_core::ast::Block {
                    span: entry_span,
                    items: vec![nepl_core::ast::Stmt::Expr(nepl_core::ast::PrefixExpr {
                        items: vec![nepl_core::ast::PrefixItem::Literal(
                            nepl_core::ast::Literal::Unit,
                            entry_span,
                        )],
                        trailing_semis: 0,
                        trailing_semi_span: None,
                        span: entry_span,
                    })],
                }),
            })],
        },
    };
    let result = check_module(
        module,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    let CoreError::Diagnostics(diags) =
        result.expect_err("non-function extern signature should fail")
    else {
        panic!("expected diagnostics");
    };
    assert!(
        diags.iter().any(|diag| diag.code
            == DiagnosticCode::Type(TypeDiagnosticCode::ExternSignatureNotFunction)),
        "missing extern signature diagnostic: {:?}",
        diags
    );
}

#[test]
fn enum_type_param_bounds_have_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

trait Marker:
    fn mark <(Self)->i32> (_self):
        0

enum Box<.T: Marker>:
    Item <.T>

fn main <()->()> ():
    ()
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::EnumTypeParamBoundsUnsupported);
}

#[test]
fn struct_type_param_bounds_have_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

trait Marker:
    fn mark <(Self)->i32> (_self):
        0

struct Box<.T: Marker>:
    value <.T>

fn main <()->()> ():
    ()
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::StructTypeParamBoundsUnsupported);
}

#[test]
fn duplicate_enum_name_has_resolve_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

enum Foo:
    A

enum Foo:
    B

fn main <()->()> ():
    ()
"#;
    compile_err_has_resolve_code(src, ResolveDiagnosticCode::ItemNameConflict);
}

#[test]
fn duplicate_struct_name_has_resolve_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

struct Foo:
    value <i32>

struct Foo:
    other <i32>

fn main <()->()> ():
    ()
"#;
    compile_err_has_resolve_code(src, ResolveDiagnosticCode::ItemNameConflict);
}

#[test]
fn name_conflict_enum_fn_has_resolve_code() {
    let src = r#"
#entry main
#indent 4

enum Foo:
    A

fn Foo <()->i32> ():
    0

fn main <()->i32> ():
    Foo
"#;
    compile_err_has_resolve_code(src, ResolveDiagnosticCode::ItemNameConflict);
}

#[test]
fn function_alias_target_not_found_has_resolve_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn plus missing;

fn main <()->i32> ():
    0
"#;
    compile_err_has_resolve_code(src, ResolveDiagnosticCode::AliasTargetNotFound);
}

#[test]
fn function_alias_name_conflict_enum_has_resolve_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

enum Plus:
    Value

fn add <(i32, i32)->i32> (a, _b):
    a

fn Plus add;

fn main <()->i32> ():
    0
"#;
    compile_err_has_resolve_code(src, ResolveDiagnosticCode::ItemNameConflict);
}

#[test]
fn wasm_cannot_use_stdio() {
    let src = r#"
#entry main
#indent 4
#import "std/stdio" as *

fn main <()->()> ():
    print "hi"
"#;
    compile_err_target(src, CompileTarget::Wasm);
}

#[test]
fn run_add_returns_12() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    add 10 2
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 12);
}

#[test]
fn match_option_some_returns_value() {
    let src = r#"
#entry main
#indent 4
#import "core/option" as *

fn main <()* >i32> ():
    match some 5:
        Some v:
            v
        None:
            0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 5);
}

#[test]
fn list_get_out_of_bounds_err() {
    let src = r#"
#entry main
#indent 4
#import "alloc/collections/list" as *
#import "core/option" as *
#import "core/result" as *

fn main <()* >i32> ():
    let lst <List<i32>> unwrap_ok<List<i32>, Diag> new<i32>;
    let lst uwok cons<i32> 1 lst;
    let r get<i32> lst 10;
    match r:
        Some v:
            v
        None:
            0
"#;
    let v = run_main_wasi_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn non_exhaustive_match_is_error() {
    let src = r#"
#entry main
#indent 4
#import "core/option" as *

fn main <()->i32> ():
    match some 1:
        Some v:
            v
"#;
    compile_err(src);
}

#[test]
fn target_directive_sets_default_to_wasi() {
    let src = r#"
#target wasi
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    print "ok"
"#;
    let wasm = compile_src_with_options(
        src,
        CompileOptions {
            target: None,
            verbose: false,
            profile: None,
        },
    );
    assert!(!wasm.is_empty());
}

#[test]
fn duplicate_target_directive_is_error() {
    let src = r#"
#target wasm
#target wasi
#entry main
fn main <()->i32> ():
    0
"#;
    compile_err_has_loader_code_with_options(
        src,
        CompileOptions {
            target: None,
            verbose: false,
            profile: None,
        },
        LoaderDiagnosticCode::TargetMultipleDirective,
    );
}

#[test]
fn unknown_target_directive_has_loader_code() {
    let src = r#"
#target wasi2
#entry main
fn main <()->i32> ():
    0
"#;
    compile_err_has_loader_code_with_options(
        src,
        CompileOptions {
            target: None,
            verbose: false,
            profile: None,
        },
        LoaderDiagnosticCode::TargetUnknown,
    );
    assert_eq!(
        compile_err_loader_code_count_with_options(
            src,
            CompileOptions {
                target: None,
                verbose: false,
                profile: None,
            },
            LoaderDiagnosticCode::TargetUnknown,
        ),
        1
    );
}

#[test]
fn missing_entry_function_has_resolve_code() {
    let src = r#"
#entry missing

fn main <()->i32> ():
    0
"#;
    compile_err_has_resolve_code(src, ResolveDiagnosticCode::EntryFunctionMissingOrAmbiguous);
}

#[test]
fn undefined_identifier_has_resolve_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    missing_value
"#;
    compile_err_has_resolve_code(src, ResolveDiagnosticCode::IdentifierUndefined);
}

#[test]
fn overloads_by_param_type_are_allowed() {
    let src = r#"
#entry main
#indent 4

fn id <(i32)->i32> (x):
    x

fn id <(f32)->f32> (x):
    x

fn main <()->i32> ():
    let tmp id 1.0;
    id 1
"#;
    compile_ok(src);
}

#[test]
fn overloads_with_different_arity_are_error() {
    let src = r#"
#entry main
#indent 4

fn foo <(i32)->i32> (x):
    x

fn foo <(i32,i32)->i32> (a,b):
    a

fn main <()->i32> ():
    foo 1
"#;
    compile_err(src);
}

#[test]
fn overload_no_match_has_type_code() {
    let src = r#"
#entry main
#indent 4

fn id <(i32)->i32> (x):
    x

fn id <(f32)->f32> (x):
    x

fn main <()->i32> ():
    id true
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::OverloadNoMatch);
}

#[test]
fn overload_type_args_mismatch_has_type_code() {
    let src = r#"
#entry main
#indent 4

fn id <.T> <(.T)->.T> (x):
    x

fn main <()->i32> ():
    id<i32, i32> 1
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::OverloadTypeArgsMismatch);
}

#[test]
fn overloads_ambiguous_return_type_is_error() {
    let src = r#"
#entry main
#indent 4

fn foo <(i32)->i32> (x):
    x

fn foo <(i32)->f32> (x):
    1.0

fn main <()->i32> ():
    let y foo 1;
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::OverloadAmbiguous);
}

#[test]
fn overloads_without_expected_return_do_not_use_return_shape_specificity() {
    let src = r#"
#entry main
#indent 4

struct Wide:
    lo <i32>
    hi <i32>

fn castlike <(i32)->i32> (x):
    x

fn castlike <(i32)->Wide> (x):
    Wide x x

fn main <()->i32> ():
    let y castlike 1
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::OverloadAmbiguous);
}

#[test]
fn trait_method_call_with_impl_compiles() {
    let src = r#"
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn main <()->i32> ():
    Show::show 1
"#;
    compile_ok(src);
}

#[test]
fn pure_function_calling_impure_trait_method_has_effect_code() {
    let src = r#"
#entry main
#indent 4

struct Cell:
    value <i32>

trait Touch:
    fn touch <(Self)*>i32> (x):
        1

impl Touch for Cell:
    fn touch <(Cell)*>i32> (x):
        1

fn main <()->i32> ():
    Touch::touch Cell 1
"#;
    compile_err_has_effect_code(src, EffectDiagnosticCode::PureCallsImpure);
}

#[test]
fn trait_bound_satisfied_in_generic() {
    let src = r#"
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn call_show <.T: Show> <(.T)->i32> (x):
    Show::show x

fn main <()->i32> ():
    call_show 5
"#;
    compile_ok(src);
}

#[test]
fn trait_unknown_capability_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

trait BadCap:
    #capability cpoy
    fn f <(Self)->Self> (x):
        x

fn main <()->()> ():
    ()
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::TraitCapabilityUnknown);
}

#[test]
fn clone_capability_bound_constrains_generic_impl_target() {
    let src = r#"
#entry main
#indent 4
#target wasm
#no_prelude

trait CloneLike:
    #capability clone
    fn mark <(Self)->i32> (_self):
        0

struct Payload:
    value <i32>

struct Wrap<.T>:
    value <.T>

impl<.T: CloneLike> CloneLike for Wrap<.T>:
    fn mark <(Wrap<.T>)->i32> (_self):
        7

fn main <()->i32> ():
    let wrapped <Wrap<Payload>> Wrap Payload 1
    CloneLike::mark wrapped
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::TraitBoundUnsatisfied);
}

#[test]
fn clone_capability_bound_allows_matching_clone_payload() {
    let src = r#"
#entry main
#indent 4
#target wasm
#no_prelude

trait CloneLike:
    #capability clone
    fn mark <(Self)->i32> (_self):
        0

struct Payload:
    value <i32>

struct Wrap<.T>:
    value <.T>

impl CloneLike for Payload:
    fn mark <(Payload)->i32> (_self):
        3

impl<.T: CloneLike> CloneLike for Wrap<.T>:
    fn mark <(Wrap<.T>)->i32> (_self):
        7

fn main <()->i32> ():
    let wrapped <Wrap<Payload>> Wrap Payload 1
    CloneLike::mark wrapped
"#;
    assert_eq!(run_main_i32(src), 7);
}

#[test]
fn recursive_clone_capability_impl_does_not_prove_itself() {
    let src = r#"
#entry main
#indent 4
#target wasm
#no_prelude

trait CloneLike:
    #capability clone
    fn mark <(Self)->i32> (_self):
        0

struct Payload:
    value <i32>

impl<.T: CloneLike> CloneLike for .T:
    fn mark <(.T)->i32> (_self):
        7

fn main <()->i32> ():
    let payload <Payload> Payload 1
    CloneLike::mark payload
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::TraitBoundUnsatisfied);
}

#[test]
fn recursive_copy_capability_impl_does_not_prove_itself() {
    let src = r#"
#entry main
#indent 4
#target wasm
#no_prelude

trait CloneLike:
    #capability clone
    fn clone_mark <(Self)->i32> (_self):
        0

trait CopyLike:
    #capability copy
    fn copy_mark <(Self)->i32> (_self):
        0

struct Payload:
    value <i32>

impl<.T: CopyLike> CloneLike for .T:
    fn clone_mark <(.T)->i32> (_self):
        1

impl<.T: CopyLike> CopyLike for .T:
    fn copy_mark <(.T)->i32> (_self):
        7

fn requires_copy <.T: CopyLike> <(.T)->i32> (_value):
    1

fn main <()->i32> ():
    let payload <Payload> Payload 1
    requires_copy payload
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::TraitBoundUnsatisfied);
}

#[test]
fn trait_method_type_params_have_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

trait Boxy:
    fn get <.T> <(Self)->.T> (x):
        x

fn main <()->()> ():
    ()
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::TraitMethodTypeParamsUnsupported);
}

#[test]
fn generic_trait_impl_method_resolves_by_trait_args() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

trait HashKey:
    #capability clone
    #capability copy
    fn clone <(Self)->Self> (self):
        self

    fn eq <(Self,Self)->bool> (a, b):
        eq a b

    fn hash32 <(Self)->i32> (self):
        0

impl HashKey for i32:
    fn clone <(i32)->i32> (self):
        self

    fn eq <(i32,i32)->bool> (a, b):
        eq a b

    fn hash32 <(i32)->i32> (self):
        self

trait Hasher<.K: HashKey>:
    #capability clone
    #capability copy
    fn hash32 <(Self,.K)->i32> (self, key):
        0

struct DefaultHash32:
    tag <()>

impl<.K: HashKey> Hasher<.K> for DefaultHash32:
    fn hash32 <(DefaultHash32,.K)->i32> (_self, key):
        HashKey::hash32 key

fn hash_with <.K: HashKey,.H: Hasher<.K>> <(.H,.K)->i32> (hasher, key):
    Hasher::hash32 hasher key

fn main <()->i32> ():
    hash_with DefaultHash32 9
"#;
    assert_eq!(run_main_i32(src), 9);
}

#[test]
fn generic_intrinsic_store_load_struct_preserves_fields() {
    let src = r#"
#entry main
#indent 4
#target std
#import "core/field" as field
#import "core/math" as *
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct Point:
    x <i32>
    y <i32>

fn roundtrip <.T> <(.T)*>.T> (x):
    let p <i32> alloc_raw size_of<.T>;
    store<.T> p x;
    let out <.T> load<.T> p;
    dealloc_raw p size_of<.T>;
    out

fn main <()*>i32> ():
    let p <Point> roundtrip<Point> Point 10 20;
    add mul field::get p "x" 100 field::get p "y"
"#;
    assert_eq!(run_main_wasi_i32_raw_memory_boundary(src), 1020);
}

#[test]
fn generic_hashkey_eq_after_load_uses_concrete_impl() {
    let src = r#"
#entry main
#indent 4
#target std
#import "core/field" as field
#import "core/math" as *
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/traits/copy" as *
#import "core/traits/hash_key" as *

struct Point:
    x <i32>
    y <i32>

impl HashKey for Point:
    fn eq <(Point,Point)->bool> (a, b):
        let ax <i32> field::get a "x"
        let ay <i32> field::get a "y"
        let bx <i32> field::get b "x"
        let by <i32> field::get b "y"
        and (eq ax bx) (eq ay by)

    fn hash32 <(Point)->i32> (self):
        xor field::get self "x" field::get self "y"

impl Clone for Point:
    fn clone <(&Point)->Point> (self):
        *self

impl Copy for Point:
    fn copy_mark <(Point)->Point> (self):
        self

fn same_after_store <.T: HashKey> <(.T,.T)*>bool> (a, b):
    let p <i32> alloc_raw size_of<.T>;
    store<.T> p a;
    let saved <.T> load<.T> p;
    dealloc_raw p size_of<.T>;
    hashkey_eq saved b

fn main <()*>i32> ():
    if same_after_store<Point> (Point 10 20) (Point 10 20) 1 0
"#;
    assert_eq!(run_main_wasi_i32_raw_memory_boundary(src), 1);
}

#[test]
fn generic_hashkey_value_survives_hash_before_store() {
    let src = r#"
#entry main
#indent 4
#target std
#import "core/field" as field
#import "core/math" as *
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/traits/copy" as *
#import "core/traits/hash_key" as *

struct Point:
    x <i32>
    y <i32>

impl HashKey for Point:
    fn eq <(Point,Point)->bool> (a, b):
        let ax <i32> field::get a "x"
        let ay <i32> field::get a "y"
        let bx <i32> field::get b "x"
        let by <i32> field::get b "y"
        and (eq ax bx) (eq ay by)

    fn hash32 <(Point)->i32> (self):
        xor field::get self "x" field::get self "y"

impl Clone for Point:
    fn clone <(&Point)->Point> (self):
        *self

impl Copy for Point:
    fn copy_mark <(Point)->Point> (self):
        self

fn hash_then_store <.T: HashKey&Copy> <(.T)*>.T> (x):
    let _h <i32> hashkey_hash32 x;
    let p <i32> alloc_raw size_of<.T>;
    store<.T> p x;
    let out <.T> load<.T> p;
    dealloc_raw p size_of<.T>;
    out

fn main <()*>i32> ():
    let p <Point> hash_then_store<Point> Point 10 20;
    add mul field::get p "x" 100 field::get p "y"
"#;
    assert_eq!(run_main_wasi_i32_raw_memory_boundary(src), 1020);
}

#[test]
fn hashmap_custom_struct_key_roundtrips_value() {
    let src = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "core/traits/copy" as *
#import "core/traits/hash" as *
#import "core/traits/hash_key" as *

struct Point:
    x <i32>
    y <i32>

impl HashKey for Point:
    fn eq <(Point,Point)->bool> (a, b):
        let ax <i32> field::get a "x"
        let ay <i32> field::get a "y"
        let bx <i32> field::get b "x"
        let by <i32> field::get b "y"
        and (eq ax bx) (eq ay by)

    fn hash32 <(Point)->i32> (self):
        xor field::get self "x" field::get self "y"

impl Clone for Point:
    fn clone <(&Point)->Point> (self):
        *self

impl Copy for Point:
    fn copy_mark <(Point)->Point> (self):
        self

fn must_hmp <(Result<HashMap<Point,i32,DefaultHash32>, Diag>)*>HashMap<Point,i32,DefaultHash32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let map0 <HashMap<Point,i32,DefaultHash32>> must_hmp new DefaultHash32;
    let map1 <HashMap<Point,i32,DefaultHash32>> must_hmp insert map0 (Point 10 20) 99;
    let got <i32> match get &map1 (Point 10 20):
        Option::Some n:
            n
        Option::None:
            0
    free map1;
    got
"#;
    assert_eq!(run_main_wasi_i32(src), 99);
}

#[test]
fn result_i64_wildcard_match_does_not_reuse_arm_bind_local() {
    let src = r#"
#target wasm
#entry main
#indent 4
#import "core/result" as *

fn main <()->i32> ():
    let r <Result<i64,i32>> Result<i64,i32>::Err 1;
    let ok <bool> match r:
        Result::Ok _:
            false
        Result::Err _:
            true
    if ok 1 0
"#;
    assert_eq!(run_main_i32(src), 1);
}

#[test]
fn match_i32_literal_arm_returns_selected_case() {
    let src = r#"
#target wasm
#entry main
#indent 4

fn classify <(i32)->i32> (x):
    match x:
        34:
            1
        92:
            2
        _:
            3

fn main <()->i32> ():
    classify 92
"#;
    assert_eq!(run_main_i32(src), 2);
}

#[test]
fn match_i32_literal_wildcard_returns_default_case() {
    let src = r#"
#target wasm
#entry main
#indent 4

fn classify <(i32)->i32> (x):
    match x:
        34:
            1
        92:
            2
        _:
            3

fn main <()->i32> ():
    classify 7
"#;
    assert_eq!(run_main_i32(src), 3);
}

#[test]
fn match_bool_literal_arms_return_selected_case() {
    let src = r#"
#target wasm
#entry main
#indent 4

fn classify <(bool)->i32> (flag):
    match flag:
        true:
            1
        false:
            2

fn main <()->i32> ():
    classify false
"#;
    assert_eq!(run_main_i32(src), 2);
}

#[test]
fn match_i32_duplicate_literal_is_error() {
    let src = r#"
#target wasm
#entry main
#indent 4

fn main <()->i32> ():
    let x <i32> 1
    match x:
        1:
            10
        1:
            20
        _:
            0
"#;
    compile_err_profile(src, BuildProfile::Debug);
}

#[test]
fn match_i32_literal_without_wildcard_is_non_exhaustive() {
    let src = r#"
#target wasm
#entry main
#indent 4

fn main <()->i32> ():
    let x <i32> 1
    match x:
        1:
            10
        2:
            20
"#;
    compile_err_profile(src, BuildProfile::Debug);
}

#[test]
fn generic_store_after_generic_trait_probe_preserves_struct() {
    let src = r#"
#entry main
#indent 4
#target std
#import "core/field" as field
#import "core/math" as *
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/traits/copy" as *
#import "core/traits/hash_key" as *

struct Point:
    x <i32>
    y <i32>

impl HashKey for Point:
    fn eq <(Point,Point)->bool> (a, b):
        let ax <i32> field::get a "x"
        let ay <i32> field::get a "y"
        let bx <i32> field::get b "x"
        let by <i32> field::get b "y"
        and (eq ax bx) (eq ay by)

    fn hash32 <(Point)->i32> (self):
        xor field::get self "x" field::get self "y"

impl Clone for Point:
    fn clone <(&Point)->Point> (self):
        *self

impl Copy for Point:
    fn copy_mark <(Point)->Point> (self):
        self

fn probe <.T: HashKey&Copy> <(.T)->bool> (key):
    hashkey_eq key key

fn write_after_probe <.T: HashKey&Copy,.V> <(.T,.V)*>.T> (key, value):
    let _ok <bool> probe<.T> key;
    let p <i32> alloc_raw add size_of<.T> size_of<.V>;
    store<.T> p key;
    store<.V> add p size_of<.T> value;
    let out <.T> load<.T> p;
    dealloc_raw p add size_of<.T> size_of<.V>;
    out

fn main <()*>i32> ():
    let p <Point> write_after_probe<Point,i32> (Point 10 20) 99;
    add mul field::get p "x" 100 field::get p "y"
"#;
    assert_eq!(run_main_wasi_i32_raw_memory_boundary(src), 1020);
}

#[test]
fn generic_store_uses_nested_address_call_without_stealing_value_arg() {
    let src = r#"
#entry main
#indent 4
#target std
#import "core/field" as field
#import "core/math" as *
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct Point:
    x <i32>
    y <i32>

fn slot_ptr <.T,.V> <(i32,i32)->i32> (base, idx):
    add base mul idx add size_of<.T> size_of<.V>

fn write_nested <.T,.V> <(.T,.V)*>.T> (key, value):
    let p <i32> alloc_raw add size_of<.T> size_of<.V>;
    store<.T> slot_ptr<.T,.V> p 0 key;
    store<.V> add p size_of<.T> value;
    let out <.T> load<.T> p;
    dealloc_raw p add size_of<.T> size_of<.V>;
    out

fn main <()*>i32> ():
    let p <Point> write_nested<Point,i32> (Point 10 20) 99;
    add mul field::get p "x" 100 field::get p "y"
"#;
    assert_eq!(run_main_wasi_i32_raw_memory_boundary(src), 1020);
}

#[test]
fn trait_bound_missing_impl_is_error() {
    let src = r#"
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

fn call_show <.T: Show> <(.T)->i32> (x):
    Show::show x

fn main <()->i32> ():
    call_show 1
"#;
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    let CoreError::Diagnostics(diags) = result.expect_err("missing trait impl should fail") else {
        panic!("expected diagnostics");
    };
    assert!(
        diags.iter().any(|diag| diag.code
            == DiagnosticCode::Type(
                nepl_core::diagnostic_codes::TypeDiagnosticCode::TraitBoundUnsatisfied
            )),
        "missing trait bound diagnostic code: {:?}",
        diags
    );
}

#[test]
fn impl_type_params_in_trait_args_allowed_for_concrete_target() {
    let src = r#"
#entry main
#indent 4

trait Mapper<.T>:
    fn map <(Self,.T)->i32> (_self, _value):
        0

impl<.T> Mapper<.T> for i32:
    fn map <(i32,.T)->i32> (_self, _value):
        7

fn main <()->i32> ():
    Mapper::map 0 123
"#;
    compile_ok(src);
}

#[test]
fn trait_application_type_param_conflict_has_type_code() {
    let src = r#"
#entry main
#indent 4

trait Mapper<.T>:
    fn map <(Self,.T)->.T> (_self, value):
        value

impl<.T> Mapper<.T> for i32:
    fn map <(i32,.T)->.T> (_self, value):
        value

fn main <()->i32> ():
    let _x <bool> Mapper::map 0 123
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::TraitConstraintConflict);
}

#[test]
fn trait_self_type_ambiguity_has_type_code() {
    let src = r#"
#entry main
#indent 4

trait Factory:
    fn make <(Self)->i32> (_self):
        0

fn choose <.A: Factory,.B: Factory> <()->i32> ():
    Factory::make

fn main <()->i32> ():
    choose
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::TraitSelfTypeAmbiguous);
}

#[test]
fn impl_generic_target_diagnostic_uses_type_expr_span() {
    let src = r#"
#entry main
#indent 4

trait Marker:
    fn mark <(Self)->i32> (x):
        0

impl Marker for .T:
    fn mark <(.T)->i32> (x):
        0

fn main <()->i32> ():
    0
"#;
    let target_start = src.find(".T").expect("generic impl target") as u32;
    let target_end = target_start + ".T".len() as u32;
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    let CoreError::Diagnostics(diags) = result.expect_err("generic impl target should fail") else {
        panic!("expected diagnostics");
    };
    let diag = diags
        .iter()
        .find(|d| {
            d.code
                == DiagnosticCode::Type(
                    nepl_core::diagnostic_codes::TypeDiagnosticCode::ImplTargetNotConcrete,
                )
        })
        .unwrap_or_else(|| panic!("missing concrete impl target diagnostic: {:?}", diags));
    assert_eq!(diag.primary.span.file_id, FileId(0));
    assert_eq!(diag.primary.span.start, target_start);
    assert_eq!(diag.primary.span.end, target_end);
}

#[test]
fn inherent_impl_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

impl i32:
    fn id <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
"#;
    assert_eq!(
        compile_err_type_code_count(src, TypeDiagnosticCode::ImplInherentUnsupported),
        1
    );
}

#[test]
fn impl_unknown_trait_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

impl Missing for i32:
    fn f <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
"#;
    assert_eq!(
        compile_err_type_code_count(src, TypeDiagnosticCode::TraitUnknown),
        1
    );
}

#[test]
fn impl_trait_type_arg_count_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

trait Boxy<.T>:
    fn get <(Self)->.T> (x):
        unreachable

impl Boxy<i32, i32> for i32:
    fn get <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
"#;
    assert_eq!(
        compile_err_type_code_count(src, TypeDiagnosticCode::TraitTypeParamsUnsupported),
        1
    );
}

#[test]
fn copy_impl_target_not_copy_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm
#no_prelude

trait Copyish:
    #capability copy
    fn copy_mark <(Self)->Self> (x):
        x

impl Copyish for &mut i32:
    fn copy_mark <(&mut i32)->&mut i32> (x):
        x

fn main <()->i32> ():
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::CopyImplTargetNotCopy);
}

#[test]
fn copy_impl_rejects_compiler_owner_token_target() {
    let src = r#"
#entry main
#indent 4
#target wasm

#import "core/mem" as *
#import "core/traits/copy" as *

impl<.T> Copy for RegionToken<.T>:
    fn copy_mark <(RegionToken<.T>)->RegionToken<.T>> (x):
        x

fn main <()->i32> ():
    0
"#;
    let loaded = load_inline_with_stdlib(src);
    let checked = nepl_core::typecheck::typecheck(
        &loaded.module,
        CompileTarget::Wasm,
        BuildProfile::Debug,
        Some(&loaded.source_map),
    );
    assert!(
        checked.diagnostics.iter().any(
            |diag| diag.code == DiagnosticCode::Type(TypeDiagnosticCode::CopyImplTargetNotCopy)
        ),
        "missing Copy impl owner-token rejection: {:?}",
        checked.diagnostics
    );
}

#[test]
fn copy_impl_allows_user_struct_named_region_token() {
    let src = r#"
#entry main
#indent 4
#target wasm
#no_prelude

trait Clone:
    #capability clone
    fn clone <(&Self)->Self> (x):
        *x

trait Copy:
    #capability copy
    fn copy_mark <(Self)->Self> (x):
        x

struct RegionToken:
    value <i32>

impl Clone for RegionToken:
    fn clone <(&RegionToken)->RegionToken> (x):
        *x

impl Copy for RegionToken:
    fn copy_mark <(RegionToken)->RegionToken> (x):
        x

fn main <()->i32> ():
    let a <RegionToken> RegionToken 1
    let b <RegionToken> a
    let c <RegionToken> a
    0
"#;
    compile_ok(src);
}

#[test]
fn duplicate_impl_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::ImplDuplicateForTraitTarget);
}

#[test]
fn copy_impl_requires_clone_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm
#no_prelude

trait Copyish:
    #capability copy
    fn copy_mark <(Self)->Self> (x):
        x

impl Copyish for i32:
    fn copy_mark <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::CopyImplRequiresClone);
}

#[test]
fn impl_duplicate_method_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x
    fn show <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::ImplDuplicateMethod);
}

#[test]
fn impl_method_type_params_have_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <.T> <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::TraitMethodTypeParamsUnsupported);
}

#[test]
fn impl_method_not_in_trait_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x
    fn extra <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::ImplMethodNotInTrait);
}

#[test]
fn impl_method_signature_mismatch_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i64> (x):
        x

fn main <()->i32> ():
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::ImplMethodSignatureMismatch);
}

#[test]
fn impl_missing_trait_method_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

trait Pair:
    fn a <(Self)->i32> (x):
        x
    fn b <(Self)->i32> (x):
        x

impl Pair for i32:
    fn a <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::ImplMissingTraitMethod);
}

#[test]
fn trait_method_arity_mismatch_is_error() {
    let src = r#"
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn main <()->i32> ():
    Show::show 1 2
"#;
    compile_err(src);
}

#[test]
fn trait_method_type_args_unsupported_has_type_code() {
    let src = r#"
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        0

fn main <()->i32> ():
    Show::show<i32> 1
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::TraitMethodTypeArgsUnsupported);
}

#[test]
fn trait_method_not_found_has_type_code() {
    let src = r#"
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        0

fn main <()->i32> ():
    Show::missing 1
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::TraitMethodNotFound);
}

#[test]
fn unknown_trait_bound_is_error() {
    let src = r#"
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

fn call_show <.T: Missing> <(.T)->i32> (x):
    0

fn main <()->i32> ():
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::TraitBoundUnknown);
}

#[test]
fn trait_bound_type_arg_count_mismatch_has_type_code() {
    let src = r#"
#entry main
#indent 4

trait Boxy<.T>:
    fn get <(Self)->.T> (x):
        unreachable

fn use_boxy <.T: Boxy<i32, i32>> <(.T)->i32> (_x):
    0

fn main <()->i32> ():
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::TraitTypeParamsUnsupported);
}

#[test]
fn unreachable_does_not_force_never_in_generic() {
    let src = r#"
#entry main
#indent 4

fn pick <.T> <(.T)->.T> (x):
    if:
        true
        then:
            x
        else:
            #intrinsic "unreachable" <> ()

fn main <()->i32> ():
    pick 1
"#;
    compile_ok(src);
}
