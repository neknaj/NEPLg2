# if と match

`if` も `match` も値を返す式です。列挙値や literal の分岐は、深い `if` のネストより `match` を優先します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *
#import "core/math" as *

fn grade %fn i32 i32 \score:
    if:
        lt score 60
        then:
            0
        else:
            if:
                lt score 80
                then:
                    1
                else:
                    2

fn escape_code %fn char i32 \c:
    match c:
        '\n':
            10
        '\r':
            13
        '\t':
            9
        _:
            0

fn main %impure fn void i32 \void:
    let checks:
        checks_new
        |> checks_push assert_eq_i32 0 grade 40
        |> checks_push assert_eq_i32 1 grade 70
        |> checks_push assert_eq_i32 2 grade 90
        |> checks_push assert_eq_i32 10 escape_code '\n'
        |> checks_push assert_eq_i32 0 escape_code 'A'
    let shown checks_print_report checks
    checks_exit_code shown
```

`escape_code` のように「同じ値を複数候補と比較する」処理は `match` で書くと、追加漏れや重複が見つけやすくなります。
