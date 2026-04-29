# hash collection rehash

`HashMap` / `HashSet` が[初期容量/しょきようりょう]を[超/こ]えて grow し、tombstone が[増/ふ]えたあとも rehash により[探索/たんさく]できることを[確認/かくにん]します。

## hashmap_grows_past_initial_capacity

[目的/もくてき]:
- `HashMap` が 16 件で[止/と]まらず、insert 前の grow により 40 件を[保持/ほじ]できることを[確認/かくにん]します。

neplg2:test
```neplg2
#target std
#entry main
#indent 4
#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn main <()*>i32> ():
    let mut hm <HashMap<i32,i32,DefaultHash32>> unwrap_ok<HashMap<i32,i32,DefaultHash32>, Diag> new DefaultHash32;
    let mut i <i32> 0;
    while lt i 40:
        do:
            set hm unwrap_ok<HashMap<i32,i32,DefaultHash32>, Diag> insert hm i mul i 10;
            set i add i 1;
    let out <i32> match get &hm 39:
        Option::Some v:
            if eq 390 v 0 2
        Option::None:
            1
    free hm;
    out
```

## hashset_grows_past_initial_capacity

[目的/もくてき]:
- `HashSet` が 16 件で[止/と]まらず、insert 前の grow により 40 件を[保持/ほじ]できることを[確認/かくにん]します。

neplg2:test
```neplg2
#target std
#entry main
#indent 4
#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "core/traits/hash" as *
#import "std/test" as *

fn main <()*>i32> ():
    let mut hs <HashSet<i32,DefaultHash32>> unwrap_ok<HashSet<i32,DefaultHash32>, Diag> new DefaultHash32;
    let mut i <i32> 0;
    while lt i 40:
        do:
            set hs unwrap_ok<HashSet<i32,DefaultHash32>, Diag> insert hs i;
            set i add i 1;
    if contains hs 39 0 1
```

## hashmap_many_inserts_for_runtime_observation

[目的/もくてき]:
- self-host compiler の symbol table を[想定/そうてい]し、100 件を[超/こ]える insert の実行時間を `nodesrc/tests.js` の JSON で[観測/かんそく]できる focused fixture にします。

neplg2:test
```neplg2
#target std
#entry main
#indent 4
#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn main <()*>i32> ():
    let mut hm <HashMap<i32,i32,DefaultHash32>> unwrap_ok<HashMap<i32,i32,DefaultHash32>, Diag> with_capacity DefaultHash32 32;
    let mut i <i32> 0;
    while lt i 160:
        do:
            set hm unwrap_ok<HashMap<i32,i32,DefaultHash32>, Diag> insert hm i add i 1;
            set i add i 1;
    let out <i32> match get &hm 159:
        Option::Some v:
            if eq 160 v 0 2
        Option::None:
            1
    free hm;
    out
```

## hashset_many_inserts_for_runtime_observation

[目的/もくてき]:
- `HashSet` でも 100 件を[超/こ]える insert を focused fixture として[残/のこ]し、grow / rehash 後の membership と実行時間を[確認/かくにん]します。

neplg2:test
```neplg2
#target std
#entry main
#indent 4
#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn main <()*>i32> ():
    let mut hs <HashSet<i32,DefaultHash32>> unwrap_ok<HashSet<i32,DefaultHash32>, Diag> with_capacity DefaultHash32 32;
    let mut i <i32> 0;
    while lt i 160:
        do:
            set hs unwrap_ok<HashSet<i32,DefaultHash32>, Diag> insert hs i;
            set i add i 1;
    if contains hs 159 0 1
```

## hashmap_rehashes_tombstones

[目的/もくてき]:
- remove で tombstone が[増/ふ]えたあと、insert が同容量 rehash を[行/おこな]って probe chain を[短/みじか]く[保/たも]つことを[結果/けっか]で[確認/かくにん]します。

neplg2:test
```neplg2
#target std
#entry main
#indent 4
#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/traits/hash" as *

fn hashmap_after_tombstones <()*>HashMap<i32,i32,DefaultHash32>> ():
    let mut hm <HashMap<i32,i32,DefaultHash32>> unwrap_ok<HashMap<i32,i32,DefaultHash32>, Diag> with_capacity DefaultHash32 8;
    let mut i <i32> 0;
    while lt i 6:
        do:
            set hm unwrap_ok<HashMap<i32,i32,DefaultHash32>, Diag> insert hm i add i 100;
            set i add i 1;
    let mut r <i32> 0;
    while lt r 5:
        do:
            set hm unwrap_ok<HashMap<i32,i32,DefaultHash32>, Diag> remove hm r;
            set r add r 1;
    set hm unwrap_ok<HashMap<i32,i32,DefaultHash32>, Diag> insert hm 100 1000;
    hm

fn main <()*>i32> ():
    let hm5 <HashMap<i32,i32,DefaultHash32>> hashmap_after_tombstones;
    let out5 <i32> match get &hm5 5:
        Option::Some v5:
            if eq 105 v5 0 3
        Option::None:
            2
    free hm5;
    if:
        ne out5 0
        then out5
        else:
            let hm100 <HashMap<i32,i32,DefaultHash32>> hashmap_after_tombstones;
            let out100 <i32> match get &hm100 100:
                Option::Some v100:
                    if eq 1000 v100 0 4
                Option::None:
                    1
            free hm100;
            out100
```

## hashset_rehashes_tombstones

[目的/もくてき]:
- remove で tombstone が[増/ふ]えたあと、insert が同容量 rehash を[行/おこな]って membership を[維持/いじ]することを[確認/かくにん]します。

neplg2:test
```neplg2
#target std
#entry main
#indent 4
#import "alloc/collections/hashset" as *
#import "alloc/diag/error" as *
#import "core/math" as *
#import "core/result" as *
#import "core/traits/hash" as *
#import "std/test" as *

fn hashset_after_tombstones <()*>HashSet<i32,DefaultHash32>> ():
    let mut hs <HashSet<i32,DefaultHash32>> unwrap_ok<HashSet<i32,DefaultHash32>, Diag> with_capacity DefaultHash32 8;
    let mut i <i32> 0;
    while lt i 6:
        do:
            set hs unwrap_ok<HashSet<i32,DefaultHash32>, Diag> insert hs i;
            set i add i 1;
    let mut r <i32> 0;
    while lt r 5:
        do:
            set hs unwrap_ok<HashSet<i32,DefaultHash32>, Diag> remove hs r;
            set r add r 1;
    set hs unwrap_ok<HashSet<i32,DefaultHash32>, Diag> insert hs 100;
    hs

fn main <()*>i32> ():
    let hs5 <HashSet<i32,DefaultHash32>> hashset_after_tombstones;
    let hs100 <HashSet<i32,DefaultHash32>> hashset_after_tombstones;
    if and (contains hs5 5) (contains hs100 100) 0 1
```
