# prefix sum と two pointers

`O(N^2)` を `O(N)` へ落とす定番として、累積和と尺取り法を扱います。  
この章は `Vec` と `std/streamio` / `kp/*` の補助 API を優先して、手書きメモリ操作を減らします。

## prefix sum で区間和を `O(1)` にする

`sum[l..r) = pref[r] - pref[l]` を使います。

neplg2:test[stdio, normalize_newlines]
stdin: "5 3\n1 2 3 4 5\n1 3\n2 5\n1 5\n"
stdout: "6\n14\n15\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/result" as *
#import "std/streamio" as *
#import "std/iotarget" as *
#import "kp/kpprefix" as *
#import "alloc/collections/vec" as *

fn main <()*> ()> ():
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let n <i32> read sc;
    let q <i32> read sc;
    let mut a <Vec<i32>> unwrap_ok new<i32>;
    let mut i <i32> 0;
    while lt i n:
        do:
            set a unwrap_ok push a read sc;
            set i add i 1;
    let pref <PrefixI32> prefix_build_vec_i32 a;
    let mut w <StreamWriter> unwrap_ok open WriteStream::Stdio;
    let mut k <i32> 0;
    while lt k q:
        do:
            let l1 <i32> read sc;
            let r1 <i32> read sc;
            let ans <i32> prefix_sum_i32 pref sub l1 1 r1;
            set w writeln w ans;
            set k add k 1;
    close sc;
    set w flush w;
    close w;
    prefix_free_i32 pref
```

## two pointers で `sum <= S` の部分配列数を数える

正の配列なら、右端を戻さない 2 ポインタで `O(N)` にできます。

neplg2:test[stdio, normalize_newlines]
stdout: "6\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/option" as *
#import "alloc/collections/vec" as *
#import "std/stdio" as *

fn count_subarrays_leq_s <(Vec<i32>,i32)->i32> (a, s):
    let span <VecDataLen<i32>> data_len<i32> a;
    let n <i32> get span "len";
    let data <i32> mem_ptr_addr get span "data";
    let mut l <i32> 0;
    let mut r <i32> 0;
    let mut sum <i32> 0;
    let mut ans <i32> 0;
    while lt l n:
        do:
            let mut can_extend <i32> 1;
            while eq can_extend 1:
                do:
                    if lt r n:
                        then:
                            let rv <i32> load<i32> add data mul r size_of<i32>;
                            if le add sum rv s:
                                then:
                                    set sum add sum rv;
                                    set r add r 1;
                                else set can_extend 0
                        else set can_extend 0
            set ans add ans sub r l;
            if lt l r:
                then set sum sub sum load<i32> add data mul l size_of<i32>
                else set r add l 1
            set l add l 1;
    ans
|
fn main <()*> ()> ():
    let a <Vec<i32>>:
        unwrap_ok new<i32>
        |> push 1 |> uwok
        |> push 2 |> uwok
        |> push 3 |> uwok
        |> push 4 |> uwok
    println_i32 count_subarrays_leq_s a 5
```
