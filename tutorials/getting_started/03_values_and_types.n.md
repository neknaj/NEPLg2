# 値と型

NEPLg2 の基本値は式として扱います。型注釈は必要な場所だけに置き、読みやすさと型推論の確認に使います。

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

fn main %impure fn unit i32 \unit:
    let n %i32 40
    let ok %bool true
    let text %str "nepl"
    let unit_value %unit unit
    let checks:
        checks_new
        |> checks_push assert_eq_i32 42 add n 2
        |> checks_push assert ok
        |> checks_push assert_str_eq "nepl" text
    let shown checks_print_report checks
    checks_exit_code shown
```

`str` は UTF-8 text の値です。byte 数と char 数は同じとは限らないため、文字列の詳しい扱いは Part 3 で分けて扱います。
