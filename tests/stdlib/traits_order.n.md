# trait [順序/じゅんじょ] capability の focused test

## [目的/もくてき]

- `core/traits/eq`
- `core/traits/ord`
- `alloc/collections/vec/sort`

の[接続/せつぞく]が stdlib [側/がわ]で[一貫/いっかん]していることを[確/たし]かめます。

## [何/なに]を[確/たし]かめるか

- `Eq` trait [経由/けいゆ]の[等値比較/とうちひかく]が[基本型/きほんがた]で[使/つか]えること
- `Ord` trait [経由/けいゆ]の[順序比較/じゅんじょひかく]が[基本型/きほんがた]で[使/つか]えること
- `vec/sort` が[局所/きょくしょ] trait ではなく `core/traits/ord` を[使/つか]って[整列/せいれつ]できること

## Eq trait [経由/けいゆ]で[等値比較/とうちひかく]できる

neplg2:test
ret: 1
```neplg2
#entry main
#target core

#import "core/traits/eq" as *
#import "core/math" as *

fn main %fn unit i32 \unit:
    if and eq_by_trait 42 42 ne_by_trait 42 7 then 1 else 0
```

## Ord trait [経由/けいゆ]で[順序比較/じゅんじょひかく]できる

neplg2:test
ret: 1
```neplg2
#entry main
#target core

#import "core/traits/ord" as *
#import "core/math" as *

fn main %fn unit i32 \unit:
    if and ord_lt 2 3 ord_ge 3 3 then 1 else 0
```

## vec sort は core/traits/ord を[使/つか]って[昇順/しょうじゅん]に[整列/せいれつ]する

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
```neplg2
#entry main
#target std

#import "std/test" as *
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/option" as *
#import "core/field" as *
#import "core/math" as *
#import "core/result" as *

fn main %impure fn unit i32 \unit:
    let v0 %Vec i32 unwrap_ok new;
    let v1 %Vec i32 unwrap_ok push v0 4;
    let v2 %Vec i32 unwrap_ok push v1 1;
    let v3 %Vec i32 unwrap_ok push v2 3;
    let v4 %Vec i32 unwrap_ok push v3 2;
    let s sort_quick_ret v4;
    let b0 %bool match get &s 0:
        Option::Some x:
            eq x 1
        Option::None:
            false
    let b1 %bool match get &s 1:
        Option::Some x:
            eq x 2
        Option::None:
            false
    let b2 %bool match get &s 2:
        Option::Some x:
            eq x 3
        Option::None:
            false
    let b3 %bool match get &s 3:
        Option::Some x:
            eq x 4
        Option::None:
            false
    let checks:
        checks_new
        |> checks_push assert b0
        |> checks_push assert b1
        |> checks_push assert b2
        |> checks_push assert b3
    free s;
    let shown checks_print_report checks
    checks_exit_code shown
```
