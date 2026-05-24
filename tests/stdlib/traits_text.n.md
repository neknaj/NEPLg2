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

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"traits_clone_generic_bound\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"clone generic add\" expected=\"14\" actual=\"14\" message=\"\"\n"
```neplg2
#entry main
#target std

#import "core/math" as *
#import "core/traits/copy" as *
#import "std/test" as *

fn clone_add <.T: Clone> %fn .T fn .T fn fn .T fn .T i32 i32 \x\y\f:
    f Clone::clone &x Clone::clone &y

fn add_i32 %fn i32 fn i32 i32 \a\b:
    add a b

fn main %impure fn () i32 \():
    let a %i32 6
    let b %i32 8
    let actual %i32 clone_add a b @add_i32
    let report:
        test_report_new "traits_clone_generic_bound"
        |> test_report_push assert_eq_i32 "clone generic add" 14 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## stringify は[利用者向/りようしゃむ]け[文字列表現/もじれつひょうげん]を[返/かえ]す

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"traits_stringify_basic_values\" count=3 failed=0\nassertion index=0 status=ok kind=str_eq label=\"stringify i32\" expected=\"42\" actual=\"42\" message=\"\"\nassertion index=1 status=ok kind=str_eq label=\"stringify bool\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=str_eq label=\"stringify u8\" expected=\"9\" actual=\"9\" message=\"\"\n"
```neplg2
#entry main
#target std

#import "std/test" as *
#import "core/traits/stringify" as *
#import "core/cast" as *

fn main %impure fn () i32 \():
    let n %u8 cast 9;
    let report:
        test_report_new "traits_stringify_basic_values"
        |> test_report_push assert_str_eq "stringify i32" "42" stringify 42
        |> test_report_push assert_str_eq "stringify bool" "true" stringify true
        |> test_report_push assert_str_eq "stringify u8" "9" stringify n
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## debug_string は `str` を[区別/くべつ]できる[形/かたち]で[表示/ひょうじ]する

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"traits_debug_basic_values\" count=3 failed=0\nassertion index=0 status=ok kind=str_eq label=\"debug str quotes\" expected=\"\\\"abc\\\"\" actual=\"\\\"abc\\\"\" message=\"\"\nassertion index=1 status=ok kind=str_eq label=\"debug i32\" expected=\"5\" actual=\"5\" message=\"\"\nassertion index=2 status=ok kind=str_eq label=\"debug u8\" expected=\"9\" actual=\"9\" message=\"\"\n"
```neplg2
#entry main
#target std

#import "std/test" as *
#import "core/traits/debug" as *
#import "core/cast" as *

fn main %impure fn () i32 \():
    let n %u8 cast 9;
    let report:
        test_report_new "traits_debug_basic_values"
        |> test_report_push assert_str_eq "debug str quotes" "\"abc\"" debug_string "abc"
        |> test_report_push assert_str_eq "debug i32" "5" debug_string 5
        |> test_report_push assert_str_eq "debug u8" "9" debug_string n
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
