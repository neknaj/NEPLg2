# std/test の基本

複数の確認を 1 つの `main` で続けて実行するときは、`std/test` の `checks_new` と `checks_push` を使います。失敗は `Result<(),str>` として集め、最後に `checks_exit_code` で runner 用の終了 code に変換します。

neplg2:test
ret: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *
#import "core/math" as *

fn main <()*>i32> ():
    let checks:
        checks_new
        |> checks_push assert_eq_i32 3 add 1 2
        |> checks_push assert_str_eq "hello" "hello"
        |> checks_push assert true
    let shown checks_print_report checks
    checks_exit_code shown
```

`checks_exit_code` は stdout を自動では汚しません。tutorial では、読者と CI artifact が確認内容を見られるように `checks_print_report` を明示的に呼びます。
