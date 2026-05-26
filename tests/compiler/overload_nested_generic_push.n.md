# nested generic overload resolution for `push`

このファイルは、`Vec<Result<unit,str>>` のような[入れ子/いれこ]になったジェネリクス型に対して、
`new` / `push` の `Result` [返却/へんきゃく]と overload [解決/かいけつ]が正しく動くことを確認する。

## nested_generic_push_direct

[目的/もくてき]:
- `push v r` という最短の書き方で、`Vec<Result<unit,str>>` に `Result<unit,str>` を追加できることを確認する。
- `push<T>` のような明示型引数に頼らず、引数型から overload が選ばれることを確認する。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"nested_generic_push_direct\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"direct nested generic push length\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let v0 %Vec Result unit str unwrap_ok new<Result<unit,str>>;
    let r %Result unit str Result::Ok unit;
    let v1 %Vec Result unit str uwok push v0 r;
    let n %i32 len<Result<unit,str>> &v1;
    free<Result<unit,str>> v1;
    let actual %i32 if eq n 1 1 0
    let report:
        test_report_new "nested_generic_push_direct"
        |> test_report_push assert_eq_i32 "direct nested generic push length" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## nested_generic_push_pipe

[目的/もくてき]:
- pipe 記法の中でも `push` が同じ overload を選べることを確認する。
- `new<Result<unit,str>> |> push (Result::Ok unit)` のような書き方が、collectable な test API の土台として使えることを確認する。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"nested_generic_push_pipe\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"pipe nested generic push length\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as *
#import "core/result" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let v %Vec Result unit str:
        unwrap_ok new<Result<unit,str>>
        |> push (Result::Ok unit) |> uwok
        |> push (Result::Err "oops") |> uwok
    let n %i32 len<Result<unit,str>> &v;
    free<Result<unit,str>> v;
    let actual %i32 if eq n 2 1 0
    let report:
        test_report_new "nested_generic_push_pipe"
        |> test_report_push assert_eq_i32 "pipe nested generic push length" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
