# capacity_stack

メモリ容量・スタック深さ・複合利用（文字列/enum/vec/再帰）の段階的な回帰テストです。

## stage1_recursive_depth_64

neplg2:test
ret: 64
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn depth <(i32,i32)->i32> (n, acc):
    if le n 0:
        acc
    else:
        depth sub n 1 add acc 1

fn main <()->i32> ():
    depth 64 0
```

## stage2_recursive_depth_512

neplg2:test
ret: 512
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn depth <(i32,i32)->i32> (n, acc):
    if le n 0:
        acc
    else:
        depth sub n 1 add acc 1

fn main <()->i32> ():
    depth 512 0
```

## stage3_vec_growth_4096

neplg2:test
ret: 4096
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "alloc/vec" as *

fn main <()->i32> ():
    let mut v vec_new<i32>;
    let mut i <i32> 0;
    while lt i 4096:
        do:
            set v vec_push<i32> v i;
            set i add i 1;
    vec_len<i32> v
```

## stage4_mem_block_store_load

neplg2:test
ret: 1535
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/mem" as *

fn main <()->i32> ():
    let n <i32> 1024;
    let bytes <i32> mul n 4;
    let p <i32> alloc bytes;

    let mut i <i32> 0;
    while lt i n:
        do:
            let slot <i32> add p mul i 4;
            store_i32 slot i;
            set i add i 1;

    let a <i32> load_i32 p;
    let b <i32> load_i32 add p mul 512 4;
    let c <i32> load_i32 add p mul 1023 4;
    dealloc p bytes;
    add add a b c
```

## stage5_string_builder_len_3000

neplg2:test
ret: 3000
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "alloc/string" as *

fn main <()->i32> ():
    let mut sb <StringBuilder> string_builder_new;
    let mut i <i32> 0;
    while lt i 1500:
        do:
            set sb sb_append sb "ab";
            set i add i 1;
    let out <str> sb_build sb;
    len out
```

## stage6_enum_vec_recursive_mix

neplg2:test
ret: 15
```neplg2
#entry main
#indent 4
#target wasm
#import "core/math" as *
#import "core/option" as *
#import "alloc/vec" as *

enum Kind:
    A
    B

fn depth <(i32)->i32> (n):
    if le n 0:
        0
    else:
        add 1 depth sub n 1

fn main <()->i32> ():
    let mut v vec_new<Kind>;
    set v vec_push<Kind> v Kind::A;
    set v vec_push<Kind> v Kind::B;
    set v vec_push<Kind> v Kind::A;
    set v vec_push<Kind> v Kind::B;
    set v vec_push<Kind> v Kind::A;
    let n <i32> vec_len<Kind> v;
    add n depth 10
```
