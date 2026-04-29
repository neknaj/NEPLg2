# match enum wildcard patterns

enum scrutinee の `match` で `_` wildcard arm が default 分岐として扱われることを確認します。

## enum_wildcard_arm_selects_default_variant

neplg2:test
ret: 20
```neplg2
#target wasm
#entry main
#indent 4

enum ItemKind:
    Import
    Function
    Struct

fn classify <(ItemKind)->i32> (kind):
    match kind:
        Import:
            10
        _:
            20

fn main <()->i32> ():
    classify ItemKind::Struct
```

## enum_wildcard_arm_allows_payload_default

neplg2:test
ret: 99
```neplg2
#target wasm
#entry main
#indent 4

enum Outcome:
    Ok <i32>
    Err

fn unwrap_or_default <(Outcome)->i32> (result):
    match result:
        Ok value:
            value
        _:
            99

fn main <()->i32> ():
    unwrap_or_default Outcome::Err
```

## enum_wildcard_must_be_last

neplg2:test[compile_fail]
diag_code: type.match.wildcard_not_last
```neplg2
#target wasm
#entry main
#indent 4

enum ItemKind:
    Import
    Function

fn main <()->i32> ():
    let kind <ItemKind> ItemKind::Import
    match kind:
        _:
            0
        Import:
            1
```

## enum_duplicate_wildcard_is_rejected

neplg2:test[compile_fail]
diag_code: type.match.duplicate_arm
```neplg2
#target wasm
#entry main
#indent 4

enum ItemKind:
    Import
    Function

fn main <()->i32> ():
    let kind <ItemKind> ItemKind::Import
    match kind:
        _:
            0
        _:
            1
```
