# intrinsic の直接テスト

`#intrinsic` の `size_of/align_of/load/store` が i64/f64/unit で正しく動くことを確認する。

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

fn main <()->i32> ():
    let s_i64 <i32> size_of<i64>;
    let a_i64 <i32> align_of<i64>;
    let s_f64 <i32> size_of<f64>;
    let a_f64 <i32> align_of<f64>;
    if:
        and eq s_i64 8 and eq a_i64 8 and eq s_f64 8 eq a_f64 8
        then:
            0
        else:
            1
```

## intrinsic_load_store_i64

neplg2:test
ret: 0
```neplg2
#target core
#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *
#import "core/mem" as *

fn main <()->i32> ():
    let p <i32> alloc_raw 8;
    let v <i64> add <i64> cast 12345 <i64> cast 67890;
    store<i64> p v;
    let got <i64> load<i64> p;
    dealloc_raw p 8;
    if eq got v 0 1
```

## intrinsic_load_store_f64

neplg2:test
ret: 0
```neplg2
#target core
#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *
#import "core/mem" as *

fn main <()->i32> ():
    let p <i32> alloc_raw 8;
    let v <f64> cast 42;
    store<f64> p v;
    let got <f64> load<f64> p;
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

fn main <()->i32> ():
    let r <Result<(),str>> Result<(),str>::Ok ();
    match r:
        Result::Ok _u:
            0
        Result::Err _e:
            1
```

## intrinsic_store_load_enum_i64_payload_uses_full_storage

neplg2:test
ret: 0
```neplg2
#target core
#entry main
#indent 4
#import "core/cast" as *
#import "core/math" as *
#import "core/mem" as *
#import "core/result" as *

fn main <()->i32> ():
    let high <i64> mul <i64> cast 65536 <i64> cast 65536;
    let v <i64> add high <i64> cast 7;
    let r <Result<(),i64>> Result<(),i64>::Err v;
    let p <i32> alloc_raw size_of<Result<(),i64>>;
    store<Result<(),i64>> p r;
    let got <Result<(),i64>> load<Result<(),i64>> p;
    dealloc_raw p size_of<Result<(),i64>>;
    match got:
        Result::Ok _u:
            1
        Result::Err e:
            if eq e v 0 2
```

## intrinsic_zero_sized_struct_constructor_keeps_heap_pointer

neplg2:test
ret: 0
```neplg2
#target core
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *

struct Z:
    tag <()>

fn main <()->i32> ():
    let p0 <i32> alloc_raw 16;
    store_i32 p0 123;
    let z <Z> Z;
    let p1 <i32> alloc_raw 16;
    let kept <bool> eq load_i32 p0 123;
    let moved <bool> gt p1 p0;
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

fn main <()->i32> ():
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

struct Pair:
    a <i32>
    b <str>

fn main <()->i32> ():
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
