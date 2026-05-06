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

fn main <()->i32> ():
    let v <Vec<i32>> make_vec4;
    sort_quick<i32> &v;
    free<i32> v;
    1234
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

fn main <()->i32> ():
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

fn main <()->i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 sort_quick_ret<i32> v0;
    let s data_len<i32> &v1;
    let n <i32> get s "len";
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

fn main <()->i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 <Vec<i32>> unwrap_ok push<i32> v0 42;
    let v2 sort_quick_ret<i32> v1;
    let s data_len<i32> &v2;
    let n <i32> get s "len";
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

fn main <()->i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 <Vec<i32>> unwrap_ok push<i32> v0 4;
    let v2 <Vec<i32>> unwrap_ok push<i32> v1 1;
    let v3 sort_quick_ret<i32> v2;
    let v4 <Vec<i32>> unwrap_ok push<i32> v3 5;
    let s data_len<i32> &v4;
    let n <i32> get s "len";
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

fn make_vec4 <()->Vec<i32>> ():
    let mut v <Vec<i32>> unwrap_ok new<i32>;
    set v unwrap_ok push<i32> v 4;
    set v unwrap_ok push<i32> v 1;
    set v unwrap_ok push<i32> v 3;
    set v unwrap_ok push<i32> v 2;
    v

fn main <()->i32> ():
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

fn make_vec4 <()->Vec<i32>> ():
    let mut v <Vec<i32>> unwrap_ok new<i32>;
    set v unwrap_ok push<i32> v 4;
    set v unwrap_ok push<i32> v 1;
    set v unwrap_ok push<i32> v 3;
    set v unwrap_ok push<i32> v 2;
    v

fn main <()->i32> ():
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

fn main <()->i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 sort_heap_ret<i32> v0;
    let s data_len<i32> &v1;
    let n <i32> get s "len";
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

fn main <()->i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 <Vec<i32>> unwrap_ok push<i32> v0 42;
    let v2 sort_heap_ret<i32> v1;
    let s data_len<i32> &v2;
    let n <i32> get s "len";
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

fn main <()->i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>;
    let v1 <Vec<i32>> unwrap_ok push<i32> v0 4;
    let v2 <Vec<i32>> unwrap_ok push<i32> v1 1;
    let v3 sort_heap_ret<i32> v2;
    let v4 <Vec<i32>> unwrap_ok push<i32> v3 5;
    let s data_len<i32> &v4;
    let n <i32> get s "len";
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

fn make_vec4 <()->Vec<i32>> ():
    let mut v <Vec<i32>> unwrap_ok new<i32>;
    set v unwrap_ok push<i32> v 4;
    set v unwrap_ok push<i32> v 1;
    set v unwrap_ok push<i32> v 3;
    set v unwrap_ok push<i32> v 2;
    v

fn main <()*>i32> ():
    let v <Vec<i32>> unwrap_ok<Vec<i32>, StdErrorKind> sort_merge_ret<i32> make_vec4;
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
    let v1 <Vec<i32>> unwrap_ok<Vec<i32>, StdErrorKind> sort_merge_ret<i32> v0;
    let s data_len<i32> &v1;
    let n <i32> get s "len";
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
    let v2 <Vec<i32>> unwrap_ok<Vec<i32>, StdErrorKind> sort_merge_ret<i32> v1;
    let s data_len<i32> &v2;
    let n <i32> get s "len";
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
    let v3 <Vec<i32>> unwrap_ok<Vec<i32>, StdErrorKind> sort_merge_ret<i32> v2;
    let v4 <Vec<i32>> unwrap_ok push<i32> v3 5;
    let s data_len<i32> &v4;
    let n <i32> get s "len";
    free<i32> v4;
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

fn make_vec4 <()->Vec<i32>> ():
    let mut v <Vec<i32>> unwrap_ok new<i32>;
    set v unwrap_ok push<i32> v 4;
    set v unwrap_ok push<i32> v 1;
    set v unwrap_ok push<i32> v 3;
    set v unwrap_ok push<i32> v 2;
    v

fn main <()->i32> ():
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

## sort_i32_ptr_basic

neplg2:test
ret: 1234
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec/sort" as *
#import "core/mem" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn put_i32 <(MemPtr<i32>,i32,i32)->()> (p, off, value):
    unwrap_ok store_i32 mem_ptr_add<i32> p off value

fn read_i32_or_zero <(MemPtr<i32>,i32)->i32> (p, off):
    match load_i32 mem_ptr_add<i32> p off:
        Option::Some value:
            value
        Option::None:
            0

fn main <()->i32> ():
    let p <MemPtr<i32>> unwrap_ok alloc_ptr<i32> 16;
    put_i32 p 0 4;
    put_i32 p 4 1;
    put_i32 p 8 3;
    put_i32 p 12 2;
    sort_i32 p 4;
    let b0 <bool> eq read_i32_or_zero p 0 1;
    let b1 <bool> eq read_i32_or_zero p 4 2;
    let b2 <bool> eq read_i32_or_zero p 8 3;
    let b3 <bool> eq read_i32_or_zero p 12 4;
    let ok <bool> and b0 and b1 and b2 b3;
    dealloc_raw mem_ptr_addr p 16;
    if ok 1234 0
```

## sort_i32_ptr_with_duplicates

neplg2:test
ret: 2234
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec/sort" as *
#import "core/mem" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn put_i32 <(MemPtr<i32>,i32,i32)->()> (p, off, value):
    unwrap_ok store_i32 mem_ptr_add<i32> p off value

fn read_i32_or_zero <(MemPtr<i32>,i32)->i32> (p, off):
    match load_i32 mem_ptr_add<i32> p off:
        Option::Some value:
            value
        Option::None:
            0

fn main <()->i32> ():
    let p <MemPtr<i32>> unwrap_ok alloc_ptr<i32> 20;
    put_i32 p 0 3;
    put_i32 p 4 1;
    put_i32 p 8 3;
    put_i32 p 12 2;
    put_i32 p 16 1;
    sort_i32 p 5;
    let b0 <bool> eq read_i32_or_zero p 0 1;
    let b1 <bool> eq read_i32_or_zero p 4 1;
    let b2 <bool> eq read_i32_or_zero p 8 2;
    let b3 <bool> eq read_i32_or_zero p 12 3;
    let b4 <bool> eq read_i32_or_zero p 16 3;
    let ok <bool> and b0 and b1 and b2 and b3 b4;
    dealloc_raw mem_ptr_addr p 20;
    if ok 2234 0
```

## sort_i32_ptr_with_negative_values

neplg2:test
ret: 3234
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec/sort" as *
#import "core/mem" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn put_i32 <(MemPtr<i32>,i32,i32)->()> (p, off, value):
    unwrap_ok store_i32 mem_ptr_add<i32> p off value

fn read_i32_or_zero <(MemPtr<i32>,i32)->i32> (p, off):
    match load_i32 mem_ptr_add<i32> p off:
        Option::Some value:
            value
        Option::None:
            0

fn main <()->i32> ():
    let p <MemPtr<i32>> unwrap_ok alloc_ptr<i32> 20;
    put_i32 p 0 -2;
    put_i32 p 4 5;
    put_i32 p 8 0;
    put_i32 p 12 -1;
    put_i32 p 16 3;
    sort_i32 p 5;
    let b0 <bool> eq read_i32_or_zero p 0 -2;
    let b1 <bool> eq read_i32_or_zero p 4 -1;
    let b2 <bool> eq read_i32_or_zero p 8 0;
    let b3 <bool> eq read_i32_or_zero p 12 3;
    let b4 <bool> eq read_i32_or_zero p 16 5;
    let ok <bool> and b0 and b1 and b2 and b3 b4;
    dealloc_raw mem_ptr_addr p 20;
    if ok 3234 0
```

## sort_i32_ptr_len0_noop

neplg2:test
ret: 4234
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec/sort" as *
#import "core/mem" as *
#import "core/math" as *
#import "core/result" as *

fn main <()->i32> ():
    let p <MemPtr<i32>> unwrap_ok alloc_ptr<i32> 4;
    sort_i32 p 0;
    dealloc_raw mem_ptr_addr p 4;
    4234
```

## sort_i32_ptr_len1_noop

neplg2:test
ret: 5234
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec/sort" as *
#import "core/mem" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()->i32> ():
    let p <MemPtr<i32>> unwrap_ok alloc_ptr<i32> 4;
    unwrap_ok store_i32 p 7;
    sort_i32 p 1;
    let ok <bool> match load_i32 p:
        Option::Some value:
            eq value 7
        Option::None:
            false
    dealloc_raw mem_ptr_addr p 4;
    if ok 5234 0
```
