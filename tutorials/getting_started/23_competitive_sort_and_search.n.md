# sort と[二分探索/にぶんたんさく]の[型/かた]

競プロでは「並べ替えてから数える」パターンが頻出です。
ここではアルゴリズムの理解を優先し、最小の自前実装で `sort` と `lower_bound` を示します。

## 挿入ソート（in-place）の最小実装

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/mem" as *
#import "std/test" as *
#import "core/math" as *

fn insertion_sort_i32 <(i32,i32)*>()> (data, len):
    let mut i <i32> 1;
    while lt i len:
        do:
            let key_off <i32> mul i 4;
            let key_ptr <i32> add data key_off;
            let key <i32> load_i32 key_ptr;
            let mut j <i32> sub i 1;
            let mut done <i32> 0;
            while eq done 0:
                if:
                    lt j 0
                    then:
                        set done 1
                    else:
                        let cur_off <i32> mul j 4;
                        let cur_ptr <i32> add data cur_off;
                        let cur <i32> load_i32 cur_ptr;
                        if lt key cur:
                            then:
                                let nxt <i32> add j 1;
                                let nxt_off <i32> mul nxt 4;
                                let nxt_ptr <i32> add data nxt_off;
                                store_i32 nxt_ptr cur;
                                set j sub j 1;
                            else:
                                set done 1;
            let ins <i32> add j 1;
            let ins_off <i32> mul ins 4;
            let ins_ptr <i32> add data ins_off;
            store_i32 ins_ptr key;
            set i add i 1;
|
fn is_sorted_i32 <(i32,i32)*>bool> (data, len):
    let mut i <i32> 1;
    let mut ok <bool> true;
    while and ok lt i len:
        do:
            let prev_i <i32> sub i 1;
            let prev_off <i32> mul prev_i 4;
            let prev_ptr <i32> add data prev_off;
            let cur_off <i32> mul i 4;
            let cur_ptr <i32> add data cur_off;
            let a <i32> load_i32 prev_ptr;
            let b <i32> load_i32 cur_ptr;
            if lt b a:
                then set ok false
                else ();
            set i add i 1;
    ok
|
fn main <()*>()> ():
    let len <i32> 3;
    let data <i32> alloc mul len 4;
    store_i32 add data 0 5;
    store_i32 add data 4 1;
    store_i32 add data 8 3;
    insertion_sort_i32 data len;
    let ok <bool> is_sorted_i32 data len;
    dealloc data mul len 4;
    assert ok;
    test_checked "insertion sort on buffer"
```

## lower_bound の仕様確認（まずは直線探索で実装）

以下は、昇順配列 `v` に対して「`x` 以上が最初に現れる位置」を返す例です。
実装は理解優先で直線探索にしてあり、後で二分探索へ置き換えできます。

neplg2:test[stdio, normalize_newlines]
stdout: "1 1 4\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "core/mem" as *
#import "std/stdio" as *

fn lower_bound_i32 <(i32,i32,i32)*>i32> (data, len, x):
    let mut j <i32> 0;
    let mut done <i32> 0;
    while eq done 0:
        if:
            ge j len
            then set done 1
            else:
                let cur_off <i32> mul j 4;
                let cur_ptr <i32> add data cur_off;
                let cur <i32> load_i32 cur_ptr;
                if lt cur x:
                    then set j add j 1
                    else set done 1;
    j
|
fn main <()*>()> ():
    let len <i32> 4;
    let data <i32> alloc mul len 4;
    store_i32 add data 0 1;
    store_i32 add data 4 3;
    store_i32 add data 8 3;
    store_i32 add data 12 7;

    print_i32 lower_bound_i32 data len 2;
    print " ";
    print_i32 lower_bound_i32 data len 3;
    print " ";
    println_i32 lower_bound_i32 data len 8;

    dealloc data mul len 4;
```

## 二分探索版 `lower_bound`（本番向け）

本番では `O(log N)` の二分探索版を使います。  
不変条件は「`[0, lo)` は `x` 未満、`[hi, len)` は `x` 以上」です。

neplg2:test[stdio, normalize_newlines]
stdout: "1 1 4\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "core/mem" as *
#import "std/stdio" as *

fn lower_bound_i32_bin <(i32,i32,i32)*>i32> (data, len, x):
    let mut lo <i32> 0;
    let mut hi <i32> len;
    while lt lo hi:
        do:
            let mid <i32> i32_div_s add lo hi 2;
            let mv <i32> load_i32 add data mul mid 4;
            if lt mv x:
                then set lo add mid 1
                else set hi mid;
    lo
|
fn main <()*>()> ():
    let len <i32> 4;
    let data <i32> alloc mul len 4;
    store_i32 add data 0 1;
    store_i32 add data 4 3;
    store_i32 add data 8 3;
    store_i32 add data 12 7;

    print_i32 lower_bound_i32_bin data len 2;
    print " ";
    print_i32 lower_bound_i32_bin data len 3;
    print " ";
    println_i32 lower_bound_i32_bin data len 8;

    dealloc data mul len 4
```
