# namespace と method 呼び出し

enum constructor、trait method、module alias は `::` で呼び出します。どの名前空間の関数を使っているかを明示できます。

neplg2:test
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

trait SizeCode:
    fn size_code <(Self)->i32> (x):
        0

impl SizeCode for i32:
    fn size_code <(i32)->i32> (x):
        if lt x 10 then 1 else 2

fn option_code <(Option<i32>)->i32> (value):
    match value:
        Option::Some inner:
            SizeCode::size_code inner
        Option::None:
            0

fn main <()*>i32> ():
    let small <Option<i32>> Option<i32>::Some 3
    let large <Option<i32>> Option<i32>::Some 30
    let empty <Option<i32>> Option<i32>::None
    let checks:
        checks_new
        |> checks_push assert_eq_i32 1 option_code small
        |> checks_push assert_eq_i32 2 option_code large
        |> checks_push assert_eq_i32 0 option_code empty
    let shown checks_print_report checks
    checks_exit_code shown
```

`Option::Some` のような省略形が使える場面もありますが、型推論が曖昧なときは `Option<i32>::Some` のように具体型を足します。
