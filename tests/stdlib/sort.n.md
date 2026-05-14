# sort.nepl のテスト

## sort_quick_i32_basic

neplg2:test
ret: 1234
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/math" as *
#import "core/result" as *

fn make_vec4 <()->Vec<i32>> ():
    let mut v <Vec<i32>> unwrap_ok new<i32>;
    set v unwrap_ok push<i32> v 4;
    set v unwrap_ok push<i32> v 1;
    set v unwrap_ok push<i32> v 3;
    set v unwrap_ok push<i32> v 2;
    v

fn main <()*>i32> ():
    let v <Vec<i32>> make_vec4;
    sort_quick<i32> &v;
    free<i32> v;
    1234
```

## sort_in_place_requires_impure_context

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *

fn pure_sort <(&Vec<i32>)->()> (v):
    sort<i32> v

fn main <()->i32> ():
    0
```

## sort_merge_i32_basic

neplg2:test
ret: 1234
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/math" as *
#import "core/result" as *

fn make_vec4 <()->Vec<i32>> ():
    let mut v <Vec<i32>> unwrap_ok new<i32>;
    set v unwrap_ok push<i32> v 4;
    set v unwrap_ok push<i32> v 1;
    set v unwrap_ok push<i32> v 3;
    set v unwrap_ok push<i32> v 2;
    v

fn main <()*>i32> ():
    let v <Vec<i32>> make_vec4;
    match sort_merge<i32> &v:
        Result::Ok _:
            free<i32> v
            1234
        Result::Err _:
            free<i32> v
            0
```

## sort_quick_ret_i32_sorted_values

neplg2:test
ret: 1334
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/field" as *
#import "core/mem" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn make_vec4 <()->Vec<i32>> ():
    let mut v <Vec<i32>> unwrap_ok new<i32>;
    set v unwrap_ok push<i32> v 4;
    set v unwrap_ok push<i32> v 1;
    set v unwrap_ok push<i32> v 3;
    set v unwrap_ok push<i32> v 2;
    v

fn main <()*>i32> ():
    let v sort_quick_ret<i32> make_vec4;
    let n <i32> len<i32> &v;
    let bn <bool> eq n 4;
    let b0 <bool> match get<i32> &v 0:
        Option::Some x:
            eq x 1
        Option::None:
            false
    let b1 <bool> match get<i32> &v 1:
        Option::Some x:
            eq x 2
        Option::None:
            false
    let b2 <bool> match get<i32> &v 2:
        Option::Some x:
            eq x 3
        Option::None:
            false
    let b3 <bool> match get<i32> &v 3:
        Option::Some x:
            eq x 4
        Option::None:
            false
    let ok <bool> and bn and b0 and b1 and b2 b3;
    free<i32> v;
    if ok 1334 0
```

## sort_quick_ret_len0_noop

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 sort_quick_ret<i32> v0;
    let n <i32> len<i32> &v1;
    free<i32> v1;
    n
```

## sort_quick_ret_len1_noop

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 <Vec<i32>> unwrap_ok push<i32> v0 42;
    let v2 sort_quick_ret<i32> v1;
    let n <i32> len<i32> &v2;
    free<i32> v2;
    n
```

## sort_quick_ret_vec_is_reusable_after_sort

neplg2:test
ret: 3
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 <Vec<i32>> unwrap_ok push<i32> v0 4;
    let v2 <Vec<i32>> unwrap_ok push<i32> v1 1;
    let v3 sort_quick_ret<i32> v2;
    let v4 <Vec<i32>> unwrap_ok push<i32> v3 5;
    let n <i32> len<i32> &v4;
    free<i32> v4;
    n
```

## sort_heap_i32_basic

neplg2:test
ret: 1234
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/math" as *
#import "core/result" as *

fn make_vec4 <()->Vec<i32>> ():
    let mut v <Vec<i32>> unwrap_ok new<i32>;
    set v unwrap_ok push<i32> v 4;
    set v unwrap_ok push<i32> v 1;
    set v unwrap_ok push<i32> v 3;
    set v unwrap_ok push<i32> v 2;
    v

fn main <()*>i32> ():
    let v <Vec<i32>> make_vec4;
    sort_heap<i32> &v;
    free<i32> v;
    1234
```

## sort_heap_ret_i32_sorted_values

neplg2:test
ret: 1434
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/field" as *
#import "core/mem" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn make_vec4 <()->Vec<i32>> ():
    let mut v <Vec<i32>> unwrap_ok new<i32>;
    set v unwrap_ok push<i32> v 4;
    set v unwrap_ok push<i32> v 1;
    set v unwrap_ok push<i32> v 3;
    set v unwrap_ok push<i32> v 2;
    v

fn main <()*>i32> ():
    let v sort_heap_ret<i32> make_vec4;
    let n <i32> len<i32> &v;
    let bn <bool> eq n 4;
    let b0 <bool> match get<i32> &v 0:
        Option::Some x:
            eq x 1
        Option::None:
            false
    let b1 <bool> match get<i32> &v 1:
        Option::Some x:
            eq x 2
        Option::None:
            false
    let b2 <bool> match get<i32> &v 2:
        Option::Some x:
            eq x 3
        Option::None:
            false
    let b3 <bool> match get<i32> &v 3:
        Option::Some x:
            eq x 4
        Option::None:
            false
    let ok <bool> and bn and b0 and b1 and b2 b3;
    free<i32> v;
    if ok 1434 0
```

## sort_heap_ret_len0_noop

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 sort_heap_ret<i32> v0;
    let n <i32> len<i32> &v1;
    free<i32> v1;
    n
```

## sort_heap_ret_len1_noop

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 <Vec<i32>> unwrap_ok push<i32> v0 42;
    let v2 sort_heap_ret<i32> v1;
    let n <i32> len<i32> &v2;
    free<i32> v2;
    n
```

## sort_heap_ret_vec_is_reusable_after_sort

neplg2:test
ret: 3
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 <Vec<i32>> unwrap_ok push<i32> v0 4;
    let v2 <Vec<i32>> unwrap_ok push<i32> v1 1;
    let v3 sort_heap_ret<i32> v2;
    let v4 <Vec<i32>> unwrap_ok push<i32> v3 5;
    let n <i32> len<i32> &v4;
    free<i32> v4;
    n
```

## sort_merge_ret_i32_sorted_values

neplg2:test
ret: 1534
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/field" as *
#import "core/mem" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn make_vec4 <()->Vec<i32>> ():
    let mut v <Vec<i32>> unwrap_ok new<i32>;
    set v unwrap_ok push<i32> v 4;
    set v unwrap_ok push<i32> v 1;
    set v unwrap_ok push<i32> v 3;
    set v unwrap_ok push<i32> v 2;
    v

fn main <()*>i32> ():
    let v <Vec<i32>> unwrap_ok<Vec<i32>, VecSortMergeError<i32>> sort_merge_ret<i32> make_vec4;
    let n <i32> len<i32> &v;
    let bn <bool> eq n 4;
    let b0 <bool> match get<i32> &v 0:
        Option::Some x:
            eq x 1
        Option::None:
            false
    let b1 <bool> match get<i32> &v 1:
        Option::Some x:
            eq x 2
        Option::None:
            false
    let b2 <bool> match get<i32> &v 2:
        Option::Some x:
            eq x 3
        Option::None:
            false
    let b3 <bool> match get<i32> &v 3:
        Option::Some x:
            eq x 4
        Option::None:
            false
    let ok <bool> and bn and b0 and b1 and b2 b3;
    free<i32> v;
    if ok 1534 0
```

## sort_merge_ret_len0_noop

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 <Vec<i32>> unwrap_ok<Vec<i32>, VecSortMergeError<i32>> sort_merge_ret<i32> v0;
    let n <i32> len<i32> &v1;
    free<i32> v1;
    n
```

## sort_merge_ret_len1_noop

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 <Vec<i32>> unwrap_ok push<i32> v0 42;
    let v2 <Vec<i32>> unwrap_ok<Vec<i32>, VecSortMergeError<i32>> sort_merge_ret<i32> v1;
    let n <i32> len<i32> &v2;
    free<i32> v2;
    n
```

## sort_merge_ret_vec_is_reusable_after_sort

neplg2:test
ret: 3
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 <Vec<i32>> unwrap_ok push<i32> v0 4;
    let v2 <Vec<i32>> unwrap_ok push<i32> v1 1;
    let v3 <Vec<i32>> unwrap_ok<Vec<i32>, VecSortMergeError<i32>> sort_merge_ret<i32> v2;
    let v4 <Vec<i32>> unwrap_ok push<i32> v3 5;
    let n <i32> len<i32> &v4;
    free<i32> v4;
    n
```

## sort_merge_ret_error_payload_returns_vec_owner

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/field" as *
#import "core/result" as *

fn main <()*>i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 <Vec<i32>> unwrap_ok push<i32> v0 7;
    let err <VecSortMergeError<i32>> VecSortMergeError<i32> v1 StdErrorKind::OutOfMemory;
    let returned <Vec<i32>> vec_sort_merge_error_vec<i32> err;
    let n <i32> len<i32> &returned;
    free<i32> returned;
    n
```

## sort_default_dispatch_i32

neplg2:test
ret: 1234
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/math" as *
#import "core/result" as *

fn make_vec4 <()->Vec<i32>> ():
    let mut v <Vec<i32>> unwrap_ok new<i32>;
    set v unwrap_ok push<i32> v 4;
    set v unwrap_ok push<i32> v 1;
    set v unwrap_ok push<i32> v 3;
    set v unwrap_ok push<i32> v 2;
    v

fn main <()*>i32> ():
    let v <Vec<i32>> make_vec4;
    sort<i32> &v;
    free<i32> v;
    1234
```

## sort_is_sorted_transition

neplg2:test
ret: 10
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/math" as *
#import "core/result" as *

fn make_vec4 <()->Vec<i32>> ():
    let mut v <Vec<i32>> unwrap_ok new<i32>;
    set v unwrap_ok push<i32> v 4;
    set v unwrap_ok push<i32> v 1;
    set v unwrap_ok push<i32> v 3;
    set v unwrap_ok push<i32> v 2;
    v

fn main <()->i32> ():
    let before_v <Vec<i32>> make_vec4;
    let before <bool> sort_is_sorted<i32> &before_v;
    free<i32> before_v;
    let after_v <Vec<i32>> block:
        let mut v <Vec<i32>> unwrap_ok new<i32>;
        set v unwrap_ok push<i32> v 1;
        set v unwrap_ok push<i32> v 2;
        set v unwrap_ok push<i32> v 3;
        set v unwrap_ok push<i32> v 4;
        v
    let after <bool> sort_is_sorted<i32> &after_v;
    free<i32> after_v;
    if and not before after 10 0
```

## sort_i32 is not exported from safe sort facade

neplg2:test[compile_fail]
diag_codes: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort" as *
#import "core/result" as *

fn main <()*>i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    sort_i32 data_mem_ptr<i32> &v0 0;
    free<i32> v0;
    0
```
