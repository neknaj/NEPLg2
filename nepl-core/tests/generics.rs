mod harness;
use harness::run_main_i32;

use nepl_core::span::FileId;
use nepl_core::{compile_wasm, CompileOptions, CompileTarget};

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

#[test]
fn generics_fn_identity_multi_instantiation() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn id <.T> <(.T)->.T> (x):
    x

fn main <()->i32> ():
    let a <i32> id 7
    let b <bool> id true
    if b:
        add a 1
        else:
            a
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 8);
}

#[test]
fn generics_enum_option_and_match() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *

enum Option<.T>:
    None
    Some <.T>

fn is_some <.T> <(Option<.T>)->bool> (o):
    match o:
        Some v:
            true
        None:
            false

fn main <()->i32> ():
    let a <Option<i32>> Option::Some 5
    let b <Option<bool>> Option::None
    let _nested <Option<Option<i32>>> Option::Some Option::Some 1
    let x <bool> is_some a
    let y <bool> is_some b
    <i32> if:
        cond:
            x
        then:
            if y 10 20
        else:
            30
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 20);
}

#[test]
fn generics_struct_pair_construction() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *
#import "core/math" as *

struct Pair<.A,.B>:
    first <.A>
    second <.B>

fn take_ab <(Pair<i32,bool>)->i32> (p):
    10

fn take_ba <(Pair<bool,i32>)->i32> (p):
    20

fn main <()->i32> ():
    let p1 <Pair<i32,bool>> Pair 1 true
    let p2 <Pair<bool,i32>> Pair false 2
    add take_ab p1 take_ba p2
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 30);
}

#[test]
fn generics_param_requires_dot() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn id <T> <(T)->T> (x):
    x

fn main <()->i32> ():
    0
"#;

    compile_err(src);
}

#[test]
fn generics_enum_param_requires_dot() {
    let src = r#"
#entry main
#indent 4
#target wasm

enum Option<T>:
    None
    Some <T>

fn main <()->i32> ():
    0
"#;

    compile_err(src);
}

#[test]
fn generics_struct_param_requires_dot() {
    let src = r#"
#entry main
#indent 4
#target wasm

struct Pair<T,U>:
    a <T>
    b <U>

fn main <()->i32> ():
    0
"#;

    compile_err(src);
}

#[test]
fn generics_enum_payload_arithmetic() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *
#import "core/math" as *

enum Option<.T>:
    None
    Some <.T>

fn bump <(Option<i32>)->i32> (o):
    match o:
        Some v:
            add v 1
        None:
            0

fn main <()->i32> ():
    bump Option::Some 9
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn generics_multi_type_params_function() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn first <.A,.B> <(.A,.B)->.A> (a,b):
    a

fn main <()->i32> ():
    let x <i32> first 3 true
    let y <bool> first false 7
    if y:
        add x 1
        else:
            x
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn generics_enum_none_typed_by_ascription() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *

enum Option<.T>:
    None
    Some <.T>

fn is_none <(Option<i32>)->bool> (o):
    match o:
        None:
            true
        Some v:
            false

fn main <()->i32> ():
    let n <Option<i32>> Option::None
    if is_none n 1 0
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn generics_make_none_from_context() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *

enum Option<.T>:
    None
    Some <.T>

fn make_none <.T> <()->Option<.T>> ():
    Option::None

fn main <()->i32> ():
    let x <Option<i32>> make_none
    match x:
        None:
            1
        Some v:
            0
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn generics_generic_calls_generic() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn id <.T> <(.T)->.T> (x):
    x

fn wrap <.U> <(.U)->.U> (x):
    id x

fn main <()->i32> ():
    let a <i32> wrap 9
    a
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 9);
}

#[test]
fn generics_pipe_into_generic() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn id <.T> <(.T)->.T> (x):
    x

fn main <()->i32> ():
    let a <i32> 5 |> id
    add a 2
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 7);
}

#[test]
fn generics_option_none_inferred_by_param() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *

enum Option<.T>:
    None
    Some <.T>

fn is_none_i32 <(Option<i32>)->bool> (o):
    match o:
        None:
            true
        Some v:
            false

fn main <()->i32> ():
    if is_none_i32 Option::None 1 0
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn generics_pair_inferred_by_param() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *

struct Pair<.A,.B>:
    first <.A>
    second <.B>

fn take_ab <(Pair<i32,bool>)->i32> (p):
    5

fn main <()->i32> ():
    take_ab Pair 1 true
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 5);
}

#[test]
fn generics_make_pair_wrapper() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *
#import "core/math" as *

struct Pair<.A,.B>:
    first <.A>
    second <.B>

fn make_pair <.A,.B> <(.A,.B)->Pair<.A,.B>> (a,b):
    Pair a b

fn take_ab <(Pair<i32,bool>)->i32> (p):
    10

fn take_ba <(Pair<bool,i32>)->i32> (p):
    20

fn main <()->i32> ():
    add take_ab make_pair 1 true take_ba make_pair false 2
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 30);
}

#[test]
fn generics_make_some_wrapper() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *
#import "core/math" as *

enum Option<.T>:
    None
    Some <.T>

fn make_some <.T> <(.T)->Option<.T>> (v):
    Option::Some v

fn main <()->i32> ():
    let a <Option<i32>> make_some 3
    let b <Option<bool>> make_some true
    let x <i32> match a:
        Some v:
            v
        None:
            0
    let y <i32> match b:
        Some flag:
            if flag 1 0
        None:
            0
    add x y
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 4);
}

#[test]
fn generics_nested_option_match() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *

enum Option<.T>:
    None
    Some <.T>

fn unwrap_nested <.T> <(Option<Option<.T>>,.T)->.T> (oo, default):
    match oo:
        Some inner:
            match inner:
                Some v:
                    v
                None:
                    default
        None:
            default

fn main <()->i32> ():
    unwrap_nested Option::Some Option::Some 9 0
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 9);
}

#[test]
fn generics_enum_two_params_match_payloads() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *

enum Either<.A,.B>:
    Left <.A>
    Right <.B>

fn pick <.A,.B> <(.A,.B,bool)->Either<.A,.B>> (a,b,cond):
    if cond:
        Either::Left a
        else:
            Either::Right b

fn to_i32 <(Either<i32,bool>)->i32> (e):
    match e:
        Left v:
            v
        Right b:
            if b 1 0

fn main <()->i32> ():
    let e <Either<i32,bool>> pick 7 true true
    to_i32 e
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 7);
}

#[test]
fn generics_nested_apply_in_payload() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/mem" as *

enum Option<.T>:
    None
    Some <.T>

enum Wrap<.T>:
    Wrap <Option<.T>>

fn unwrap <(Wrap<i32>)->i32> (w):
    match w:
        Wrap o:
            match o:
                Some v:
                    v
                None:
                    0

fn main <()->i32> ():
    unwrap Wrap::Wrap Option::Some 12
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 12);
}

#[test]
fn generics_ascription_mismatch_is_error() {
    let src = r#"
#entry main
#indent 4
#target wasm

enum Option<.T>:
    None
    Some <.T>

fn main <()->i32> ():
    let x <Option<i32>> Option::Some true
    0
"#;

    compile_err(src);
}

#[test]
fn generics_same_type_param_mismatch_is_error() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn same <.T> <(.T,.T)->i32> (a,b):
    0

fn main <()->i32> ():
    same 1 true
"#;

    compile_err(src);
}

#[test]
fn generics_enum_payload_mismatch_is_error() {
    let src = r#"
#entry main
#indent 4
#target wasm

enum Either<.A,.B>:
    Left <.A>
    Right <.B>

fn main <()->i32> ():
    let e <Either<i32,bool>> Either::Left true
    0
"#;

    compile_err(src);
}

#[test]
fn generics_nested_apply_payload_mismatch_is_error() {
    let src = r#"
#entry main
#indent 4
#target wasm

enum Option<.T>:
    None
    Some <.T>

enum Wrap<.T>:
    Wrap <Option<.T>>

fn main <()->i32> ():
    let w <Wrap<i32>> Wrap::Wrap Option::Some true
    0
"#;

    compile_err(src);
}

#[test]
fn generics_wrong_arg_count_is_error() {
    let src = r#"
#entry main
#indent 4
#target wasm

enum Option<.T>:
    None
    Some <.T>

fn main <()->i32> ():
    let x <Option<i32,bool>> Option::None
    0
"#;

    compile_err(src);
}
