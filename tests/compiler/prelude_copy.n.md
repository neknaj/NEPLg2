# prelude で `Copy` / `Clone` capability を供給する focused test

## [目的/もくてき]

- loader が root module に[対/たい]して `std/prelude_base` を[既定/きてい]で[読/よ]み[込/こ]み、`Copy` / `Clone` impl を source [側/がわ]から[供給/きょうきゅう]できることを[確/たし]かめます。
- `#prelude` と `#no_prelude` の[組/く]み[合/あ]わせでも、[明示的/めいじてき] prelude が[優先/ゆうせん]されることを[固定/こてい]します。

## [何/なに]を[確/たし]かめるか

- `#import "core/traits/copy"` を[書/か]かなくても `.T: Copy` を[使/つか]った generic function が[通/とお]ること
- `#prelude std/prelude_base` と `#no_prelude` を[併記/へいき]しても、[明示的/めいじてき] prelude の[効果/こうか]が[残/のこ]ること

## default_prelude_supplies_copy_impls

neplg2:test
ret: 7
```neplg2
#entry main
#target core
#indent 4
#import "core/math" as *

fn clone_left <.T: Copy> <(.T, (.T)->i32)->i32> (x, f):
    f x

fn as_i32 <(i32)->i32> (x):
    x

fn main <()->i32> ():
    clone_left 7 @as_i32
```

## explicit_prelude_survives_no_prelude

neplg2:test
ret: 11
```neplg2
#entry main
#target core
#indent 4
#prelude std/prelude_base
#no_prelude

fn clone_left <.T: Copy> <(.T, (.T)->i32)->i32> (x, f):
    f x

fn as_i32 <(i32)->i32> (x):
    x

fn main <()->i32> ():
    clone_left 11 @as_i32
```

## no_prelude_disables_copy_trait_supply

neplg2:test[compile_fail]
diag_code: type.trait_bound.unknown
```neplg2
#entry main
#target core
#indent 4
#no_prelude

fn clone_left <.T: Copy> <(.T, (.T)->i32)->i32> (x, f):
    f x

fn as_i32 <(i32)->i32> (x):
    x

fn main <()->i32> ():
    clone_left 3 @as_i32
```

## generic_mem_ptr_copy_impl

[目的/もくてき]

- generic impl で[定義/ていぎ]した `Copy` capability が `MemPtr<.T>` の[具体化/ぐたいか]にも[適用/てきよう]され、move [検査/けんさ]が[不必要/ふひつよう]に[失敗/しっぱい]しないことを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか

- `impl<.T> Copy for MemPtr<.T>` が prelude [経由/けいゆ]で[読/よ]み[込/こ]まれること
- `MemPtr<i32>` を 2 [回/かい][読/よ]んでも moved [扱/あつか]いにならないこと

neplg2:test
ret: 1
```neplg2
#entry main
#target std
#indent 4
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/math" as *

fn main <()->i32> ():
    let p mem_ptr_wrap<i32> 32;
    let a mem_ptr_addr p;
    let b mem_ptr_addr p;
    if eq add a b 64 1 0
```
