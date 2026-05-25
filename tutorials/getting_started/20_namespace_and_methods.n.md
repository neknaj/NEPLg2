# namespace と method 呼び出し

enum constructor、trait method、module alias は `::` で呼び出します。どの名前空間の関数を使っているかを明示できます。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
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
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *
#import "core/math" as *

trait SizeCode:
    fn size_code %fn Self i32 \x:
        0

impl SizeCode for i32:
    fn size_code %fn i32 i32 \x:
        if lt x 10 then 1 else 2

fn option_code %fn Option i32 i32 \value:
    match value:
        Option::Some inner:
            SizeCode::size_code inner
        Option::None:
            0

fn main %impure fn () i32 \():
    let small %Option i32 Option::Some 3
    let large %Option i32 Option::Some 30
    let empty %Option i32 Option::None
    let checks:
        checks_new
        |> checks_push assert_eq_i32 1 option_code small
        |> checks_push assert_eq_i32 2 option_code large
        |> checks_push assert_eq_i32 0 option_code empty
    let shown checks_print_report checks
    checks_exit_code shown
```

`Option::Some` のように constructor 名だけで書き、型推論が曖昧なときは周囲の `%Option i32` のような型注釈で具体型を補います。
