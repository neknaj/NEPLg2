# trait [能力/のうりょく]と[文字列表現/もじれつひょうげん]の focused test

## [目的/もくてき]

- `core/traits/copy`
- `core/traits/stringify`
- `core/traits/debug`

を stdlib [側/がわ]の[標準/ひょうじゅん] trait として[読/よ]み[込/こ]めることを[確/たし]かめます。

## [何/なに]を[確/たし]かめるか

- `Copy` / `Clone` capability が stdlib trait [宣言/せんげん]を[経由/けいゆ]しても[機能/きのう]すること
- `Stringify` が[基本型/きほんがた]の[文字列表現/もじれつひょうげん]を[統一的/とういつてき]に[返/かえ]すこと
- `Debug` が `str` に[引用符/いんようふ]を[付/つ]けるなど、`Stringify` と[区別/くべつ]されること

## stdlib [定義/ていぎ]の Copy / Clone capability を generic bound で[利用/りよう]できる

neplg2:test
ret: 14
```neplg2
#entry main
#target core

#import "core/math" as *
#import "core/traits/copy" as *

fn clone_add <.T: Clone> <(.T,.T,(.T,.T)->i32)->i32> (x, y, f):
    f Clone::clone &x Clone::clone &y

fn add_i32 <(i32,i32)->i32> (a, b):
    add a b

fn main <()->i32> ():
    let a <i32> 6
    let b <i32> 8
    clone_add a b @add_i32
```

## stringify は[利用者向/りようしゃむ]け[文字列表現/もじれつひょうげん]を[返/かえ]す

neplg2:test
ret: 0
```neplg2
#entry main
#target std

#import "std/test" as *
#import "core/traits/stringify" as *
#import "core/cast" as *

fn main <()*>i32> ():
    let n <u8> cast 9;
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push assert_str_eq "42" stringify 42
        |> checks_push assert_str_eq "true" stringify true
        |> checks_push assert_str_eq "9" stringify n
    checks_exit_code checks
```

## debug_string は `str` を[区別/くべつ]できる[形/かたち]で[表示/ひょうじ]する

neplg2:test
ret: 0
```neplg2
#entry main
#target std

#import "std/test" as *
#import "core/traits/debug" as *

fn main <()*>i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push assert_str_eq "\"abc\"" debug_string "abc"
        |> checks_push assert_str_eq "5" debug_string 5
    checks_exit_code checks
```
