mod harness;
use harness::{run_main_i32, run_main_wasi_i32};

use nepl_core::diagnostic_codes::{DiagnosticCode, TypeDiagnosticCode};
use nepl_core::error::CoreError;
use nepl_core::loader::Loader;
use nepl_core::span::FileId;
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

#[test]
fn function_neplg21_lambda_param_syntax() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn inc %fn i32 i32 \x:
    add x 1

fn main %fn () i32 \():
    inc 41
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

fn main %fn () i32 \():
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

fn main %fn () i32 \():
    let f get_op true
    f 10 5
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 15);
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

fn main %fn () i32 \():
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

fn main %fn () i32 \():
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
fn function_neplg21_unconstrained_generic_call_type_args_are_type_error() {
    let option_src = r#"
#entry main
#indent 4
#target wasm
#import "core/option" as *

fn main %fn () i32 \():
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

fn main %fn () i32 \():
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

fn main %fn () i32 \():
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

fn main %fn () i32 \():
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

fn main %fn () i32 \():
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
