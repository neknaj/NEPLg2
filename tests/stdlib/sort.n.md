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
    sort_quick<i32> make_vec4;
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
    match sort_merge<i32> make_vec4:
        Result::Ok _:
            1234
        Result::Err _:
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
    let s data_len<i32> &v;
    let p <i32> mem_ptr_addr get s "data";
    let n <i32> get s "len";
    let bn <bool> eq n 4;
    let b0 <bool> eq load_i32 add p 0 1;
    let b1 <bool> eq load_i32 add p 4 2;
    let b2 <bool> eq load_i32 add p 8 3;
    let b3 <bool> eq load_i32 add p 12 4;
    if and bn and b0 and b1 and b2 b3 1334 0
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
    get s "len"
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
    get s "len"
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
    get s "len"
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
    sort_heap<i32> make_vec4;
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
    let s data_len<i32> &v;
    let p <i32> mem_ptr_addr get s "data";
    let n <i32> get s "len";
    let bn <bool> eq n 4;
    let b0 <bool> eq load_i32 add p 0 1;
    let b1 <bool> eq load_i32 add p 4 2;
    let b2 <bool> eq load_i32 add p 8 3;
    let b3 <bool> eq load_i32 add p 12 4;
    if and bn and b0 and b1 and b2 b3 1434 0
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
    get s "len"
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
    get s "len"
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
    get s "len"
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
    let s data_len<i32> &v;
    let p <i32> mem_ptr_addr get s "data";
    let n <i32> get s "len";
    let bn <bool> eq n 4;
    let b0 <bool> eq load_i32 add p 0 1;
    let b1 <bool> eq load_i32 add p 4 2;
    let b2 <bool> eq load_i32 add p 8 3;
    let b3 <bool> eq load_i32 add p 12 4;
    if and bn and b0 and b1 and b2 b3 1534 0
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
    get s "len"
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
    get s "len"
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
    get s "len"
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
    sort<i32> make_vec4;
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
    let before sort_is_sorted<i32> make_vec4;
    let after sort_is_sorted<i32> block:
        let mut v <Vec<i32>> unwrap_ok new<i32>;
        set v unwrap_ok push<i32> v 1;
        set v unwrap_ok push<i32> v 2;
        set v unwrap_ok push<i32> v 3;
        set v unwrap_ok push<i32> v 4;
        v
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

fn main <()->i32> ():
    let p <i32> alloc_raw 16;
    store_i32 add p 0 4;
    store_i32 add p 4 1;
    store_i32 add p 8 3;
    store_i32 add p 12 2;
    sort_i32 p 4;
    let b0 <bool> eq load_i32 add p 0 1;
    let b1 <bool> eq load_i32 add p 4 2;
    let b2 <bool> eq load_i32 add p 8 3;
    let b3 <bool> eq load_i32 add p 12 4;
    let ok <bool> and b0 and b1 and b2 b3;
    dealloc_raw p 16;
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

fn main <()->i32> ():
    let p <i32> alloc_raw 20;
    store_i32 add p 0 3;
    store_i32 add p 4 1;
    store_i32 add p 8 3;
    store_i32 add p 12 2;
    store_i32 add p 16 1;
    sort_i32 p 5;
    let b0 <bool> eq load_i32 add p 0 1;
    let b1 <bool> eq load_i32 add p 4 1;
    let b2 <bool> eq load_i32 add p 8 2;
    let b3 <bool> eq load_i32 add p 12 3;
    let b4 <bool> eq load_i32 add p 16 3;
    let ok <bool> and b0 and b1 and b2 and b3 b4;
    dealloc_raw p 20;
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

fn main <()->i32> ():
    let p <i32> alloc_raw 20;
    store_i32 add p 0 -2;
    store_i32 add p 4 5;
    store_i32 add p 8 0;
    store_i32 add p 12 -1;
    store_i32 add p 16 3;
    sort_i32 p 5;
    let b0 <bool> eq load_i32 add p 0 -2;
    let b1 <bool> eq load_i32 add p 4 -1;
    let b2 <bool> eq load_i32 add p 8 0;
    let b3 <bool> eq load_i32 add p 12 3;
    let b4 <bool> eq load_i32 add p 16 5;
    let ok <bool> and b0 and b1 and b2 and b3 b4;
    dealloc_raw p 20;
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

fn main <()->i32> ():
    let p <i32> alloc_raw 4;
    sort_i32 p 0;
    dealloc_raw p 4;
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

fn main <()->i32> ():
    let p <i32> alloc_raw 4;
    store_i32 p 7;
    sort_i32 p 1;
    let ok <bool> eq load_i32 p 7;
    dealloc_raw p 4;
    if ok 5234 0
```
