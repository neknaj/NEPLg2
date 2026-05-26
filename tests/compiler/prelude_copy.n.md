# prelude で `Copy` / `Clone` capability を供給する focused test

## [目的/もくてき]

- loader が root module に[対/たい]して `std/prelude_base` を[既定/きてい]で[読/よ]み[込/こ]み、`Copy` / `Clone` impl を source [側/がわ]から[供給/きょうきゅう]できることを[確/たし]かめます。
- `#prelude` と `#no_prelude` の[組/く]み[合/あ]わせでも、[明示的/めいじてき] prelude が[優先/ゆうせん]されることを[固定/こてい]します。

## [何/なに]を[確/たし]かめるか

- `#import "core/traits/copy"` を[書/か]かなくても `.T: Copy` を[使/つか]った generic function が[通/とお]ること
- `#prelude std/prelude_base` と `#no_prelude` を[併記/へいき]しても、[明示的/めいじてき] prelude の[効果/こうか]が[残/のこ]ること

## default_prelude_supplies_copy_impls

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"default_prelude_supplies_copy_impls\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"default prelude copy clone\" expected=\"7\" actual=\"7\" message=\"\"\n"
```neplg2
#entry main
#target std
#indent 4
#import "core/math" as *
#import "std/test" as *

fn clone_left <.T: Copy> %fn .T fn fn .T i32 i32 \x\f:
    f x

fn as_i32 %fn i32 i32 \x:
    x

fn main %impure fn unit i32 \unit:
    let actual %i32 clone_left 7 @as_i32
    let report:
        test_report_new "default_prelude_supplies_copy_impls"
        |> test_report_push assert_eq_i32 "default prelude copy clone" 7 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## explicit_prelude_survives_no_prelude

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"explicit_prelude_survives_no_prelude\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"explicit prelude copy clone\" expected=\"11\" actual=\"11\" message=\"\"\n"
```neplg2
#entry main
#target std
#indent 4
#prelude std/prelude_base
#no_prelude
#import "std/test" as *

fn clone_left <.T: Copy> %fn .T fn fn .T i32 i32 \x\f:
    f x

fn as_i32 %fn i32 i32 \x:
    x

fn main %impure fn unit i32 \unit:
    let actual %i32 clone_left 11 @as_i32
    let report:
        test_report_new "explicit_prelude_survives_no_prelude"
        |> test_report_push assert_eq_i32 "explicit prelude copy clone" 11 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## no_prelude_disables_copy_trait_supply

neplg2:test[compile_fail]
diag_code: type.trait_bound.unknown
```neplg2
#entry main
#target core
#indent 4
#no_prelude

fn clone_left <.T: Copy> %fn .T fn fn .T i32 i32 \x\f:
    f x

fn as_i32 %fn i32 i32 \x:
    x

fn main %fn unit i32 \unit:
    clone_left 3 @as_i32
```

## generic_mem_ptr_copy_impl

[目的/もくてき]

- generic impl で[定義/ていぎ]した `Copy` capability が `MemPtr<.T>` の[具体化/ぐたいか]にも[適用/てきよう]され、move [検査/けんさ]が[不必要/ふひつよう]に[失敗/しっぱい]しないことを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか

- `impl<.T> Copy for MemPtr<.T>` が prelude [経由/けいゆ]で[読/よ]み[込/こ]まれること
- `MemPtr<i32>` を 2 [回/かい][読/よ]んでも moved [扱/あつか]いにならないこと

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"generic_mem_ptr_copy_impl\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic MemPtr copy address sum\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#target std
#indent 4
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let p mem_ptr_wrap<i32> 32;
    let a mem_ptr_addr p;
    let b mem_ptr_addr p;
    let actual %i32 if eq add a b 64 1 0
    let report:
        test_report_new "generic_mem_ptr_copy_impl"
        |> test_report_push assert_eq_i32 "generic MemPtr copy address sum" 1 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
