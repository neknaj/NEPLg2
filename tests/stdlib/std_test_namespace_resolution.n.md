# tests/std_test_namespace_resolution.n.md

## std_test_report_survives_fenwick_add_import

[目的/もくてき]:
- `Fenwick.add` のように collection facade が `add` という public API を[持/も]っていても、`std/test` と `std/stdio` の[内部/ないぶ] arithmetic がその[名前/なまえ]に[汚染/おせん]されないことを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `std/test` report の `count` 加算が `core/math::add` として解決されること。
- stdout write path の iovec offset 加算が `core/math::add` として解決されること。
- 利用者が Fenwick と `std/test` を同じ doctest で使っても canonical report を出せること。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"std_test_report_survives_fenwick_add_import\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"fenwick len after add\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"std/test report unaffected\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/fenwick" as fw
#import "alloc/diag/error" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let tree %Fenwick:
        unwrap_ok fw::new 2
        |> fw::add 0 1 |> uwok
    let size %i32 fw::len &tree
    fw::free tree
    let report:
        test_report_new "std_test_report_survives_fenwick_add_import"
        |> test_report_push assert_eq_i32 "fenwick len after add" 2 size
        |> test_report_push assert "std/test report unaffected" true
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
