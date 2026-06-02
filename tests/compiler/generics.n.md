# generics.rs 由来の doctest

このファイルは Rust テスト `generics.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## generics_fn_identity_multi_instantiation

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_fn_identity_multi_instantiation\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic identity multi instantiation\" expected=\"8\" actual=\"8\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "core/math" as m
#import "core/math" as *
#import "std/test" as *

fn id <.T> %fn .T .T \x:
    x

fn main %impure fn void i32 \void:
    let a %i32 id 7
    let b %bool id true
    let actual %i32 if b:
        m::add a 1
        else:
            a
    let report:
        test_report_new "generics_fn_identity_multi_instantiation"
        |> test_report_push assert_eq_i32 "generic identity multi instantiation" 8 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_enum_option_and_match

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_enum_option_and_match\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic enum option match\" expected=\"20\" actual=\"20\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "std/test" as *

enum LocalOption<.T>:
    None
    Some %.T

fn is_some <.T> %fn LocalOption .T bool \o:
    match o:
        Some v:
            true
        None:
            false

fn main %impure fn void i32 \void:
    let a %LocalOption i32 LocalOption::Some 5
    let b %LocalOption bool LocalOption::None
    let _nested %LocalOption LocalOption i32 LocalOption::Some LocalOption::Some 1
    let x %bool is_some a
    let y %bool is_some b
    let actual %i32 if:
        cond:
            x
        then:
            if y 10 20
        else:
            30
    let report:
        test_report_new "generics_enum_option_and_match"
        |> test_report_push assert_eq_i32 "generic enum option match" 20 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_struct_pair_construction

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_struct_pair_construction\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic struct pair construction\" expected=\"30\" actual=\"30\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "core/math" as m
#import "core/math" as *
#import "std/test" as *

struct Pair<.A,.B>:
    first %.A
    second %.B

fn take_ab %fn Pair i32 bool i32 \p:
    10

fn take_ba %fn Pair bool i32 i32 \p:
    20

fn main %impure fn void i32 \void:
    let p1 %Pair i32 bool Pair 1 true
    let p2 %Pair bool i32 Pair false 2
    let actual %i32 m::add take_ab p1 take_ba p2
    let report:
        test_report_new "generics_struct_pair_construction"
        |> test_report_push assert_eq_i32 "generic struct pair construction" 30 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_param_requires_dot

neplg2:test[compile_fail]
diag_codes: parser.type_expr.invalid
```neplg2

#entry main
#indent 4
#target core
#no_prelude

fn id %T %fn T T \x:
    x

fn main %fn void i32 \void:
    0
```

## generics_enum_param_requires_dot

neplg2:test[compile_fail]
diag_codes: parser.type_expr.invalid
```neplg2

#entry main
#indent 4
#target core
#no_prelude

enum Option<T>:
    None
    Some %T

fn main %fn void i32 \void:
    0
```

## generics_struct_param_requires_dot

neplg2:test[compile_fail]
diag_codes: parser.type_expr.invalid
```neplg2

#entry main
#indent 4
#target core
#no_prelude

struct Pair<T,U>:
    a %T
    b %U

fn main %fn void i32 \void:
    0
```

## generics_enum_payload_arithmetic

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_enum_payload_arithmetic\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic enum payload arithmetic\" expected=\"10\" actual=\"10\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "core/math" as m
#import "core/math" as *
#import "std/test" as *

enum LocalOption<.T>:
    None
    Some %.T

fn bump %fn LocalOption i32 i32 \o:
    match o:
        Some v:
            m::add v 1
        None:
            0

fn main %impure fn void i32 \void:
    let actual %i32 bump LocalOption::Some 9
    let report:
        test_report_new "generics_enum_payload_arithmetic"
        |> test_report_push assert_eq_i32 "generic enum payload arithmetic" 10 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_multi_type_params_function

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_multi_type_params_function\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic multi type params function\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "core/math" as m
#import "core/math" as *
#import "std/test" as *

fn first <.A,.B> %fn .A fn .B .A \a\b:
    a

fn main %impure fn void i32 \void:
    let x %i32 first 3 true
    let y %bool first false 7
    let actual %i32 if y:
        m::add x 1
        else:
            x
    let report:
        test_report_new "generics_multi_type_params_function"
        |> test_report_push assert_eq_i32 "generic multi type params function" 3 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_enum_none_typed_by_ascription

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_enum_none_typed_by_ascription\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic enum none typed by ascription\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "std/test" as *

enum LocalOption<.T>:
    None
    Some %.T

fn is_none_i32 %fn LocalOption i32 bool \o:
    match o:
        None:
            true
        Some v:
            false

fn main %impure fn void i32 \void:
    let n %LocalOption i32 LocalOption::None
    let actual %i32 if is_none_i32 n 1 0
    let report:
        test_report_new "generics_enum_none_typed_by_ascription"
        |> test_report_push assert_eq_i32 "generic enum none typed by ascription" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_make_none_from_context

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_make_none_from_context\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic none from context\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "std/test" as *

enum LocalOption<.T>:
    None
    Some %.T

fn make_none <.T> %fn void LocalOption .T \void:
    LocalOption::None

fn main %impure fn void i32 \void:
    let x %LocalOption i32 make_none
    let actual %i32 match x:
        None:
            1
        Some v:
            0
    let report:
        test_report_new "generics_make_none_from_context"
        |> test_report_push assert_eq_i32 "generic none from context" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_generic_calls_generic

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_generic_calls_generic\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic calls generic\" expected=\"9\" actual=\"9\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "std/test" as *

fn id <.T> %fn .T .T \x:
    x

fn wrap <.U> %fn .U .U \x:
    id x

fn main %impure fn void i32 \void:
    let a %i32 wrap 9
    let actual %i32 a
    let report:
        test_report_new "generics_generic_calls_generic"
        |> test_report_push assert_eq_i32 "generic calls generic" 9 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_pipe_into_generic

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_pipe_into_generic\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"pipe into generic\" expected=\"7\" actual=\"7\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "core/math" as m
#import "core/math" as *
#import "std/test" as *

fn id <.T> %fn .T .T \x:
    x

fn main %impure fn void i32 \void:
    let a %i32 5 |> id
    let actual %i32 m::add a 2
    let report:
        test_report_new "generics_pipe_into_generic"
        |> test_report_push assert_eq_i32 "pipe into generic" 7 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_option_none_inferred_by_param

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_option_none_inferred_by_param\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"option none inferred by param\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "std/test" as *

enum LocalOption<.T>:
    None
    Some %.T

fn is_none_i32 %fn LocalOption i32 bool \o:
    match o:
        None:
            true
        Some v:
            false

fn main %impure fn void i32 \void:
    let actual %i32 if is_none_i32 LocalOption::None 1 0
    let report:
        test_report_new "generics_option_none_inferred_by_param"
        |> test_report_push assert_eq_i32 "option none inferred by param" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_pair_inferred_by_param

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_pair_inferred_by_param\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic pair inferred by param\" expected=\"5\" actual=\"5\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "std/test" as *

struct Pair<.A,.B>:
    first %.A
    second %.B

fn take_ab %fn Pair i32 bool i32 \p:
    5

fn main %impure fn void i32 \void:
    let actual %i32 take_ab Pair 1 true
    let report:
        test_report_new "generics_pair_inferred_by_param"
        |> test_report_push assert_eq_i32 "generic pair inferred by param" 5 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_make_pair_wrapper

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_make_pair_wrapper\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic make pair wrapper\" expected=\"30\" actual=\"30\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "core/math" as m
#import "core/math" as *
#import "std/test" as *

struct Pair<.A,.B>:
    first %.A
    second %.B

fn make_pair <.A,.B> %fn .A fn .B Pair .A .B \a\b:
    Pair a b

fn take_ab %fn Pair i32 str i32 \p:
    10

fn take_ba %fn Pair str i32 i32 \p:
    20

fn main %impure fn void i32 \void:
    let p1 %Pair i32 str Pair 1 "a"
    let p2 %Pair str i32 Pair "b" 2
    let actual %i32 m::add take_ab p1 take_ba p2
    let report:
        test_report_new "generics_make_pair_wrapper"
        |> test_report_push assert_eq_i32 "generic make pair wrapper" 30 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_make_some_wrapper

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_make_some_wrapper\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic make some wrapper\" expected=\"4\" actual=\"4\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "core/math" as m
#import "core/math" as *
#import "std/test" as *

enum LocalOption<.T>:
    None
    Some %.T

fn make_some <.T> %fn .T LocalOption .T \v:
    LocalOption::Some v

fn main %impure fn void i32 \void:
    let a %LocalOption i32 make_some 3
    let b %LocalOption bool make_some true
    let x %i32 match a:
        Some v:
            v
        None:
            0
    let y %i32 match b:
        Some flag:
            if flag 1 0
        None:
            0
    let actual %i32 m::add x y
    let report:
        test_report_new "generics_make_some_wrapper"
        |> test_report_push assert_eq_i32 "generic make some wrapper" 4 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_nested_option_match

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_nested_option_match\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic nested option match\" expected=\"9\" actual=\"9\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "std/test" as *

enum LocalOption<.T>:
    None
    Some %.T

fn unwrap_nested <.T> %fn LocalOption LocalOption .T fn .T .T \oo\default:
    match oo:
        Some inner:
            match inner:
                Some v:
                    v
                None:
                    default
        None:
            default

fn main %impure fn void i32 \void:
    let inner %LocalOption i32 LocalOption::Some 9
    let outer %LocalOption LocalOption i32 LocalOption::Some inner
    let actual %i32 unwrap_nested outer 0
    let report:
        test_report_new "generics_nested_option_match"
        |> test_report_push assert_eq_i32 "generic nested option match" 9 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_enum_two_params_match_payloads

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_enum_two_params_match_payloads\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic enum two params match payloads\" expected=\"7\" actual=\"7\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "std/test" as *

enum Either<.A,.B>:
    Left %.A
    Right %.B

fn pick <.A,.B> %fn .A fn .B fn bool Either .A .B \a\b\flag:
    if flag:
        Either::Left a
        else:
            Either::Right b

fn to_i32 %fn Either i32 bool i32 \e:
    match e:
        Left v:
            v
        Right b:
            if b 1 0

fn main %impure fn void i32 \void:
    let e %Either i32 bool pick 7 true true
    let actual %i32 to_i32 e
    let report:
        test_report_new "generics_enum_two_params_match_payloads"
        |> test_report_push assert_eq_i32 "generic enum two params match payloads" 7 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_nested_apply_in_payload

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generics_nested_apply_in_payload\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic nested apply in payload\" expected=\"12\" actual=\"12\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#no_prelude
#import "std/test" as *

enum LocalOption<.T>:
    None
    Some %.T

enum Wrap<.T>:
    Wrap %LocalOption .T

fn unwrap %fn Wrap i32 i32 \w:
    match w:
        Wrap o:
            match o:
                Some v:
                    v
                None:
                    0

fn main %impure fn void i32 \void:
    let actual %i32 unwrap Wrap::Wrap LocalOption::Some 12
    let report:
        test_report_new "generics_nested_apply_in_payload"
        |> test_report_push assert_eq_i32 "generic nested apply in payload" 12 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## generics_ascription_mismatch_is_error

neplg2:test[compile_fail]
diag_codes: type.annotation.mismatch
```neplg2

#entry main
#indent 4
#target core
#no_prelude

enum Option<.T>:
    None
    Some %.T

fn main %fn void i32 \void:
    let x %Option i32 Option::Some true
    0
```

## generics_same_type_param_mismatch_is_error

neplg2:test[compile_fail]
diag_codes: type.overload.no_match, type.return.mismatch
```neplg2

#entry main
#indent 4
#target core
#no_prelude

fn same <.T> %fn .T fn .T i32 \a\b:
    0

fn main %fn void i32 \void:
    same 1 true
```

## generics_enum_payload_mismatch_is_error

neplg2:test[compile_fail]
diag_codes: type.annotation.mismatch
```neplg2

#entry main
#indent 4
#target core
#no_prelude

enum Either<.A,.B>:
    Left %.A
    Right %.B

fn main %fn void i32 \void:
    let e %Either i32 bool Either::Left true
    0
```

## generics_nested_apply_payload_mismatch_is_error

neplg2:test[compile_fail]
diag_codes: type.annotation.mismatch
```neplg2

#entry main
#indent 4
#target core
#no_prelude

enum Option<.T>:
    None
    Some %.T

enum Wrap<.T>:
    Wrap %Option .T

fn main %fn void i32 \void:
    let w %Wrap i32 Wrap::Wrap Option::Some true
    0
```

## generics_wrong_arg_count_is_error

neplg2:test[compile_fail]
diag_codes: type.annotation.mismatch
```neplg2

#entry main
#indent 4
#target core
#no_prelude

enum Option<.T>:
    None
    Some %.T

fn main %fn void i32 \void:
    let x %Option i32 bool Option::None
    0
```
