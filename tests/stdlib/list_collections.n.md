# tests/list_collections.n.md

## list_reverse_preserves_order

[目的/もくてき]:
- `reverse` が入力 list の node owner を[再利用/さいりよう]し、[逆順/ぎゃくじゅん] list を[返/かえ]すことを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `reverse`
- [逆順/ぎゃくじゅん] list の `get`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"list_reverse_preserves_order\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"reverse first item\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"reverse last item\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/list" as *
#import "alloc/diag/error" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let src_first %List i32:
        unwrap_ok new
        |> push 3 |> uwok
        |> push 2 |> uwok
        |> push 1 |> uwok
    let first_rev %List i32 reverse src_first;
    let first_ok %bool match get &first_rev 0:
        Option::Some x:
            eq x 3
        Option::None:
            false
    free first_rev;
    let src_last %List i32:
        unwrap_ok new
        |> push 3 |> uwok
        |> push 2 |> uwok
        |> push 1 |> uwok
    let last_rev %List i32 reverse src_last;
    let last_ok %bool match get &last_rev 2:
        Option::Some x:
            eq x 1
        Option::None:
            false
    free last_rev;
    let report:
        test_report_new "list_reverse_preserves_order"
        |> test_report_push assert "reverse first item" first_ok
        |> test_report_push assert "reverse last item" last_ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## list_reverse_empty_is_empty

[目的/もくてき]:
- [空/から] list の `reverse` が[確保/かくほ]なしで[空/から] list を[保/たも]つことを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `reverse`
- [空/から] list
- `is_empty`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"list_reverse_empty_is_empty\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"reverse empty remains empty\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/list" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let empty %List i32 unwrap_ok new;
    let rev %List i32 reverse empty;
    let ok %bool is_empty &rev
    free rev
    let report:
        test_report_new "list_reverse_empty_is_empty"
        |> test_report_push assert "reverse empty remains empty" ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## list_map_filter_return_result

[目的/もくてき]:
- `map` / `filter` が `Result` として[成功/せいこう]を[返/かえ]し、[通常/つうじょう]の[変換/へんかん]結果を[確認/かくにん]できることを[確/たし]かめます。

[何/なに]を[確/たし]かめるか:
- `map`
- `filter`
- `Result::Ok`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"list_map_filter_return_result\" count=2 failed=0\nassertion index=0 status=ok kind=bool label=\"map returns transformed item\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"filter returns even items\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/list" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *
#import "std/test" as *

fn inc %fn i32 i32 \x:
    add x 1

fn is_even %fn i32 bool \x:
    eq rem_s x 2 0

fn main %impure fn unit i32 \unit:
    let map_src %List i32:
        unwrap_ok new
        |> push 3 |> uwok
        |> push 2 |> uwok
        |> push 1 |> uwok
    let map_ok %bool match map map_src inc:
        Result::Err e:
            let recovered %List i32 list_transform_error_list e
            free recovered
            false
        Result::Ok mapped:
            let ok %bool match get &mapped 1:
                Option::Some x:
                    eq x 3
                Option::None:
                    false
            free mapped
            ok
    let filter_src %List i32:
        unwrap_ok new
        |> push 4 |> uwok
        |> push 3 |> uwok
        |> push 2 |> uwok
        |> push 1 |> uwok
    let filter_ok %bool match filter filter_src is_even:
        Result::Err e:
            let recovered %List i32 list_transform_error_list e
            free recovered
            false
        Result::Ok filtered:
            let ok %bool eq len &filtered 2
            free filtered
            ok
    let report:
        test_report_new "list_map_filter_return_result"
        |> test_report_push assert "map returns transformed item" map_ok
        |> test_report_push assert "filter returns even items" filter_ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
