# std/test の集約 API

## std_test_collect_success_summary

[目的/もくてき]:
- `std/test` の collectable な `check_*` を `Vec<Result<(),str>>` へ積み、すべて成功した場合に summary だけが表示されることを確認します。

[何/なに]を[確/たし]かめるか:
- `|> push check_* ...` の形で複数検査を収集できること。
- test case [側/がわ]で `checks_print_report` を[返/かえ]す[直前/ちょくぜん]に[明示的/めいじてき]に[呼/よ]ぶと、summary と human [向/む]け[一覧/いちらん]が stdout に[出/で]ること。
- `checks_exit_code` が 0 を返すこと。

neplg2:test[stdio, normalize_newlines, strip_ansi]
ret: 0
stdout: "Checked [ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 3 add 1 2
        |> checks_push check_str_eq "ab" concat "a" "b"
        |> checks_push check_ok_i32 Result<i32,i32>::Ok 7
        |> checks_push check_err_i32 Result<i32,i32>::Err 5
    let shown <Vec<Result<(),str>>> checks_print_report checks
    checks_exit_code shown
```

## std_test_collect_failure_summary_and_details

[目的/もくてき]:
- 途中に失敗が含まれても後続の `check_*` が継続実行され、最後にまとめて失敗報告されることを確認します。

[何/なに]を[確/たし]かめるか:
- test case [側/がわ]で `checks_print_report` を[返/かえ]す[直前/ちょくぜん]に[呼/よ]ぶと `[ok,err <msg>,...]` 形式の summary と human [向/む]け[一覧/いちらん]が stdout に[出/で]ること。
- human [向/む]け表示で、すべての要素が `[index] ok / err <msg>` として並ぶこと。
- 1 件でも `Err` があれば、`checks_exit_code` が 1 を返すこと。

neplg2:test[stdio, normalize_newlines, strip_ansi]
ret: 1
stdout: "FAIL: [ok,err assert_eq_i32 failed: expected=2 actual=3,ok,err assert_str_eq failed: expected=\"left\" actual=\"right\"]\n[0] ok\n[1] err assert_eq_i32 failed: expected=2 actual=3\n[2] ok\n[3] err assert_str_eq failed: expected=\"left\" actual=\"right\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 3 add 1 2
        |> checks_push check_eq_i32 2 3
        |> checks_push check_err_i32 Result<i32,i32>::Err 5
        |> checks_push check_str_eq "left" "right"
    let shown <Vec<Result<(),str>>> checks_print_report checks
    checks_exit_code shown
```
