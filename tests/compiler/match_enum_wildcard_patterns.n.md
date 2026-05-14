# match enum wildcard patterns

enum scrutinee の `match` で `_` wildcard arm が default 分岐として扱われることを確認します。

## enum_wildcard_arm_selects_default_variant

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"enum_wildcard_arm_selects_default_variant\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"wildcard default variant\" expected=\"20\" actual=\"20\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "std/test" as *

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

fn main <()*>i32> ():
    let actual <i32> classify ItemKind::Struct
    let report:
        test_report_new "enum_wildcard_arm_selects_default_variant"
        |> test_report_push assert_eq_i32 "wildcard default variant" 20 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## enum_wildcard_arm_allows_payload_default

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"enum_wildcard_arm_allows_payload_default\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"wildcard payload default\" expected=\"99\" actual=\"99\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "std/test" as *

enum LocalOutcome:
    Value <i32>
    Missing

fn unwrap_or_default <(LocalOutcome)->i32> (result):
    match result:
        Value value:
            value
        _:
            99

fn main <()*>i32> ():
    let actual <i32> unwrap_or_default LocalOutcome::Missing
    let report:
        test_report_new "enum_wildcard_arm_allows_payload_default"
        |> test_report_push assert_eq_i32 "wildcard payload default" 99 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
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
