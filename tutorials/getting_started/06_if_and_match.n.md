# if と match

`if` も `match` も値を返す式です。列挙値や literal の分岐は、深い `if` のネストより `match` を優先します。

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *

fn grade <(i32)->i32> (score):
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

fn escape_code <(char)->i32> (c):
    match c:
        '\n':
            10
        '\r':
            13
        '\t':
            9
        _:
            0

fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 0 grade 40
        |> checks_push check_eq_i32 1 grade 70
        |> checks_push check_eq_i32 2 grade 90
        |> checks_push check_eq_i32 10 escape_code '\n'
        |> checks_push check_eq_i32 0 escape_code 'A'
    checks_exit_code checks
```

`escape_code` のように「同じ値を複数候補と比較する」処理は `match` で書くと、追加漏れや重複が見つけやすくなります。
