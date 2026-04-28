# 値と型

NEPLg2 の基本値は式として扱います。型注釈は必要な場所だけに置き、読みやすさと型推論の確認に使います。

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let n <i32> 40
    let ok <bool> true
    let text <str> "nepl"
    let unit <()> ()
    let checks:
        checks_new
        |> checks_push check_eq_i32 42 add n 2
        |> checks_push check ok
        |> checks_push check_str_eq "nepl" text
    checks_exit_code checks
```

`str` は UTF-8 text の値です。byte 数と char 数は同じとは限らないため、文字列の詳しい扱いは Part 3 で分けて扱います。
