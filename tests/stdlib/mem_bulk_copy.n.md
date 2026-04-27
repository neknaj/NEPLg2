# core/mem bulk copy regression

## mem_copy_non_overlap

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core

#import "core/mem" as *
#import "core/math" as *

fn main <()->i32> ():
    let p <i32> alloc_raw 16
    let dst <i32> add p 8
    store_u8 add p 0 11
    store_u8 add p 1 22
    store_u8 add p 2 33
    mem_copy dst p 3
    let ok0 <bool> eq load_u8 add dst 0 11
    let ok1 <bool> eq load_u8 add dst 1 22
    let ok2 <bool> eq load_u8 add dst 2 33
    let ok <bool> and ok0 and ok1 ok2
    dealloc_raw p 16
    if ok 1 0
```

## mem_move_overlap

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core

#import "core/mem" as *
#import "core/math" as *

fn main <()->i32> ():
    let p <i32> alloc_raw 8
    store_u8 add p 0 65
    store_u8 add p 1 66
    store_u8 add p 2 67
    store_u8 add p 3 68
    store_u8 add p 4 69
    store_u8 add p 5 70
    mem_move add p 2 p 4
    let ok0 <bool> eq load_u8 add p 2 65
    let ok1 <bool> eq load_u8 add p 3 66
    let ok2 <bool> eq load_u8 add p 4 67
    let ok3 <bool> eq load_u8 add p 5 68
    let ok <bool> and ok0 and ok1 and ok2 ok3
    dealloc_raw p 8
    if ok 1 0
```

## mem_copy_zero_length_allows_empty_ptr

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core

#import "core/mem" as *
#import "core/result" as *

fn main <()->i32> ():
    let empty <MemPtr<u8>> mem_ptr_wrap 0
    match mem_copy<u8> empty empty 0:
        Result::Ok _:
            1
        Result::Err _e:
            0
```

## typed_mem_copy_rejects_non_copy_owner

neplg2:test[compile_fail]
diag_id: 3069
```neplg2
#entry main
#indent 4
#target core

#import "core/mem" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap 0
    let r <Result<(),str>> mem_copy<LocalToken> p p 0
    0
```

## typed_mem_move_rejects_non_copy_owner

neplg2:test[compile_fail]
diag_id: 3069
```neplg2
#entry main
#indent 4
#target core

#import "core/mem" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap 0
    let r <Result<(),str>> mem_move<LocalToken> p p 0
    0
```

## realloc_raw_preserves_bytes

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core

#import "core/mem" as *
#import "core/math" as *

fn main <()->i32> ():
    let p <i32> alloc_raw 4
    store_u8 add p 0 17
    store_u8 add p 3 99
    let q <i32> realloc_raw p 4 12
    let ok0 <bool> ne q 0
    let ok1 <bool> eq load_u8 add q 0 17
    let ok2 <bool> eq load_u8 add q 3 99
    let ok <bool> and ok0 and ok1 ok2
    dealloc_raw q 12
    if ok 1 0
```

## bytebuf_string_roundtrip_uses_bulk_copy_path

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/io" as *
#import "alloc/string" as *

fn main <()*>i32> ():
    let buf <ByteBuf> io_bytebuf_from_str "bulk-copy-bytebuf"
    let text <str> io_bytebuf_to_str buf
    if str_eq text "bulk-copy-bytebuf" 0 1
```

## large_bulk_copy_fixture

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core

#import "core/mem" as *
#import "core/math" as *

fn main <()->i32> ():
    let len <i32> 4096
    let src <i32> alloc_raw len
    let dst <i32> alloc_raw len
    memset_u8 src len 7
    memset_u8 dst len 0
    store_u8 add src 0 11
    store_u8 add src 2048 22
    store_u8 add src 4095 33
    mem_copy dst src len
    let ok0 <bool> eq load_u8 add dst 0 11
    let ok1 <bool> eq load_u8 add dst 2048 22
    let ok2 <bool> eq load_u8 add dst 4095 33
    let ok <bool> and ok0 and ok1 ok2
    dealloc_raw dst len
    dealloc_raw src len
    if ok 1 0
```
