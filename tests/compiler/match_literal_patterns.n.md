# match literal patterns

`match` の arm 見出しで整数 literal、bool literal、char literal、`_` wildcard を扱えることを確認します。

## i32_literal_arm_selects_matching_case

neplg2:test
ret: 2
```neplg2
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
```

## i32_literal_arm_uses_wildcard_default

neplg2:test
ret: 3
```neplg2
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
```

## bool_literal_arms_are_exhaustive

neplg2:test
ret: 2
```neplg2
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
```

## i32_duplicate_literal_is_rejected

neplg2:test[compile_fail]
diag_code: type.match.duplicate_arm
```neplg2
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
```

## i32_literal_match_requires_wildcard

neplg2:test[compile_fail]
diag_code: type.match.non_exhaustive
```neplg2
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
```

## wildcard_must_be_last

neplg2:test[compile_fail]
diag_code: type.match.wildcard_not_last
```neplg2
#target wasm
#entry main
#indent 4

fn main <()->i32> ():
    let x <i32> 1
    match x:
        _:
            0
        1:
            1
```

## char_literal_arm_selects_matching_case

neplg2:test
ret: 2
```neplg2
#target wasm
#entry main
#indent 4

fn classify <(char)->i32> (c):
    match c:
        'a':
            1
        '\n':
            2
        _:
            3

fn main <()->i32> ():
    classify '\n'
```

## char_literal_accepts_unicode_scalar

neplg2:test
ret: 1
```neplg2
#target wasm
#entry main
#indent 4

fn main <()->i32> ():
    let c <char> '\u{3042}'
    match c:
        '\u{3042}':
            1
        _:
            0
```

## char_match_rejects_integer_arm

neplg2:test[compile_fail]
diag_code: type.match.pattern_unsupported
```neplg2
#target wasm
#entry main
#indent 4

fn main <()->i32> ():
    let c <char> 'A'
    match c:
        65:
            1
        _:
            0
```

## char_literal_arm_matches_i32_code_point

neplg2:test
ret: 1
```neplg2
#target wasm
#entry main
#indent 4

fn main <()->i32> ():
    let x <i32> 65
    match x:
        'A':
            1
        _:
            0
```

## char_literal_arm_matches_u8_code_point

neplg2:test
ret: 1
```neplg2
#target wasm
#entry main
#indent 4

fn classify <(u8)->i32> (x):
    match x:
        '\n':
            1
        _:
            0

fn main <()->i32> ():
    classify '\n'
```

## char_literal_function_argument_uses_integer_context

neplg2:test
ret: 65
```neplg2
#target wasm
#entry main
#indent 4

fn takes_i32 <(i32)->i32> (x):
    x

fn main <()->i32> ():
    takes_i32 'A'
```

## char_literal_backspace_and_formfeed_escapes_compile

neplg2:test
ret: 20
```neplg2
#target wasm
#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    let b <i32> '\b'
    let f <i32> '\f'
    add b f
```
