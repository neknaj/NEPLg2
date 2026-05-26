# 前置呼び出しと pipe

NEPLg2 の関数呼び出しは前置です。`add 1 2` は「`add` に `1` と `2` を渡す」式です。処理を左から右へ読ませたいときだけ pipe を使います。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok]
    ##: [0] ok
    ##: [1] ok
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *
#import "core/math" as *

fn double %fn i32 i32 \x:
    mul x 2

fn add_five %fn i32 i32 \x:
    add x 5

fn main %impure fn unit i32 \unit:
    let prefix %i32 double add_five 10
    let piped %i32:
        10
        |> add_five
        |> double
    let checks:
        checks_new
        |> checks_push assert_eq_i32 30 prefix
        |> checks_push assert_eq_i32 30 piped
    let shown checks_print_report checks
    checks_exit_code shown
```

pipe は可読性のための道具です。まず前置呼び出しで読めるようになってから、段階的な変換に pipe を使うと混乱しにくくなります。
