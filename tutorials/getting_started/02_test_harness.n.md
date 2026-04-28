# std/test の基本

複数の確認を 1 つの `main` で続けて実行するときは、`std/test` の `checks_new` と `checks_push` を使います。失敗は `Result<(),str>` として集め、最後に `checks_exit_code` で runner 用の終了 code に変換します。

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 3 add 1 2
        |> checks_push check_str_eq "hello" "hello"
        |> checks_push check true
    checks_exit_code checks
```

`checks_exit_code` は stdout を自動では汚しません。実行結果を表示したいときだけ `checks_print_report` を明示的に呼びます。
