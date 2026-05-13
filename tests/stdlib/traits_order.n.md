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

fn main <()->i32> ():
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

fn main <()->i32> ():
    if and ord_lt 2 3 ord_ge 3 3 then 1 else 0
```

## vec sort は core/traits/ord を[使/つか]って[昇順/しょうじゅん]に[整列/せいれつ]する

neplg2:test
```neplg2
#entry main
#target std

#import "std/test" as *
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/option" as *
#import "core/field" as *
#import "core/math" as *

fn main <()*>i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 <Vec<i32>> unwrap_ok push<i32> v0 4;
    let v2 <Vec<i32>> unwrap_ok push<i32> v1 1;
    let v3 <Vec<i32>> unwrap_ok push<i32> v2 3;
    let v4 <Vec<i32>> unwrap_ok push<i32> v3 2;
    let s sort_quick_ret<i32> v4;
    let span <VecDataLen<i32>> data_len<i32> s;
    let data <i32> mem_ptr_addr get span "data";
    let a0 <i32> load_i32 data;
    let a1 <i32> load_i32 add data 4;
    let a2 <i32> load_i32 add data 8;
    let a3 <i32> load_i32 add data 12;
    if and and eq a0 1 eq a1 2 and eq a2 3 eq a3 4 1 0
```
