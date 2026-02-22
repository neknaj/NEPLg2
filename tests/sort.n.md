# sort.nepl のテスト

## sort_quick_i32_basic

neplg2:test
ret: 1234
```neplg2
#entry main
#indent 4
#target core
#import "alloc/vec" as *
#import "alloc/sort" as *
#import "core/math" as *

fn make_vec4 <()*>Vec<i32>> ():
    let mut v vec_new<i32>;
    set v vec_push<i32> v 4;
    set v vec_push<i32> v 1;
    set v vec_push<i32> v 3;
    set v vec_push<i32> v 2;
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
#import "alloc/vec" as *
#import "alloc/sort" as *
#import "core/math" as *

fn make_vec4 <()*>Vec<i32>> ():
    let mut v vec_new<i32>;
    set v vec_push<i32> v 4;
    set v vec_push<i32> v 1;
    set v vec_push<i32> v 3;
    set v vec_push<i32> v 2;
    v

fn main <()->i32> ():
    sort_merge<i32> make_vec4;
    1234
```

## sort_heap_i32_basic

neplg2:test
ret: 1234
```neplg2
#entry main
#indent 4
#target core
#import "alloc/vec" as *
#import "alloc/sort" as *
#import "core/math" as *

fn make_vec4 <()*>Vec<i32>> ():
    let mut v vec_new<i32>;
    set v vec_push<i32> v 4;
    set v vec_push<i32> v 1;
    set v vec_push<i32> v 3;
    set v vec_push<i32> v 2;
    v

fn main <()->i32> ():
    sort_heap<i32> make_vec4;
    1234
```

## sort_default_dispatch_i32

neplg2:test
ret: 1234
```neplg2
#entry main
#indent 4
#target core
#import "alloc/vec" as *
#import "alloc/sort" as *
#import "core/math" as *

fn make_vec4 <()*>Vec<i32>> ():
    let mut v vec_new<i32>;
    set v vec_push<i32> v 4;
    set v vec_push<i32> v 1;
    set v vec_push<i32> v 3;
    set v vec_push<i32> v 2;
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
#import "alloc/vec" as *
#import "alloc/sort" as *
#import "core/math" as *

fn make_vec4 <()*>Vec<i32>> ():
    let mut v vec_new<i32>;
    set v vec_push<i32> v 4;
    set v vec_push<i32> v 1;
    set v vec_push<i32> v 3;
    set v vec_push<i32> v 2;
    v

fn main <()->i32> ():
    let before sort_is_sorted<i32> make_vec4;
    let after sort_is_sorted<i32> block:
        let mut v vec_new<i32>;
        set v vec_push<i32> v 1;
        set v vec_push<i32> v 2;
        set v vec_push<i32> v 3;
        set v vec_push<i32> v 4;
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
#import "alloc/sort" as *
#import "core/mem" as *
#import "core/math" as *

fn main <()->i32> ():
    let p <i32> alloc 16;
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
    dealloc p 16;
    if ok 1234 0
```
