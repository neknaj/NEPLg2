# generics.rs 由来の doctest

このファイルは Rust テスト `generics.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## generics_fn_identity_multi_instantiation

neplg2:test
ret: 8
```neplg2

#entry main
#indent 4
#target core
#no_prelude
#import "core/math" as m
#import "core/math" as *

fn id <.T> <(.T)->.T> (x):
    x

fn main <()->i32> ():
    let a <i32> id 7
    let b <bool> id true
    if b:
        m::add a 1
        else:
            a
```

## generics_enum_option_and_match

neplg2:test
ret: 20
```neplg2

#entry main
#indent 4
#target core
#no_prelude

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
```

## generics_struct_pair_construction

neplg2:test
ret: 30
```neplg2

#entry main
#indent 4
#target core
#no_prelude
#import "core/math" as m
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
    m::add take_ab p1 take_ba p2
```

## generics_param_requires_dot

neplg2:test[compile_fail]
diag_codes: parser.type_expr.invalid
```neplg2

#entry main
#indent 4
#target core
#no_prelude

fn id <T> <(T)->T> (x):
    x

fn main <()->i32> ():
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
    Some <T>

fn main <()->i32> ():
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
    a <T>
    b <U>

fn main <()->i32> ():
    0
```

## generics_enum_payload_arithmetic

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core
#no_prelude
#import "core/math" as m
#import "core/math" as *

enum LocalOption<.T>:
    None
    Some <.T>

fn bump <(LocalOption<i32>)->i32> (o):
    match o:
        Some v:
            m::add v 1
        None:
            0

fn main <()->i32> ():
    bump LocalOption::Some 9
```

## generics_multi_type_params_function

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#no_prelude
#import "core/math" as m
#import "core/math" as *

fn first <.A,.B> <(.A,.B)->.A> (a,b):
    a

fn main <()->i32> ():
    let x <i32> first 3 true
    let y <bool> first false 7
    if y:
        m::add x 1
        else:
            x
```

## generics_enum_none_typed_by_ascription

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target core
#no_prelude

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
    let n <Option<i32>> Option::None
    if is_none_i32 n 1 0
```

## generics_make_none_from_context

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target core
#no_prelude

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
```

## generics_generic_calls_generic

neplg2:test
ret: 9
```neplg2

#entry main
#indent 4
#target core
#no_prelude

fn id <.T> <(.T)->.T> (x):
    x

fn wrap <.U> <(.U)->.U> (x):
    id x

fn main <()->i32> ():
    let a <i32> wrap 9
    a
```

## generics_pipe_into_generic

neplg2:test
ret: 7
```neplg2

#entry main
#indent 4
#target core
#no_prelude
#import "core/math" as m
#import "core/math" as *

fn id <.T> <(.T)->.T> (x):
    x

fn main <()->i32> ():
    let a <i32> 5 |> id
    m::add a 2
```

## generics_option_none_inferred_by_param

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target core
#no_prelude

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
```

## generics_pair_inferred_by_param

neplg2:test
ret: 5
```neplg2

#entry main
#indent 4
#target core
#no_prelude

struct Pair<.A,.B>:
    first <.A>
    second <.B>

fn take_ab <(Pair<i32,bool>)->i32> (p):
    5

fn main <()->i32> ():
    take_ab Pair 1 true
```

## generics_make_pair_wrapper

neplg2:test
ret: 30
```neplg2

#entry main
#indent 4
#target core
#no_prelude
#import "core/math" as m
#import "core/math" as *

struct Pair<.A,.B>:
    first <.A>
    second <.B>

fn make_pair <.A,.B> <(.A,.B)->Pair<.A,.B>> (a,b):
    Pair a b

fn take_ab <(Pair<i32,str>)->i32> (p):
    10

fn take_ba <(Pair<str,i32>)->i32> (p):
    20

fn main <()->i32> ():
    let p1 <Pair<i32,str>> Pair 1 "a"
    let p2 <Pair<str,i32>> Pair "b" 2
    m::add take_ab p1 take_ba p2
```

## generics_make_some_wrapper

neplg2:test
ret: 4
```neplg2

#entry main
#indent 4
#target core
#no_prelude
#import "core/math" as m
#import "core/math" as *

enum LocalOption<.T>:
    None
    Some <.T>

fn make_some <.T> <(.T)->LocalOption<.T>> (v):
    LocalOption::Some v

fn main <()->i32> ():
    let a <LocalOption<i32>> make_some 3
    let b <LocalOption<bool>> make_some true
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
    m::add x y
```

## generics_nested_option_match

neplg2:test
ret: 9
```neplg2

#entry main
#indent 4
#target core
#no_prelude

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
    let inner <Option<i32>> Option::Some 9
    let outer <Option<Option<i32>>> Option::Some inner
    unwrap_nested outer 0
```

## generics_enum_two_params_match_payloads

neplg2:test
ret: 7
```neplg2

#entry main
#indent 4
#target core
#no_prelude

enum Either<.A,.B>:
    Left <.A>
    Right <.B>

fn pick <.A,.B> <(.A,.B,bool)->Either<.A,.B>> (a,b,flag):
    if flag:
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
```

## generics_nested_apply_in_payload

neplg2:test
ret: 12
```neplg2

#entry main
#indent 4
#target core
#no_prelude

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
    Some <.T>

fn main <()->i32> ():
    let x <Option<i32>> Option::Some true
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

fn same <.T> <(.T,.T)->i32> (a,b):
    0

fn main <()->i32> ():
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
    Left <.A>
    Right <.B>

fn main <()->i32> ():
    let e <Either<i32,bool>> Either::Left true
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
    Some <.T>

enum Wrap<.T>:
    Wrap <Option<.T>>

fn main <()->i32> ():
    let w <Wrap<i32>> Wrap::Wrap Option::Some true
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
    Some <.T>

fn main <()->i32> ():
    let x <Option<i32,bool>> Option::None
    0
```
