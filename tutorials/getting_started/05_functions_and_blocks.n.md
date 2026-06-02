# 関数、block、末尾式

関数本体の最後の式が戻り値になります。途中の式を文として終わらせるときは `;` を置きます。

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
#import "core/result" as *
#import "std/test" as *
#import "core/math" as *

fn clamp_0_10 %fn i32 i32 \x:
    if:
        lt x 0
        then:
            0
        else:
            if:
                lt 10 x
                then:
                    10
                else:
                    x

fn score %fn i32 i32 \raw:
    let adjusted %i32 add raw 3
    clamp_0_10 adjusted

fn main %impure fn void i32 \void:
    let checks:
        checks_new
        |> checks_push assert_eq_i32 0 score -10
        |> checks_push assert_eq_i32 8 score 5
        |> checks_push assert_eq_i32 10 score 20
    let shown checks_print_report checks
    checks_exit_code shown
```

`score` の末尾にある `clamp_0_10 adjusted` が戻り値です。計算途中の名前は、意味が増えるときだけ置きます。
