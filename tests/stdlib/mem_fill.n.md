# core/mem の fill 系テスト

## memset_u8_basic

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let p %i32 alloc_raw 8;
    memset_u8 p 8 65;
    let ok %bool and eq load_u8 add p 0 65 eq load_u8 add p 7 65;
    dealloc_raw p 8;
    if ok 1 0
```

## fill_i32_basic

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let p %i32 alloc_raw 16;
    fill_i32 p 4 99;
    let b0 %bool eq load_i32 add p 0 99;
    let b1 %bool eq load_i32 add p 4 99;
    let b2 %bool eq load_i32 add p 8 99;
    let b3 %bool eq load_i32 add p 12 99;
    let ok %bool and b0 and b1 and b2 b3;
    dealloc_raw p 16;
    if ok 1 0
```

## fill_u8_alias

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let p %i32 alloc_raw 4;
    fill_u8 p 4 7;
    let ok %bool and eq load_u8 add p 1 7 eq load_u8 add p 3 7;
    dealloc_raw p 4;
    if ok 1 0
```
