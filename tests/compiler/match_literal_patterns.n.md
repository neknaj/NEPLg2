# match literal patterns

`match` の arm 見出しで整数 literal、bool literal、`_` wildcard を扱えることを確認します。

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
diag_id: 3008
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
diag_id: 3009
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
diag_id: 3098
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
