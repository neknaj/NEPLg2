mod harness;
use harness::{run_main_i32, run_main_wasi_i32};

use nepl_core::diagnostic::Severity;
use nepl_core::diagnostic_codes::{DiagnosticCode, TypeDiagnosticCode};
use nepl_core::error::CoreError;
use nepl_core::hir::HirExprKind;
use nepl_core::loader::Loader;
use nepl_core::span::FileId;
use nepl_core::typecheck;
use nepl_core::BuildProfile;
use nepl_core::{compile_module_with_source_map, compile_wasm, CompileOptions, CompileTarget};
use std::path::PathBuf;

fn compile_err(src: &str) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: None,
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
            target: None,
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

fn compile_with_loader(src: &str) -> Result<Vec<u8>, CoreError> {
    let mut loader = Loader::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("stdlib"),
    );
    let loaded = loader
        .load_inline("<test>".into(), src.to_string())
        .expect("load");
    compile_module_with_source_map(
        loaded.module,
        Some(&loaded.source_map),
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    )
    .map(|artifact| artifact.wasm)
}

fn compile_with_loader_err_has_type_code(src: &str, code: TypeDiagnosticCode) {
    let CoreError::Diagnostics(diags) = compile_with_loader(src).expect_err("expected diagnostics")
    else {
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

fn typecheck_with_loader(src: &str) -> typecheck::TypeCheckResult {
    let mut loader = Loader::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("stdlib"),
    );
    let loaded = loader
        .load_inline("<test>".into(), src.to_string())
        .expect("load");
    typecheck::typecheck(
        &loaded.module,
        CompileTarget::Wasm,
        BuildProfile::Debug,
        Some(&loaded.source_map),
    )
}

#[test]
fn function_neplg21_lambda_param_syntax() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn inc %fn i32 i32 \x:
    add x 1

fn main %fn unit i32 \unit:
    inc 41
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 42);
}

#[test]
fn function_neplg21_unit_keyword_marks_zero_arg_signature_and_lambda() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn answer %fn unit i32 \unit:
    41

fn main %fn unit i32 \unit:
    let value %i32 answer
    add value 1
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 42);
}

#[test]
fn function_neplg21_curried_type_notation_flattens_params() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn add_nums %fn i32 fn i32 i32 \a\b:
    add a b

fn main %fn unit i32 \unit:
    add_nums 10 20
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 30);
}

#[test]
fn function_neplg21_grouped_result_preserves_function_return() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn add_op %fn i32 fn i32 i32 \a\b:
    add a b

fn sub_op %fn i32 fn i32 i32 \a\b:
    sub a b

fn get_op %fn bool (fn i32 fn i32 i32) \cnd:
    if cnd:
        then:
            add_op
            @add_op
        else:
            sub_op
            @sub_op

fn main %fn unit i32 \unit:
    let f get_op true
    f 10 5
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 15);
}

#[test]
fn function_memo_call_accepts_explicit_pure_named_function_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/memo" as *

fn inc %fn i32 i32 \x:
    add x 1

fn main %fn unit i32 \unit:
    let f %fn i32 i32 memo_call @inc
    f 41
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 42);
}

#[test]
fn function_memo_call_lowers_to_dedicated_hir_boundary() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/memo" as *

fn inc %fn i32 i32 \x:
    add x 1

fn main %fn unit i32 \unit:
    let f %fn i32 i32 memo_call @inc
    f 41
"#;
    let checked = typecheck_with_loader(src);
    assert!(
        checked
            .diagnostics
            .iter()
            .all(|diagnostic| !matches!(diagnostic.severity, Severity::Error)),
        "typecheck diagnostics: {:#?}",
        checked.diagnostics
    );
    let module = checked.module.expect("typed module");
    let main = module
        .functions
        .iter()
        .find(|function| function.origin_name == "main")
        .expect("main function");
    let nepl_core::hir::HirBody::Block(block) = &main.body else {
        panic!("main should have a block body");
    };
    let Some(first_line) = block.lines.first() else {
        panic!("main should bind the memoized function");
    };
    let HirExprKind::Let { value, .. } = &first_line.expr.kind else {
        panic!(
            "first line should be a let expression: {:#?}",
            first_line.expr
        );
    };
    assert!(
        matches!(value.kind, HirExprKind::MemoizedFunctionValue(_)),
        "memo_call must preserve a compiler-known HIR boundary: {:#?}",
        value.kind
    );
}

#[test]
fn function_memo_call_rejects_implicit_function_argument() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/memo" as *

fn inc %fn i32 i32 \x:
    add x 1

fn main %fn unit i32 \unit:
    let f %fn i32 i32 memo_call inc
    f 41
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallRequiresFunctionValue);
}

/// `@inc` を一度 local binding に入れると、その値は通常の関数値になる。
///
/// Phase 1 の `memo_call` は private cache region と closure identity の proof をまだ
/// 持たないため、明示的な `@name` がその場で渡された場合だけを compiler-known boundary とする。
/// alias 経由の関数値を許すと、後続の高階関数経路と同じ扱いになり、capture や identity の
/// 設計が未確定なまま pure API を広げてしまう。
#[test]
fn function_memo_call_rejects_function_value_alias() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/memo" as *

fn inc %fn i32 i32 \x:
    add x 1

fn main %fn unit i32 \unit:
    let aliased %fn i32 i32 @inc
    let f %fn i32 i32 memo_call aliased
    f 41
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallRequiresFunctionValue);
}

/// 関数値を別の高階関数へ渡して戻した場合も、Phase 1 の `memo_call` には渡せない。
///
/// `id_func @inc` の結果は型としては `%fn i32 i32` だが、`memo_call` が要求する
/// compiler-known boundary は「この場で書かれた `@inc`」である。受け渡し後の関数値を
/// 許すには、function identity と private cache region が値経路で失われないことを
/// 別途証明する必要がある。
#[test]
fn function_memo_call_rejects_passed_through_function_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/memo" as *

fn inc %fn i32 i32 \x:
    add x 1

fn id_func %fn (fn i32 i32) (fn i32 i32) \func:
    func

fn main %fn unit i32 \unit:
    let selected %fn i32 i32 id_func @inc
    let f %fn i32 i32 memo_call selected
    f 41
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallRequiresFunctionValue);
}

/// 高階関数から戻った関数値は、たとえ中身が named pure function であっても、
/// Phase 1 の `memo_call` では受け取らない。
///
/// この制限は部分適用を導入しない NEPLg2.1 の方針と対応している。`memo_call choose true`
/// のような通常の値経路を compiler-known primitive として扱うには、function identity と
/// private cache region の non-escape proof を Resource IR まで接続してからにする。
#[test]
fn function_memo_call_rejects_returned_function_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/memo" as *

fn inc %fn i32 i32 \x:
    add x 1

fn dec %fn i32 i32 \x:
    sub x 1

fn choose %fn bool fn i32 i32 \flag:
    if flag:
        then:
            @inc
        else:
            @dec

fn main %fn unit i32 \unit:
    let selected %fn i32 i32 choose true
    let f %fn i32 i32 memo_call selected
    f 41
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallRequiresFunctionValue);
}

/// 関数リテラルは capture の有無や allocation identity を Resource IR で証明するまで
/// memoization の対象にしない。
///
/// 現在の Phase 1 は `memo_call @named_function` だけを許す。非 capture に見える
/// function literal でも、将来 capture を持つ closure と同じ値表現へ進む可能性があるため、
/// compiler-known primitive の入力としては拒否し、専用の private cache backend 設計を待つ。
#[test]
fn function_memo_call_rejects_function_literal_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/memo" as *

fn main %fn unit i32 \unit:
    let local %fn i32 i32 \x:
        add x 1
    let f %fn i32 i32 memo_call local
    f 41
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallRequiresFunctionValue);
}

#[test]
fn function_memo_call_rejects_impure_function_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/memo" as *

fn touch %impure fn i32 i32 \x:
    x

fn main %fn unit i32 \unit:
    let f %fn i32 i32 memo_call @touch
    f 41
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallRequiresPureFunction);
}

#[test]
fn function_memo_call_rejects_phase1_str_key_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/memo" as *

fn same_text %fn str str \s:
    s

fn main %fn unit i32 \unit:
    let f %fn str str memo_call @same_text
    0
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallUnsupportedKey);
}

/// f32 can be a Phase 1 value because it is copied out of the private cache,
/// but it is not a Phase 1 key.  NaN and equality/hash normalization need an
/// explicit design before a floating-point key can be admitted.
#[test]
fn function_memo_call_rejects_phase1_f32_key_even_with_user_memo_key_impl() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/memo" as *
#import "core/traits/memo" as *

impl MemoKey for f32:
    fn memo_key_eq %fn f32 fn f32 bool \a\b:
        true

    fn memo_key_hash32 %fn f32 i32 \self:
        0

fn same_float %fn f32 f32 \x:
    x

fn main %fn unit i32 \unit:
    let f %fn f32 f32 memo_call @same_float
    0
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallUnsupportedKey);
}

/// The key predicate is structural.  A nominal wrapper cannot hide an f32 field
/// behind a user-written MemoKey implementation while floating-point key
/// equality is still deliberately unsupported.
#[test]
fn function_memo_call_rejects_phase1_structural_f32_key_field() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *
#import "core/memo" as *
#import "core/traits/copy" as *
#import "core/traits/memo" as *

struct FloatKey:
    value %f32

impl MemoKey for FloatKey:
    fn memo_key_eq %fn FloatKey fn FloatKey bool \a\b:
        true

    fn memo_key_hash32 %fn FloatKey i32 \self:
        0

impl MemoValue for FloatKey:
    fn memo_value_mark %fn FloatKey FloatKey \value:
        value

impl Clone for FloatKey:
    fn clone %fn &FloatKey FloatKey \x:
        *x

impl Copy for FloatKey:
    fn copy_mark %fn FloatKey FloatKey \x:
        x

fn same_float_key %fn FloatKey FloatKey \x:
    x

fn main %fn unit i32 \unit:
    let f %fn FloatKey FloatKey memo_call @same_float_key
    0
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallUnsupportedKey);
}

/// unit is both a valid key and a valid value when it is written as a grouped
/// argument type.  This regression protects the distinction between `%fn unit`
/// as the zero-argument function marker and `%fn (unit)` as a unary function
/// whose argument value is the unit singleton.
#[test]
fn function_memo_call_accepts_phase1_unit_key_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/memo" as *

fn unit_to_i32 %fn (unit) i32 \x:
    41

fn main %fn unit i32 \unit:
    let f %fn (unit) i32 memo_call @unit_to_i32
    f unit
"#;
    compile_with_loader(src).expect("unit key/value should be accepted");
}

/// A user-defined aggregate is accepted only when the ordinary trait model can
/// prove Copy and no memory-owner or Drop boundary is present.  This keeps the
/// Phase 1 rule structural without treating every nominal struct as cache-safe.
#[test]
fn function_memo_call_accepts_phase1_structural_copy_key_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as field
#import "core/math" as *
#import "core/mem" as *
#import "core/memo" as *
#import "core/traits/copy" as *
#import "core/traits/memo" as *

struct Pair:
    value %i32

impl MemoKey for Pair:
    fn memo_key_eq %fn Pair fn Pair bool \a\b:
        eq field::get a "value" field::get b "value"

    fn memo_key_hash32 %fn Pair i32 \self:
        field::get self "value"

impl MemoValue for Pair:
    fn memo_value_mark %fn Pair Pair \value:
        value

impl Clone for Pair:
    fn clone %fn &Pair Pair \x:
        *x

impl Copy for Pair:
    fn copy_mark %fn Pair Pair \x:
        x

fn same_pair %fn Pair Pair \p:
    p

fn main %fn unit i32 \unit:
    let f %fn Pair Pair memo_call @same_pair
    let p %Pair Pair 41
    let _q %Pair f p
    0
"#;
    compile_with_loader(src).expect("structural Copy key/value should be accepted");
}

/// Copy alone is not enough for a memo cache key.  The key side must also have
/// a MemoKey implementation so the cache lookup contract has stable equality
/// and hash behavior.
#[test]
fn function_memo_call_rejects_phase1_copy_struct_without_memo_key() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/memo" as *
#import "core/traits/copy" as *

struct Pair:
    value %i32

impl Clone for Pair:
    fn clone %fn &Pair Pair \x:
        *x

impl Copy for Pair:
    fn copy_mark %fn Pair Pair \x:
        x

fn same_pair %fn Pair Pair \p:
    p

fn main %fn unit i32 \unit:
    let f %fn Pair Pair memo_call @same_pair
    0
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallUnsupportedKey);
}

/// MemoKey and MemoValue are separate contracts.  A type can be hashable as a
/// key while still not being approved as a value returned from the private
/// cache boundary.
#[test]
fn function_memo_call_rejects_phase1_copy_struct_without_memo_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/field" as field
#import "core/math" as *
#import "core/memo" as *
#import "core/traits/copy" as *
#import "core/traits/memo" as *

struct Pair:
    value %i32

impl MemoKey for Pair:
    fn memo_key_eq %fn Pair fn Pair bool \a\b:
        eq field::get a "value" field::get b "value"

    fn memo_key_hash32 %fn Pair i32 \self:
        field::get self "value"

impl Clone for Pair:
    fn clone %fn &Pair Pair \x:
        *x

impl Copy for Pair:
    fn copy_mark %fn Pair Pair \x:
        x

fn same_pair %fn Pair Pair \p:
    p

fn main %fn unit i32 \unit:
    let f %fn Pair Pair memo_call @same_pair
    0
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallUnsupportedValue);
}

#[test]
fn function_memo_call_rejects_phase1_non_copy_struct_key_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/memo" as *

struct Pair:
    value %i32

fn same_pair %fn Pair Pair \p:
    p

fn main %fn unit i32 \unit:
    let f %fn Pair Pair memo_call @same_pair
    0
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallUnsupportedKey);
}

/// RegionToken owns a free obligation.  It must not become a MemoKey or
/// MemoValue because caching it would hide owner identity and resource
/// lifecycle from the public pure function type.
#[test]
fn function_memo_call_rejects_phase1_region_token_key_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *
#import "core/memo" as *

fn same_region %fn RegionToken i32 RegionToken i32 \region:
    region

fn main %fn unit i32 \unit:
    let f %fn RegionToken i32 RegionToken i32 memo_call @same_region
    0
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallUnsupportedKey);
}

/// Function values have observable identity and callable behavior, so Phase 1
/// memoization must reject them even if the ordinary Copy model can move the
/// function reference as a small value.
#[test]
fn function_memo_call_rejects_phase1_function_value_key_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/memo" as *

fn same_function %fn (fn i32 i32) (fn i32 i32) \func:
    func

fn main %fn unit i32 \unit:
    let f %fn (fn i32 i32) (fn i32 i32) memo_call @same_function
    0
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallUnsupportedKey);
}

/// References are aliases into another storage lifetime.  A private memo cache
/// may not key on or return those aliases until the Resource IR can prove that
/// the lifetime and identity cannot escape the private cache boundary.
#[test]
fn function_memo_call_rejects_phase1_reference_key() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/memo" as *

fn read_ref %fn &i32 i32 \x:
    *x

fn main %fn unit i32 \unit:
    let f %fn &i32 i32 memo_call @read_ref
    0
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallUnsupportedKey);
}

/// MemPtr is a non-owning raw-memory view.  It is Copy for low-level boundary
/// code, but it is not a stable MemoKey or MemoValue because pointer identity
/// and pointed storage are outside the pure memoized function contract.
#[test]
fn function_memo_call_rejects_phase1_mem_ptr_key_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *
#import "core/memo" as *

fn same_ptr %fn MemPtr i32 MemPtr i32 \ptr:
    ptr

fn main %fn unit i32 \unit:
    let f %fn MemPtr i32 MemPtr i32 memo_call @same_ptr
    0
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallUnsupportedKey);
}

#[test]
fn function_memo_call_rejects_phase1_generic_function_value() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/memo" as *

fn identity <.T: Copy> %fn .T .T \x:
    x

fn main %fn unit i32 \unit:
    let f %fn i32 i32 memo_call @identity
    0
"#;
    compile_with_loader_err_has_type_code(
        src,
        TypeDiagnosticCode::MemoCallUnresolvedFunctionIdentity,
    );
}

#[test]
fn function_memo_call_rejects_immediate_application_until_private_cache_backend_exists() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/memo" as *

fn inc %fn i32 i32 \x:
    add x 1

fn main %fn unit i32 \unit:
    memo_call @inc 41
"#;
    compile_with_loader_err_has_type_code(src, TypeDiagnosticCode::MemoCallBoundaryRestricted);
}

#[test]
fn function_memo_call_local_function_with_same_name_is_not_compiler_known() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn inc %fn i32 i32 \x:
    add x 1

fn memo_call %fn (fn i32 i32) (fn i32 i32) \func:
    func

fn main %fn unit i32 \unit:
    let f %fn i32 i32 memo_call @inc
    f 41
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 42);
}

#[test]
fn function_neplg21_overload_selects_generic_impl_for_composite_copy_bound() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/option" as *
#import "core/traits/drop" as *

fn choose <.U: Copy> %fn .U i32 \x:
    1

fn choose <.U: Drop> %impure fn .U i32 \x:
    2

fn wrap <.T: Copy> %fn Option .T i32 \opt:
    choose opt

fn main %fn unit i32 \unit:
    let opt %Option i32 some 41
    wrap opt
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn function_neplg21_overloaded_generic_call_uses_ascribed_result_without_type_args() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn inc %fn i32 i32 \x:
    add x 1

fn positive_double %fn i32 Result i32 str \x:
    if gt x 0:
        then ok mul x 2
        else err "non-positive"

fn main %fn unit i32 \unit:
    let opt %Option i32 some 10
    let mapped %Option i32 map opt inc
    let res0 %Result i32 str ok 3
    let res1 %Result i32 str and_then res0 positive_double
    add unwrap mapped unwrap_ok res1
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 17);
}

#[test]
fn function_neplg21_nested_generic_producer_uses_overloaded_consumer_argument() {
    let src = r#"
#entry main
#indent 4
#target wasm
#no_prelude
#import "core/field" as field
#import "core/result" as *

struct DefaultHash32:
    tag %unit

struct HashSet<.T,.H>:
    marker %i32

struct HashSetUpdateError<.T,.H>:
    owner %HashSet .T .H

struct HashMap<.K,.V,.H>:
    marker %i32

fn new <.T,.H> %fn .H Result HashSet .T .H str \_hasher:
    ok HashSet<.T,.H> 0

fn new <.K,.V,.H> %fn .H Result HashMap .K .V .H str \_hasher:
    ok HashMap<.K,.V,.H> 0

fn must_hs %fn Result HashSet i32 DefaultHash32 str HashSet i32 DefaultHash32 \r:
    unwrap_ok r

fn must_hs %fn Result HashSet i32 DefaultHash32 HashSetUpdateError i32 DefaultHash32 HashSet i32 DefaultHash32 \r:
    match r:
        Result::Ok hs:
            hs
        Result::Err e:
            let hs %HashSet i32 DefaultHash32 field::get e "owner"
            hs

fn main %fn unit i32 \unit:
    let hs %HashSet i32 DefaultHash32 must_hs new DefaultHash32
    let marker %i32 field::get hs "marker"
    marker
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn function_neplg21_unconstrained_generic_call_type_args_are_type_error() {
    let option_src = r#"
#entry main
#indent 4
#target wasm
#import "core/option" as *

fn main %fn unit i32 \unit:
    if is_none none:
        then 1
        else 0
"#;
    compile_with_loader_err_has_type_code(
        option_src,
        TypeDiagnosticCode::GenericTypeArgsUnresolved,
    );

    let result_ok_src = r#"
#entry main
#indent 4
#target wasm
#import "core/result" as *

fn main %fn unit i32 \unit:
    if is_err ok 5:
        then 1
        else 0
"#;
    compile_with_loader_err_has_type_code(
        result_ok_src,
        TypeDiagnosticCode::GenericTypeArgsUnresolved,
    );

    let result_err_src = r#"
#entry main
#indent 4
#target wasm
#import "core/result" as *

fn main %fn unit i32 \unit:
    if is_ok err 7:
        then 1
        else 0
"#;
    compile_with_loader_err_has_type_code(
        result_err_src,
        TypeDiagnosticCode::GenericTypeArgsUnresolved,
    );
}

#[test]
fn function_neplg21_generic_call_type_args_resolve_from_explicit_consumer() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/option" as *
#import "core/result" as *

fn main %fn unit i32 \unit:
    if is_none<i32> none:
        then:
            if is_err<i32,i32> ok 5:
                then 0
                else:
                    if is_ok<i32,i32> err 7:
                        then 0
                        else 9
        else 0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 9);
}

#[test]
fn function_neplg21_generic_body_type_params_remain_allowed() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/option" as *

fn absent <.T> %fn Option .T bool \opt:
    is_none opt

fn main %fn unit i32 \unit:
    if absent<i32> none:
        then 11
        else 0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 11);
}

#[test]
fn function_basic_def_and_call() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn inc <(i32)->i32> (x):
    add x 1

fn main <()->i32> ():
    inc 41
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 42);
}

#[test]
fn function_nested() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    fn double <(i32)->i32> (x):
        mul x 2

    double 10
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 20);
}

#[test]
fn function_alias() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn add_nums <(i32, i32)->i32> (a, b):
    add a b

fn plus add_nums;
fn plus @add_nums;

fn main <()->i32> ():
    plus 10 20
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 30);
}

#[test]
fn function_first_class() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn square <(i32)->i32> (x):
    mul x x

fn apply <(i32, (i32)->i32)->i32> (val, func):
    func val

fn main <()->i32> ():
    apply 5 square
    apply 5 @square
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 25);
}

#[test]
fn function_return() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn add_op <(i32, i32)->i32> (a, b):
    add a b

fn sub_op <(i32, i32)->i32> (a, b):
    sub a b

fn get_op <(bool)->(i32, i32)->i32> (cnd):
    if cnd:
        then:
            add_op
            @add_op
        else:
            sub_op
            @sub_op

fn main <()->i32> ():
    let f get_op true
    f 10 5
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 15);
}

#[test]
fn function_signature_not_function_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn bad <i32> ():
    0

fn main <()->i32> ():
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::FunctionSignatureNotFunction);
}

#[test]
fn function_parameter_count_mismatch_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn bad <(i32)->i32> ():
    0

fn main <()->i32> ():
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::ArgumentArityMismatch);
}

#[test]
fn function_return_type_mismatch_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn bad <()->i32> ():
    ()

fn main <()->i32> ():
    bad
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::ReturnTypeMismatch);
}

#[test]
fn function_value_capture_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let y <i32> 10;
    fn add_y <(i32)->i32> (x):
        add x y
    let f @add_y;
    f 5
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::FunctionValueCapturingUnsupported);
}

#[test]
fn function_ref_requires_callable_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let x <i32> 1;
    let f @x;
    0
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::FunctionRefRequiresCallable);
}

#[test]
fn variable_type_args_not_allowed_has_type_code() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let x <i32> 1;
    x<i32>
"#;
    compile_err_has_type_code(src, TypeDiagnosticCode::VariableTypeArgsNotAllowed);
}

#[test]
fn function_literal() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let f <(i32)->i32> (x):
        add x 1

    f 10
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 11);
}

#[test]
fn function_literal_no_args() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let f <()->i32> ():
        123

    f
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 123);
}

#[test]
fn function_recursive_factorial() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn fact <(i32)->i32> (n):
    if le n 1:
        1
    else:
        mul n fact sub n 1

fn main <()->i32> ():
    fact 5
"#;
    // 5 * 4 * 3 * 2 * 1 = 120
    let v = run_main_i32(src);
    assert_eq!(v, 120);
}

#[test]
fn function_first_class_literal() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn apply <(i32, (i32)->i32)->i32> (val, func):
    func val

fn main <()->i32> ():
    // 関数リテラルを直接引数として渡す
    apply 10 (x):
        mul x 3
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 30);
}

#[test]
fn function_nested_capture_variable() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let y <i32> 10;

    // ネストされた関数が外側のスコープの変数 'y' をキャプチャする
    fn add_y <(i32)->i32> (x):
        add x y

    add_y 5
"#;
    // 5 + 10 = 15
    let v = run_main_i32(src);
    assert_eq!(v, 15);
}

#[test]
fn function_purity_check_pure_calls_impure() {
    let src = r#"
#entry main
#indent 4
#target wasi
#import "std/stdio" as *

// 副作用を持つ非純粋関数
fn impure_print <(i32)*>i32> (x):
    println_i32 x;
    x

// 純粋関数から非純粋関数を呼び出す (エラーになるべき)
fn pure_caller <(i32)->i32> (x):
    impure_print x

fn main <()->i32> ():
    pure_caller 1
"#;
    compile_err(src);
}

#[test]
fn function_purity_check_impure_calls_pure() {
    let src = r#"
#entry main
#indent 4
#target wasi
#import "std/stdio" as *
#import "core/math" as *

// 純粋関数
fn pure_mul <(i32, i32)->i32> (a, b):
    mul a b

// 非純粋関数から純粋関数を呼び出す (これはOK)
fn impure_caller <(i32)*>i32> (x):
    let res <i32> pure_mul x 10;
    println_i32 res;
    res

fn main <()*>i32> ():
    impure_caller 5
"#;
    // このテストはコンパイルと実行が通ることを確認します。
    // 実際の出力はキャプチャしませんが、戻り値は確認できます。
    let v = run_main_wasi_i32(src);
    assert_eq!(v, 50);
}

#[test]
fn function_complex_call_precedence() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn inc <(i32)->i32> (x):
    add x 1

fn main <()->i32> ():
    // sub 100 (mul (inc 5) (add 2 3))
    // sub 100 (mul 6 5)
    // sub 100 30
    // => 70
    sub 100 mul inc 5 add 2 3
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 70);
}
