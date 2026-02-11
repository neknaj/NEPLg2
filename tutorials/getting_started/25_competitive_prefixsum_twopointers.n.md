# prefix sum と two pointers

競技プログラミングでは、`O(N^2)` を `O(N)` へ落とす定番が重要です。
この章では「累積和（prefix sum）」と「尺取り法（two pointers）」を最小実装で確認します。

## prefix sum で区間和を高速化する

`sum[l..r] = pref[r] - pref[l]` を使うと、各クエリを `O(1)` で処理できます。

neplg2:test[stdio, normalize_newlines]
stdin: "5 3\n1 2 3 4 5\n1 3\n2 5\n1 5\n"
stdout: "6\n14\n15\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "core/mem" as *
#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*> ()> ():
    let sc <i32> scanner_new;
    let n <i32> scanner_read_i32 sc;
    let q <i32> scanner_read_i32 sc;

    let pref_len <i32> add n 1;
    let pref <i32> alloc mul pref_len 4;
    store_i32 pref 0;

    let mut i <i32> 1;
    while le i n:
        do:
            let a <i32> scanner_read_i32 sc;
            let prev_off <i32> mul sub i 1 4;
            let prev_ptr <i32> add pref prev_off;
            let prev <i32> load_i32 prev_ptr;
            let cur <i32> add prev a;
            let cur_off <i32> mul i 4;
            let cur_ptr <i32> add pref cur_off;
            store_i32 cur_ptr cur;
            set i add i 1;

    let w <i32> writer_new;
    let mut k <i32> 0;
    while lt k q:
        do:
            let l1 <i32> scanner_read_i32 sc;
            let r1 <i32> scanner_read_i32 sc;
            let l <i32> sub l1 1;
            let left_off <i32> mul l 4;
            let right_off <i32> mul r1 4;
            let left <i32> load_i32 add pref left_off;
            let right <i32> load_i32 add pref right_off;
            writer_write_i32 w sub right left;
            writer_writeln w;
            set k add k 1;

    writer_flush w;
    writer_free w;
    dealloc pref mul pref_len 4
```

## two pointers で条件を満たす部分配列数を数える

正の配列で `sum <= S` を満たす部分配列数を `O(N)` で数える例です。

neplg2:test[stdio, normalize_newlines]
stdout: "6\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "core/mem" as *
#import "std/stdio" as *

fn count_subarrays_leq_s <(i32,i32,i32)*>i32> (data, n, s):
    let mut l <i32> 0;
    let mut r <i32> 0;
    let mut sum <i32> 0;
    let mut ans <i32> 0;

    while lt l n:
        do:
            while and lt r n le add sum load_i32 add data mul r 4 s:
                do:
                    set sum add sum load_i32 add data mul r 4;
                    set r add r 1;

            set ans add ans sub r l;

            if lt l r:
                then:
                    set sum sub sum load_i32 add data mul l 4;
                    set l add l 1;
                else:
                    set l add l 1;
                    set r add l 0;
    ans
|
fn main <()*> ()> ():
    let n <i32> 4;
    let data <i32> alloc mul n 4;
    store_i32 add data 0 1;
    store_i32 add data 4 2;
    store_i32 add data 8 3;
    store_i32 add data 12 4;

    let ans <i32> count_subarrays_leq_s data n 5;
    println_i32 ans;
    dealloc data mul n 4
```
