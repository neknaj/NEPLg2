# intrinsic の直接テスト

`#intrinsic` の `size_of/align_of` と unit payload lowering が通常 source から正しく使えることを確認する。

raw memory の `load/store/alloc/dealloc` は compiler-owned raw-memory boundary の source proof がある場所だけで使える。
このファイルの doctest は通常利用者 source として実行されるため、直接 raw memory operation を呼ぶ例は拒否されることを確認する。
raw load/store の runtime codegen は `nepl-core/tests/intrinsic.rs` の compiler-owned raw boundary harness で検証する。

## internal_mem_ptr_wrap_requires_raw_boundary

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#target std
#entry main
#indent 4
#import "core/mem" as *
#import "core/mem/internal" as *

fn main %impure fn () i32 \():
    let _p %MemPtr i32 mem_ptr_wrap 16
    0
```

## internal_mem_ptr_addr_requires_raw_boundary

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#target std
#entry main
#indent 4
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/result" as *

fn main %impure fn () i32 \():
    match alloc_region<i32> 1:
        Result::Err _e:
            0
        Result::Ok region:
            let p %MemPtr i32 region_ptr &region
            let raw %i32 mem_ptr_addr p
            match dealloc_region<i32> region:
                Result::Err _cleanup:
                    0
                Result::Ok _:
                    raw
```

## intrinsic_size_and_align_direct

neplg2:test
ret: 0
```neplg2
#target core
#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn main %fn () i32 \():
    let s_i64 %i32 size_of<i64>;
    let a_i64 %i32 align_of<i64>;
    let s_f64 %i32 size_of<f64>;
    let a_f64 %i32 align_of<f64>;
    if:
        and eq s_i64 8 and eq a_i64 8 and eq s_f64 8 eq a_f64 8
        then:
            0
        else:
            1
```

## intrinsic_load_store_i64_requires_raw_boundary

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#target core
#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn main %impure fn () i32 \():
    let p %i32 alloc_raw 8;
    let v %i64 add %i64 cast 12345 %i64 cast 67890;
    store<i64> p v;
    let got %i64 load<i64> p;
    dealloc_raw p 8;
    if eq got v 0 1
```

## intrinsic_load_store_f64_requires_raw_boundary

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#target core
#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn main %impure fn () i32 \():
    let p %i32 alloc_raw 8;
    let v %f64 cast 42;
    store<f64> p v;
    let got %f64 load<f64> p;
    dealloc_raw p 8;
    if eq got v 0 1
```

## intrinsic_load_store_unit_no_stack_leak

neplg2:test
ret: 0
```neplg2
#target core
#entry main
#indent 4
#import "core/result" as *

fn main %fn () i32 \():
    let r %Result () str Result<(),str>::Ok ();
    match r:
        Result::Ok _u:
            0
        Result::Err _e:
            1
```

## intrinsic_store_load_enum_i64_payload_requires_raw_boundary

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#target core
#entry main
#indent 4
#import "core/cast" as *
#import "core/math" as *
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn main %impure fn () i32 \():
    let high %i64 mul %i64 cast 65536 %i64 cast 65536;
    let v %i64 add high %i64 cast 7;
    let r %Result () i64 Result<(),i64>::Err v;
    let p %i32 alloc_raw size_of<Result<(),i64>>;
    store<Result<(),i64>> p r;
    let got %Result () i64 load<Result<(),i64>> p;
    dealloc_raw p size_of<Result<(),i64>>;
    match got:
        Result::Ok _u:
            1
        Result::Err e:
            if eq e v 0 2
```

## intrinsic_zero_sized_struct_raw_probe_requires_raw_boundary

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#target core
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct Z:
    tag %()

fn main %impure fn () i32 \():
    let p0 %i32 alloc_raw 16;
    store_i32 p0 123;
    let z %Z Z;
    let p1 %i32 alloc_raw 16;
    let kept %bool eq load_i32 p0 123;
    let moved %bool gt p1 p0;
    dealloc_raw p0 16;
    dealloc_raw p1 16;
    if and kept moved 0 1
```

## intrinsic_argument_type_mismatch_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.intrinsic.arg_type_mismatch
```neplg2
#target core
#entry main
#indent 4

fn main %fn () i32 \():
    #intrinsic "i32_to_f32" <> (true)
    0
```

## intrinsic_size_of_std_layout

neplg2:test
ret: 0
```neplg2
#target std
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *

struct Pair:
    a %i32
    b %str

fn main %fn () i32 \():
    if:
        eq size_of<str> 4
        then:
            if:
                eq size_of<Pair> 8
                then:
                    if eq align_of<Pair> 4 0 3
                else:
                    2
        else:
            1
```
