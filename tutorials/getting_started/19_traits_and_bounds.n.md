# trait と bound

trait は「その型が持つ能力」を表します。generic 関数に bound を付けると、その能力を持つ型だけを受け取れます。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *
#import "core/math" as *

trait Score:
    fn score %fn Self i32 \x:
        x

impl Score for i32:
    fn score %fn i32 i32 \x:
        x

fn add_score <.T: Score> %fn .T fn i32 i32 \x\bonus:
    add Score::score x bonus

fn main %impure fn void i32 \void:
    let checks:
        checks_new
        |> checks_push assert_eq_i32 15 add_score 10 5
    let shown checks_print_report checks
    checks_exit_code shown
```

stdlib の collection は `HashKey&Copy` や `Ord` のような bound を使います。必要な能力を型に明示することで、暗黙の shallow copy を避けます。
