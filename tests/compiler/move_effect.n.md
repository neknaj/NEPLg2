# move/effect 回帰テスト

## pure からメモリ操作を呼べない

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#entry main
#indent 4
#target core
#import "core/cast" as *

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn compute <()->i32> ():
    let p <i32> alloc_raw 4
    store_i32 p 123
    let v <i32> load_i32 p
    dealloc_raw p 4
    v

fn main <()->i32> ():
    compute
```

## pure から dealloc_raw で allocator state を変更できない

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn release_external <(i32)->i32> (p):
    dealloc_raw p 4
    0

fn main <()->i32> ():
    release_external 16
```

## pure から realloc_raw で allocator state を変更できない

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn resize_external <(i32)->i32> (p):
    let q <i32> realloc_raw p 4 8
    dealloc_raw q 8
    0

fn main <()->i32> ():
    resize_external 16
```

## pure から mem_size で線形メモリ状態を観測できない

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn observe_memory <()->i32> ():
    mem_size

fn main <()->i32> ():
    observe_memory
```

## pure から alloc_raw の raw address を返せない

neplg2:test[compile_fail]
diag_code: resource.raw.identity_escape
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn leak_raw <()->i32> ():
    alloc_raw 4

fn main <()->i32> ():
    leak_raw
```

## pure から alloc_raw の raw address を struct に包んで返せない

neplg2:test[compile_fail]
diag_code: resource.raw.identity_escape
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct RawBox:
    ptr <i32>

fn leak_box <()->RawBox> ():
    let p <i32> alloc_raw 4
    RawBox p

fn main <()->i32> ():
    let b <RawBox> leak_box
    0
```

## pure helper 経由でも alloc_raw の raw address を返せない

neplg2:test[compile_fail]
diag_code: resource.raw.identity_escape
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn raw_id <(i32)->i32> (p):
    p

fn leak_via_helper <()->i32> ():
    let p <i32> alloc_raw 4
    raw_id p

fn main <()->i32> ():
    leak_via_helper
```

## function value 経由でも alloc_raw の raw address を返せない

neplg2:test[compile_fail]
diag_code: resource.raw.identity_escape
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn raw_id <(i32)->i32> (p):
    p

fn leak_via_function_value <()->i32> ():
    let f @raw_id;
    let p <i32> alloc_raw 4
    f p

fn main <()->i32> ():
    leak_via_function_value
```

## higher-order helper 経由でも alloc_raw の raw address を返せない

neplg2:test[compile_fail]
diag_code: resource.raw.identity_escape
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn raw_id <(i32)->i32> (p):
    p

fn apply_raw <(i32, (i32)->i32)->i32> (p, f):
    f p

fn leak_via_higher_order <()->i32> ():
    let p <i32> alloc_raw 4
    apply_raw p @raw_id

fn main <()->i32> ():
    leak_via_higher_order
```

## raw slot 経由でも alloc_raw の raw address を返せない

neplg2:test[compile_fail]
diag_code: resource.raw.identity_escape
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn leak_via_raw_slot <()->i32> ():
    let p <i32> alloc_raw 4
    let slot <i32> alloc_raw 4
    store_i32 slot p
    load_i32 slot

fn main <()->i32> ():
    leak_via_raw_slot
```

## realloc 後の raw slot 経由でも alloc_raw の raw address を返せない

neplg2:test[compile_fail]
diag_code: resource.raw.identity_escape
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn leak_via_realloc_slot <()->i32> ():
    let p <i32> alloc_raw 4
    let slot <i32> alloc_raw 4
    store_i32 slot p
    let grown <i32> realloc_raw slot 4 8
    if:
        lt 0 grown
        then:
            load_i32 grown
        else:
            load_i32 slot

fn main <()->i32> ():
    leak_via_realloc_slot
```

## mem_copy 後の raw slot 経由でも alloc_raw の raw address を返せない

neplg2:test[compile_fail]
diag_code: resource.raw.identity_escape
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn leak_via_copied_slot <()->i32> ():
    let p <i32> alloc_raw 4
    let src <i32> alloc_raw 4
    let dst <i32> alloc_raw 4
    store_i32 src p
    mem_copy dst src 4
    load_i32 dst

fn main <()->i32> ():
    leak_via_copied_slot
```

## parameter raw slot 経由でも alloc_raw の raw address を返せない

neplg2:test[compile_fail]
diag_code: resource.raw.identity_escape
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn leak_via_param_slot <(i32)->i32> (slot):
    let p <i32> alloc_raw 4
    store_i32 slot p
    load_i32 slot

fn main <()->i32> ():
    let slot <i32> alloc_raw 4
    leak_via_param_slot slot
```

## copied parameter raw slot 経由でも alloc_raw の raw address を返せない

neplg2:test[compile_fail]
diag_code: resource.raw.identity_escape
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn leak_via_copied_param_slot <(i32)->i32> (slot):
    let alias <i32> slot
    let p <i32> alloc_raw 4
    store_i32 alias p
    load_i32 slot

fn main <()->i32> ():
    let slot <i32> alloc_raw 4
    leak_via_copied_param_slot slot
```

## helper に渡した raw identity も parameter raw slot 経由で返せない

neplg2:test[compile_fail]
diag_code: resource.raw.identity_escape
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn raw_slot_id <(i32,i32)->i32> (slot, p):
    store_i32 slot p
    load_i32 slot

fn main <()->i32> ():
    let p <i32> alloc_raw 4
    let slot <i32> alloc_raw 4
    raw_slot_id slot p
```

## helper から返った raw slot pointer 経由でも alloc_raw の raw address を返せない

neplg2:test[compile_fail]
diag_code: resource.raw.identity_escape
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn slot_id <(i32)->i32> (slot):
    slot

fn leak_via_returned_slot <(i32)->i32> (slot):
    let alias <i32> slot_id slot
    let p <i32> alloc_raw 4
    store_i32 alias p
    load_i32 slot

fn main <()->i32> ():
    let slot <i32> alloc_raw 4
    leak_via_returned_slot slot
```

## function value から返った raw slot pointer 経由でも alloc_raw の raw address を返せない

neplg2:test[compile_fail]
diag_code: resource.raw.identity_escape
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn slot_id2 <(i32)->i32> (slot):
    slot

fn leak_via_indirect_returned_slot <(i32)->i32> (slot):
    let f <(i32)->i32> @slot_id2
    let alias <i32> f slot
    let p <i32> alloc_raw 4
    store_i32 alias p
    load_i32 slot

fn main <()->i32> ():
    let slot <i32> alloc_raw 4
    leak_via_indirect_returned_slot slot
```

## pure から raw load intrinsic を直接呼べない

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#entry main
#indent 4
#target core

fn read_raw <()->i32> ():
    #intrinsic "load" <i32> (16)

fn main <()->i32> ():
    read_raw
```

## pure から raw store intrinsic を直接呼べない

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#entry main
#indent 4
#target core

fn write_raw <()->i32> ():
    #intrinsic "store" <i32> (16, 1)
    0

fn main <()->i32> ():
    write_raw
```

## non-Copy raw load は同じ place から二重に所有値を作れない

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    let a <LocalToken> load<LocalToken> p
    let b <LocalToken> load<LocalToken> p
    0
```

## non-Copy raw load は alias した place から二重に所有値を作れない

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    let q <i32> p
    store<LocalToken> p LocalToken @token_id
    let a <LocalToken> load<LocalToken> p
    let b <LocalToken> load<LocalToken> q
    0
```

## non-Copy raw load は同じ MemPtr 由来 address から二重に所有値を作れない

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let r1 <i32> mem_ptr_addr p
    let r2 <i32> mem_ptr_addr p
    store<LocalToken> r1 LocalToken @token_id
    let a <LocalToken> load<LocalToken> r1
    let b <LocalToken> load<LocalToken> r2
    0
```

## non-Copy raw load は copy した MemPtr 由来 address から二重に所有値を作れない

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let q <MemPtr<LocalToken>> p
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let a <LocalToken> load<LocalToken> mem_ptr_addr p
    let b <LocalToken> load<LocalToken> mem_ptr_addr q
    0
```

## non-Copy raw load は mem_ptr_add した同じ MemPtr place から二重に所有値を作れない

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let q <MemPtr<LocalToken>> mem_ptr_add<LocalToken> p 0
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let a <LocalToken> load<LocalToken> mem_ptr_addr p
    let b <LocalToken> load<LocalToken> mem_ptr_addr q
    0
```

## dealloc_ptr は mem_ptr_add した same-place live non-Copy payload を捨てられない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let q <MemPtr<LocalToken>> mem_ptr_add<LocalToken> p 0
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let r <Result<(),str>> dealloc_ptr<LocalToken> q size_of<LocalToken>
    0
```

## direct raw mem_ptr_add の user source 利用は raw boundary 外として拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let q <MemPtr<LocalToken>> mem_ptr_add<LocalToken> p 8
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    store<LocalToken> mem_ptr_addr q LocalToken @token_id
    let a <LocalToken> load<LocalToken> mem_ptr_addr p
    let b <LocalToken> load<LocalToken> mem_ptr_addr q
    0
```

## non-literal mem_ptr_add offset は same-base raw place として保守的に扱う

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn choose_offset <(bool)->i32> (flag):
    if flag 0 8

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let off <i32> choose_offset true
    let q <MemPtr<LocalToken>> mem_ptr_add<LocalToken> p off
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let a <LocalToken> load<LocalToken> mem_ptr_addr p
    let b <LocalToken> load<LocalToken> mem_ptr_addr q
    0
```

## non-literal mem_ptr_add offset は既知の nonzero offset とも overlap する

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn choose_payload_offset <(bool)->i32> (flag):
    if flag 8 16

fn main <()*>i32> ():
    let base <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let exact <MemPtr<LocalToken>> mem_ptr_add<LocalToken> base 8
    let off <i32> choose_payload_offset true
    let q <MemPtr<LocalToken>> mem_ptr_add<LocalToken> base off
    store<LocalToken> mem_ptr_addr exact LocalToken @token_id
    let a <LocalToken> load<LocalToken> mem_ptr_addr q
    let b <LocalToken> load<LocalToken> mem_ptr_addr exact
    0
```

## non-literal mem_ptr_add store は same-base live non-Copy payload を上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn choose_offset <(bool)->i32> (flag):
    if flag 0 8

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let off <i32> choose_offset true
    let q <MemPtr<LocalToken>> mem_ptr_add<LocalToken> p off
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    store<LocalToken> mem_ptr_addr q LocalToken @token_id
    0
```

## non-literal mem_ptr_add dealloc_ptr は same-base live non-Copy payload を捨てられない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn choose_offset <(bool)->i32> (flag):
    if flag 0 8

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let off <i32> choose_offset true
    let q <MemPtr<LocalToken>> mem_ptr_add<LocalToken> p off
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let r <Result<(),str>> dealloc_ptr<LocalToken> q size_of<LocalToken>
    0
```

## non-literal raw address add は same-base raw place として保守的に扱う

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn choose_offset <(bool)->i32> (flag):
    if flag 0 8

fn main <()*>i32> ():
    let p <i32> 16
    let off <i32> choose_offset true
    let q <i32> add p off
    store<LocalToken> p LocalToken @token_id
    let a <LocalToken> load<LocalToken> p
    let b <LocalToken> load<LocalToken> q
    0
```

## signed raw address sub は base provenance を保持する

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let base <i32> 24
    let q <i32> sub base size_of<LocalToken>
    store<LocalToken> q LocalToken @token_id
    let a <LocalToken> load<LocalToken> q
    let b <LocalToken> load<LocalToken> sub base size_of<LocalToken>
    0
```

## direct raw literal address helper の user source 利用は raw boundary 外として拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn slot_ptr <.T,.V> <(i32,i32)->i32> (base, idx):
    add base mul idx add size_of<.T> size_of<.V>

fn main <()*>i32> ():
    let p <i32> 16
    store<LocalToken> slot_ptr<LocalToken,i32> p 0 LocalToken @token_id
    store_i32 add p size_of<LocalToken> 123
    let a <LocalToken> load<LocalToken> p
    0
```

## non-literal raw address helper offset は same-base raw place として保守的に扱う

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn choose_offset <(bool)->i32> (flag):
    if flag 0 size_of<LocalToken>

fn slot_ptr <.T,.V> <(i32,i32)->i32> (base, idx):
    add base mul idx add size_of<.T> size_of<.V>

fn main <()*>i32> ():
    let p <i32> 16
    let off <i32> choose_offset true
    store<LocalToken> p LocalToken @token_id
    store<LocalToken> slot_ptr<LocalToken,i32> p off LocalToken @token_id
    0
```

## non-Copy raw store は未moveの place を上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    store<LocalToken> p LocalToken @token_id
    0
```

## direct raw non-Copy store の user source 利用は raw boundary 外として拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    let a <LocalToken> load<LocalToken> p
    store<LocalToken> p LocalToken @token_id
    0
```

## raw dealloc は initialized non-Copy place を捨てられない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    dealloc_raw p size_of<LocalToken>
    0
```

## direct raw dealloc の user source 利用は raw boundary 外として拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    let a <LocalToken> load<LocalToken> p
    dealloc_raw p size_of<LocalToken>
    0
```

## dealloc_ptr は initialized non-Copy MemPtr place を捨てられない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let r <Result<(),str>> dealloc_ptr<LocalToken> p size_of<LocalToken>
    0
```

## dealloc_region は initialized non-Copy RegionToken place を捨てられない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let token <RegionToken<LocalToken>> region_new<LocalToken> p size_of<LocalToken>
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let r <Result<(),str>> dealloc_region<LocalToken> token
    0
```

## raw address 経由の dealloc_region 前処理は user source では拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    match alloc_region<LocalToken> 1:
        Result::Err _e:
            1
        Result::Ok token:
            let p <MemPtr<LocalToken>> region_ptr &token
            store<LocalToken> mem_ptr_addr p LocalToken @token_id
            let a <LocalToken> load<LocalToken> mem_ptr_addr p
            match dealloc_region<LocalToken> token:
                Result::Err _e:
                    1
                Result::Ok _:
                    0
```

## region_ptr_at の Ok bind は RegionToken raw place として扱う

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let token <RegionToken<LocalToken>> region_new<LocalToken> p size_of<LocalToken>
    match region_ptr_at<LocalToken,LocalToken> &token 0:
        Result::Ok q:
            store<LocalToken> mem_ptr_addr p LocalToken @token_id
            let a <LocalToken> load<LocalToken> mem_ptr_addr p
            let b <LocalToken> load<LocalToken> mem_ptr_addr q
            0
        Result::Err _e:
            0
```

## region_ptr_at の non-literal Ok bind は unknown-offset raw place として扱う

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn choose_offset <(bool)->i32> (flag):
    if flag 0 4

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let token <RegionToken<LocalToken>> region_new<LocalToken> p size_of<LocalToken>
    let off <i32> choose_offset true
    match region_ptr_at<LocalToken,LocalToken> &token off:
        Result::Ok q:
            store<LocalToken> mem_ptr_addr p LocalToken @token_id
            let r <Result<(),str>> dealloc_ptr<LocalToken> q size_of<LocalToken>
            0
        Result::Err _e:
            0
```

## enum payload 変数の MemPtr alias は match bind へ引き継ぐ

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let res <Result<MemPtr<LocalToken>,str>> Result<MemPtr<LocalToken>,str>::Ok p
    match res:
        Result::Ok q:
            store<LocalToken> mem_ptr_addr p LocalToken @token_id
            let a <LocalToken> load<LocalToken> mem_ptr_addr p
            let b <LocalToken> load<LocalToken> mem_ptr_addr q
            0
        Result::Err _e:
            0
```

## enum payload alias は branch merge 後も一致する場合だけ保持する

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let mut res <Result<MemPtr<LocalToken>,str>> Result<MemPtr<LocalToken>,str>::Err "none"
    if true:
        then:
            set res Result<MemPtr<LocalToken>,str>::Ok p
        else:
            set res Result<MemPtr<LocalToken>,str>::Ok p
    match res:
        Result::Ok q:
            store<LocalToken> mem_ptr_addr p LocalToken @token_id
            let a <LocalToken> load<LocalToken> mem_ptr_addr p
            let b <LocalToken> load<LocalToken> mem_ptr_addr q
            0
        Result::Err _e:
            0
```

## aggregate field の MemPtr alias は field get 後も保持する

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/field" as field
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

struct PtrHolder:
    ptr <MemPtr<LocalToken>>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let holder <PtrHolder> PtrHolder p
    let q <MemPtr<LocalToken>> field::get holder "ptr"
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let a <LocalToken> load<LocalToken> mem_ptr_addr p
    let b <LocalToken> load<LocalToken> mem_ptr_addr q
    0
```

## aggregate field alias は branch merge 後も一致する場合だけ保持する

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/field" as field
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

struct PtrHolder:
    ptr <MemPtr<LocalToken>>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let mut holder <PtrHolder> PtrHolder p
    if true:
        then:
            set holder PtrHolder p
        else:
            set holder PtrHolder p
    let q <MemPtr<LocalToken>> field::get holder "ptr"
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let a <LocalToken> load<LocalToken> mem_ptr_addr p
    let b <LocalToken> load<LocalToken> mem_ptr_addr q
    0
```

## enum payload 内 aggregate field の MemPtr alias は match bind 後も保持する

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/field" as field
#import "core/result" as *
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

struct PtrHolder:
    ptr <MemPtr<LocalToken>>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let holder <PtrHolder> PtrHolder p
    let res <Result<PtrHolder,str>> Result<PtrHolder,str>::Ok holder
    match res:
        Result::Ok h:
            let q <MemPtr<LocalToken>> field::get h "ptr"
            store<LocalToken> mem_ptr_addr p LocalToken @token_id
            let a <LocalToken> load<LocalToken> mem_ptr_addr p
            let b <LocalToken> load<LocalToken> mem_ptr_addr q
            0
        Result::Err _e:
            0
```

## enum payload 内 aggregate field alias は branch merge 後も一致する場合だけ保持する

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/field" as field
#import "core/result" as *
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

struct PtrHolder:
    ptr <MemPtr<LocalToken>>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let holder <PtrHolder> PtrHolder p
    let mut res <Result<PtrHolder,str>> Result<PtrHolder,str>::Err "none"
    if true:
        then:
            set res Result<PtrHolder,str>::Ok holder
        else:
            set res Result<PtrHolder,str>::Ok holder
    match res:
        Result::Ok h:
            let q <MemPtr<LocalToken>> field::get h "ptr"
            store<LocalToken> mem_ptr_addr p LocalToken @token_id
            let a <LocalToken> load<LocalToken> mem_ptr_addr p
            let b <LocalToken> load<LocalToken> mem_ptr_addr q
            0
        Result::Err _e:
            0
```

## 関数が返した MemPtr alias は raw place の同一性を保持する

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn id_ptr <(MemPtr<LocalToken>)->MemPtr<LocalToken>> (p):
    p

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let q <MemPtr<LocalToken>> id_ptr p
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let a <LocalToken> load<LocalToken> mem_ptr_addr p
    let b <LocalToken> load<LocalToken> mem_ptr_addr q
    0
```

## 関数が返した aggregate field の MemPtr alias は保持する

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/field" as field
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

struct PtrHolder:
    ptr <MemPtr<LocalToken>>

fn token_id <(i32)->i32> (x):
    x

fn make_holder <(MemPtr<LocalToken>)->PtrHolder> (p):
    PtrHolder p

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let holder <PtrHolder> make_holder p
    let q <MemPtr<LocalToken>> field::get holder "ptr"
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let a <LocalToken> load<LocalToken> mem_ptr_addr p
    let b <LocalToken> load<LocalToken> mem_ptr_addr q
    0
```

## 関数が返した Result payload の MemPtr alias は match bind 後も保持する

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn ok_ptr <(MemPtr<LocalToken>)->Result<MemPtr<LocalToken>,str>> (p):
    Result<MemPtr<LocalToken>,str>::Ok p

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let res <Result<MemPtr<LocalToken>,str>> ok_ptr p
    match res:
        Result::Ok q:
            store<LocalToken> mem_ptr_addr p LocalToken @token_id
            let a <LocalToken> load<LocalToken> mem_ptr_addr p
            let b <LocalToken> load<LocalToken> mem_ptr_addr q
            0
        Result::Err _e:
            0
```

## 関数が返した Result payload 内 aggregate field alias は保持する

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/field" as field
#import "core/result" as *
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

struct PtrHolder:
    ptr <MemPtr<LocalToken>>

fn token_id <(i32)->i32> (x):
    x

fn ok_holder <(PtrHolder)->Result<PtrHolder,str>> (holder):
    Result<PtrHolder,str>::Ok holder

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let holder <PtrHolder> PtrHolder p
    let res <Result<PtrHolder,str>> ok_holder holder
    match res:
        Result::Ok h:
            let q <MemPtr<LocalToken>> field::get h "ptr"
            store<LocalToken> mem_ptr_addr p LocalToken @token_id
            let a <LocalToken> load<LocalToken> mem_ptr_addr p
            let b <LocalToken> load<LocalToken> mem_ptr_addr q
            0
        Result::Err _e:
            0
```

## if で返した関数戻り値の MemPtr alias は両分岐一致時に保持する

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn choose_ptr <(bool,MemPtr<LocalToken>)->MemPtr<LocalToken>> (flag, p):
    if flag:
        then:
            p
        else:
            p

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let q <MemPtr<LocalToken>> choose_ptr true p
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let a <LocalToken> load<LocalToken> mem_ptr_addr p
    let b <LocalToken> load<LocalToken> mem_ptr_addr q
    0
```

## 関数が返した Result を直接 match しても payload alias は保持する

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn ok_ptr <(MemPtr<LocalToken>)->Result<MemPtr<LocalToken>,str>> (p):
    Result<MemPtr<LocalToken>,str>::Ok p

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    match ok_ptr p:
        Result::Ok q:
            store<LocalToken> mem_ptr_addr p LocalToken @token_id
            let a <LocalToken> load<LocalToken> mem_ptr_addr p
            let b <LocalToken> load<LocalToken> mem_ptr_addr q
            0
        Result::Err _e:
            0
```

## raw realloc は initialized non-Copy place を byte move できない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    let q <i32> realloc_raw p size_of<LocalToken> 32
    q
```

## direct raw realloc の user source 利用は raw boundary 外として拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> alloc_raw size_of<LocalToken>
    store<LocalToken> p LocalToken @token_id
    let a <LocalToken> load<LocalToken> p
    let q <i32> realloc_raw p size_of<LocalToken> 32
    if:
        lt 0 q
        then:
            dealloc_raw q 32
            0
        else:
            dealloc_raw p size_of<LocalToken>
            0
```

## realloc_ptr は initialized non-Copy MemPtr place を byte move できない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let r <Result<MemPtr<LocalToken>,str>> realloc_ptr<LocalToken> p size_of<LocalToken> 32
    0
```

## raw mem_copy は initialized non-Copy source を複製できない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let src <i32> 16
    let dst <i32> 64
    store<LocalToken> src LocalToken @token_id
    mem_copy dst src size_of<LocalToken>
    0
```

## raw mem_move は initialized non-Copy source を byte move できない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let src <i32> 16
    let dst <i32> 64
    store<LocalToken> src LocalToken @token_id
    mem_move dst src size_of<LocalToken>
    0
```

## raw mem_copy は initialized non-Copy destination を上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let src <i32> 16
    let dst <i32> 64
    store<LocalToken> dst LocalToken @token_id
    mem_copy dst src size_of<LocalToken>
    0
```

## MemPtr mem_copy は initialized non-Copy destination を上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let raw_dst <i32> 16
    let raw_src <i32> 64
    let dst <MemPtr<i32>> mem_ptr_wrap<i32> raw_dst
    let src <MemPtr<i32>> mem_ptr_wrap<i32> raw_src
    store<LocalToken> raw_dst LocalToken @token_id
    store_i32 raw_src 123
    let r <Result<(),str>> mem_copy<i32> dst src 1
    0
```

## direct raw mem_copy の user source 利用は raw boundary 外として拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let src <i32> 16
    let dst <i32> 64
    store<LocalToken> src LocalToken @token_id
    let a <LocalToken> load<LocalToken> src
    mem_copy dst src size_of<LocalToken>
    0
```

## direct raw mem_copy は Copy bytes でも user source では拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn main <()*>i32> ():
    let src <i32> 16
    let dst <i32> 64
    store_i32 src 123
    mem_copy dst src 4
    load_i32 dst
```

## raw store_i32 は initialized non-Copy place を上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    store_i32 p 0
    0
```

## MemPtr store_i32 は initialized non-Copy place を上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let raw <i32> 16
    let pi <MemPtr<i32>> mem_ptr_wrap<i32> raw
    store<LocalToken> raw LocalToken @token_id
    store_i32 pi 0
    0
```

## 関数内の MemPtr store_i32 は caller の initialized non-Copy place を上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn clobber_i32 <(MemPtr<i32>)->()> (p):
    let r <Result<(),str>> store_i32 p 0
    ()

fn main <()*>i32> ():
    let raw <i32> 16
    let pi <MemPtr<i32>> mem_ptr_wrap<i32> raw
    store<LocalToken> raw LocalToken @token_id
    clobber_i32 pi
    0
```

## if 条件内の helper raw write も caller の initialized non-Copy place を上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn clobber_and_true <(MemPtr<i32>)*>bool> (p):
    let r <Result<(),str>> store_i32 p 0
    true

fn gated_clobber <(MemPtr<i32>)*>()> (p):
    if clobber_and_true p:
        then:
            ()
        else:
            ()

fn main <()*>i32> ():
    let raw <i32> 16
    let pi <MemPtr<i32>> mem_ptr_wrap<i32> raw
    store<LocalToken> raw LocalToken @token_id
    gated_clobber pi
    0
```

## higher-order helper の function value raw write も caller の initialized non-Copy place を上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn clobber_i32 <(MemPtr<i32>)*>()> (p):
    let r <Result<(),str>> store_i32 p 0
    ()

fn apply_clobber <(MemPtr<i32>, (MemPtr<i32>)*>())*>()> (p, f):
    f p

fn main <()*>i32> ():
    let raw <i32> 16
    let pi <MemPtr<i32>> mem_ptr_wrap<i32> raw
    store<LocalToken> raw LocalToken @token_id
    apply_clobber pi @clobber_i32
    0
```

## 多段 higher-order helper の function value raw write も caller の initialized non-Copy place を上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn clobber_i32 <(MemPtr<i32>)*>()> (p):
    let r <Result<(),str>> store_i32 p 0
    ()

fn apply_clobber <(MemPtr<i32>, (MemPtr<i32>)*>())*>()> (p, f):
    f p

fn forward_clobber <(MemPtr<i32>, (MemPtr<i32>)*>())*>()> (p, f):
    apply_clobber p f

fn main <()*>i32> ():
    let raw <i32> 16
    let pi <MemPtr<i32>> mem_ptr_wrap<i32> raw
    store<LocalToken> raw LocalToken @token_id
    forward_clobber pi @clobber_i32
    0
```

## 分岐で選ばれた function value raw write も caller の initialized non-Copy place を上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn clobber_a <(MemPtr<i32>)*>()> (p):
    let r <Result<(),str>> store_i32 p 0
    ()

fn clobber_b <(MemPtr<i32>)*>()> (p):
    let r <Result<(),str>> store_i32 p 1
    ()

fn main <()*>i32> ():
    let raw <i32> 16
    let pi <MemPtr<i32>> mem_ptr_wrap<i32> raw
    store<LocalToken> raw LocalToken @token_id
    let f <(MemPtr<i32>)*>()> if true:
        then:
            @clobber_a
        else:
            @clobber_b
    f pi
    0
```

## aggregate field に保存した function value raw write も caller の initialized non-Copy place を上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

struct CallbackHolder:
    cb <(MemPtr<i32>)*>()>

fn clobber_i32 <(MemPtr<i32>)*>()> (p):
    let r <Result<(),str>> store_i32 p 0
    ()

fn call_holder <(MemPtr<i32>, CallbackHolder)*>()> (p, holder):
    let f <(MemPtr<i32>)*>()> field::get holder "cb"
    f p

fn main <()*>i32> ():
    let raw <i32> 16
    let pi <MemPtr<i32>> mem_ptr_wrap<i32> raw
    store<LocalToken> raw LocalToken @token_id
    let holder <CallbackHolder> CallbackHolder @clobber_i32
    call_holder pi holder
    0
```

## enum payload の function value raw write も caller の initialized non-Copy place を上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/option" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn clobber_i32 <(MemPtr<i32>)*>()> (p):
    let r <Result<(),str>> store_i32 p 0
    ()

fn call_option <(MemPtr<i32>, Option<(MemPtr<i32>)*>()>)*>()> (p, opt):
    match opt:
        Option::Some f:
            f p
        Option::None:
            ()

fn main <()*>i32> ():
    let raw <i32> 16
    let pi <MemPtr<i32>> mem_ptr_wrap<i32> raw
    store<LocalToken> raw LocalToken @token_id
    call_option pi Option::Some @clobber_i32
    0
```

## generic raw store の Copy 値でも initialized non-Copy place は上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    store<i32> p 0
    0
```

## raw memset_u8 は initialized non-Copy place を byte overwrite できない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    memset_u8 p size_of<LocalToken> 0
    0
```

## raw fill_i32 は initialized non-Copy place を上書きできない

neplg2:test[compile_fail]
diag_code: resource.cell.initialized_conflict
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    fill_i32 p 1 0
    0
```

## direct raw byte write の user source 利用は raw boundary 外として拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    let a <LocalToken> load<LocalToken> p
    store_i32 p 0
    0
```

## direct raw byte write は Copy storage でも user source では拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn main <()*>i32> ():
    let p <i32> 16
    store_i32 p 123
    store_i32 p 456
    load_i32 p
```

## direct raw aggregate load の user source 利用は raw boundary 外として拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/field" as field
#import "core/math" as *
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

struct Holder:
    count <i32>
    token <LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<Holder> p Holder 7 LocalToken @token_id
    let a <i32> field::get load<Holder> p "count"
    let b <i32> field::get load<Holder> p "count"
    let h <Holder> load<Holder> p
    add a b
```

## direct raw aggregate read と re-export get の組み合わせは user source では拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

struct Holder:
    count <i32>
    token <LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<Holder> p Holder 7 LocalToken @token_id
    let a <i32> get load<Holder> p "count"
    let b <i32> get load<Holder> p "count"
    let h <Holder> load<Holder> p
    add a b
```

## generic Copy impl を持つ direct raw aggregate read は user source では拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/traits/copy" as *
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

struct Holder:
    count <i32>
    ptr <MemPtr<u8>>
    token <LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<Holder> p Holder 7 mem_ptr_wrap<u8> 64 LocalToken @token_id
    let ptr <MemPtr<u8>> get load<Holder> p "ptr"
    let raw <i32> mem_ptr_addr ptr
    let h <Holder> load<Holder> p
    add raw sub 14 64
```

## generic aggregate の direct raw field read は user source では拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/traits/copy" as *
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

struct Holder<.H>:
    count <i32>
    ptr <MemPtr<u8>>
    token <.H>

fn token_id <(i32)->i32> (x):
    x

fn touch <.H> <(Holder<.H>)*>i32> (h):
    let p <i32> 16
    store<Holder<.H>> p h
    let ptr <MemPtr<u8>> get load<Holder<.H>> p "ptr"
    let raw <i32> mem_ptr_addr ptr
    let out <Holder<.H>> load<Holder<.H>> p
    add raw sub 14 64

fn main <()*>i32> ():
    touch<LocalToken> Holder<LocalToken> 7 mem_ptr_wrap<u8> 64 LocalToken @token_id
```

## direct raw place を含む branch merge 検査は user source では raw boundary が先に拒否する

neplg2:test[compile_fail]
diag_code: resource.raw.memory_outside_boundary
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    let mut i <i32> 0
    while lt i 2:
        do:
            set i add i 1
    let out <LocalToken> load<LocalToken> p
    i
```

## raw aggregate field から move した non-Copy field は二重に取り出せない

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/field" as field
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

struct Holder:
    count <i32>
    token <LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<Holder> p Holder 7 LocalToken @token_id
    let a <LocalToken> field::get load<Holder> p "token"
    let b <LocalToken> field::get load<Holder> p "token"
    0
```

## raw aggregate field から non-Copy field を move した後は aggregate 全体を取り出せない

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/field" as field
#import "core/field" as *

struct LocalToken:
    raw <(i32)->i32>

struct Holder:
    count <i32>
    token <LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <i32> 16
    store<Holder> p Holder 7 LocalToken @token_id
    let a <LocalToken> field::get load<Holder> p "token"
    let h <Holder> load<Holder> p
    0
```

## pure から impure 関数を呼ぶと拒否

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#entry main
#indent 4
#target std

#import "std/stdio" as *
#import "core/field" as *

fn put <(i32)*>()> (x):
    print_i32 x

fn bad <(i32)->i32> (x):
    put x
    x

fn main <()->i32> ():
    bad 1
```

## pure の raw body で I/O を含む場合は拒否

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#entry main
#indent 4
#target core

fn raw_io <()->i32> ():
    #if[target=wasm]
    #wasm:
        i32.const 0
        call $fd_write
        drop
        i32.const 0
    #if[target=llvm]
    #llvmir:
        define i32 @raw_io() {
        entry:
            %x = call i32 @fd_write(i32 0)
            ret i32 0
        }

fn main <()->i32> ():
    raw_io
```

## ローカル変数の set は pure のまま使える

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_effect_local_set_pure\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"pure local set result\" expected=\"42\" actual=\"42\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "core/math" as *
#import "std/test" as test

fn bump_local <(i32)->i32> (n):
    let mut x <i32> n
    set x add x 2
    x

fn main <()*>i32> ():
    let actual <i32> bump_local 40
    let report:
        test::test_report_new "move_effect_local_set_pure"
        |> test::test_report_push test::assert_eq_i32 "pure local set result" 42 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## Copy impl がある struct は再利用できる

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_effect_copy_struct_reuse\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"copy struct reuse\" expected=\"60\" actual=\"60\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "core/math" as *
#import "core/traits/copy" as *
#import "core/field" as *
#import "std/test" as test

struct Point:
    x <i32>
    y <i32>

impl Clone for Point:
    fn clone <(&Point)->Point> (x):
        *x

impl Copy for Point:
    fn copy_mark <(Point)->Point> (x):
        x

fn sum_point <(Point)->i32> (p):
    add get p "x" get p "y"

fn main <()*>i32> ():
    let p1 <Point> Point 10 20
    let p2 <Point> p1
    let actual <i32> add sum_point p1 sum_point p2
    let report:
        test::test_report_new "move_effect_copy_struct_reuse"
        |> test::test_report_push test::assert_eq_i32 "copy struct reuse" 60 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## Copy impl がある具体化済み generic struct は再利用できる

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_effect_copy_generic_concrete_reuse\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"copy concrete generic struct reuse\" expected=\"6\" actual=\"6\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "core/math" as *
#import "core/traits/copy" as *
#import "core/field" as *
#import "std/test" as test

struct Pair<.T>:
    a <.T>
    b <.T>

impl Clone for Pair<i32>:
    fn clone <(&Pair<i32>)->Pair<i32>> (x):
        *x

impl Copy for Pair<i32>:
    fn copy_mark <(Pair<i32>)->Pair<i32>> (x):
        x

fn sum_pair <(Pair<i32>)->i32> (p):
    add get p "a" get p "b"

fn main <()*>i32> ():
    let q1 <Pair<i32>> Pair 1 2
    let q2 <Pair<i32>> q1
    let actual <i32> add sum_pair q1 sum_pair q2
    let report:
        test::test_report_new "move_effect_copy_generic_concrete_reuse"
        |> test::test_report_push test::assert_eq_i32 "copy concrete generic struct reuse" 6 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## Copy bound つき generic Copy impl は具体化後に再利用できる

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_effect_copy_generic_bound_reuse\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"copy generic bound reuse\" expected=\"6\" actual=\"6\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "core/math" as *
#import "core/traits/copy" as *
#import "core/field" as *
#import "std/test" as test

struct Pair<.T>:
    a <.T>
    b <.T>

impl<.T: Copy> Clone for Pair<.T>:
    fn clone <(&Pair<.T>)->Pair<.T>> (x):
        *x

impl<.T: Copy> Copy for Pair<.T>:
    fn copy_mark <(Pair<.T>)->Pair<.T>> (x):
        x

fn sum_pair <(Pair<i32>)->i32> (p):
    add get p "a" get p "b"

fn main <()*>i32> ():
    let q1 <Pair<i32>> Pair 1 2
    let q2 <Pair<i32>> q1
    let actual <i32> add sum_pair q1 sum_pair q2
    let report:
        test::test_report_new "move_effect_copy_generic_bound_reuse"
        |> test::test_report_push test::assert_eq_i32 "copy generic bound reuse" 6 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## Copy bound つき generic Copy impl は非 Copy の具体型へ適用しない

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core

#import "core/option" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let token <LocalToken> LocalToken @token_id
    let opt <Option<LocalToken>> Option::Some token
    let first <Option<LocalToken>> opt
    let second <Option<LocalToken>> opt
    0
```

## Copy bound つき generic Copy impl は Copy の具体型へ適用する

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_effect_copy_bound_applies_to_copy_type\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"copy bound applies to copy type\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "core/option" as *
#import "std/test" as test

fn main <()*>i32> ():
    let opt <Option<i32>> Option::Some 1
    let first <Option<i32>> opt
    let second <Option<i32>> opt
    let actual <i32> 0
    let report:
        test::test_report_new "move_effect_copy_bound_applies_to_copy_type"
        |> test::test_report_push test::assert_eq_i32 "copy bound applies to copy type" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## Copy impl がある enum は再利用できる

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_effect_copy_enum_reuse\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"copy enum reuse\" expected=\"14\" actual=\"14\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "core/math" as *
#import "core/traits/copy" as *
#import "std/test" as test

enum Score:
    Single <i32>
    Zero

impl Clone for Score:
    fn clone <(&Score)->Score> (x):
        *x

impl Copy for Score:
    fn copy_mark <(Score)->Score> (x):
        x

fn as_i32 <(Score)->i32> (s):
    match s:
        Score::Single v:
            v
        Score::Zero:
            0

fn main <()*>i32> ():
    let s1 <Score> Score::Single 7
    let s2 <Score> s1
    let actual <i32> add as_i32 s1 as_i32 s2
    let report:
        test::test_report_new "move_effect_copy_enum_reuse"
        |> test::test_report_push test::assert_eq_i32 "copy enum reuse" 14 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## 関数内で未定義変数を set すると拒否

neplg2:test[compile_fail]
diag_code: type.variable.undefined
```neplg2
#entry main
#indent 4
#target core

let mut g <i32> 0

fn bump_global <(i32)->i32> (x):
    set g x
    g

fn main <()->i32> ():
    bump_global 5
```

## 非Copy値の shared borrow 中 move は拒否

neplg2:test[compile_fail]
diag_code: resource.borrow.move_from_shared
```neplg2
#entry main
#indent 4
#target core

struct Boxed:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let b <Boxed> Boxed @token_id
    let r &b
    let c b
    let keep r
    0
```

## Copy値への borrow は move を阻害しない

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_effect_copy_borrow_allows_reuse\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"copy borrow allows reuse\" expected=\"11\" actual=\"11\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "core/math" as *
#import "std/test" as test

fn main <()*>i32> ():
    let x <i32> 10
    let r &x
    let actual <i32> add x 1
    let report:
        test::test_report_new "move_effect_copy_borrow_allows_reuse"
        |> test::test_report_push test::assert_eq_i32 "copy borrow allows reuse" 11 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## Copy impl の対象が非Copy型なら拒否

neplg2:test[compile_fail]
diag_code: type.copy_impl.requires_clone
```neplg2
#entry main
#indent 4
#target core
#no_prelude

trait Clone:
    #capability clone
    fn clone <(Self)->Self> (x):
        x

trait Copy:
    #capability copy
    fn copy_mark <(Self)->Self> (x):
        x

struct LocalToken:
    raw <(i32)->i32>

impl Copy for LocalToken:
    fn copy_mark <(LocalToken)->LocalToken> (x):
        x

fn main <()->i32> ():
    0
```

## compiler owner token への Copy impl は拒否する

`RegionToken<T>` は compiler memory boundary が発行する free obligation owner token です。
field 構造が `MemPtr<T>` と `i32` だけに見えても、`Copy` capability を付けると
同じ解放責務を持つ token を複製できてしまうため拒否します。

neplg2:test[compile_fail]
diag_code: type.copy_impl.target_not_copy
```neplg2
#entry main
#indent 4
#target core

#import "core/mem" as *
#import "core/traits/copy" as *

impl<.T> Copy for RegionToken<.T>:
    fn copy_mark <(RegionToken<.T>)->RegionToken<.T>> (x):
        x

fn main <()->i32> ():
    0
```

## 同名の通常 struct への Copy impl は拒否しない

compiler memory owner token として認識された定義だけを禁止対象にします。
`#no_prelude` の通常 user struct が偶然 `RegionToken` という名前を持つ場合は、
field が copyable で `Clone` もあるなら通常の `Copy` として扱えます。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#no_prelude

trait Clone:
    #capability clone
    fn clone <(&Self)->Self> (x):
        *x

trait Copy:
    #capability copy
    fn copy_mark <(Self)->Self> (x):
        x

struct RegionToken:
    value <i32>

impl Clone for RegionToken:
    fn clone <(&RegionToken)->RegionToken> (x):
        *x

impl Copy for RegionToken:
    fn copy_mark <(RegionToken)->RegionToken> (x):
        x

fn main <()->i32> ():
    let a <RegionToken> RegionToken 1
    let b <RegionToken> a
    let c <RegionToken> a
    0
```

## Copy impl には Clone impl が必要

neplg2:test[compile_fail]
diag_code: type.copy_impl.requires_clone
```neplg2
#entry main
#indent 4
#target core
#no_prelude

trait Clone:
    #capability clone
    fn clone <(Self)->Self> (x):
        x

trait Copy:
    #capability copy
    fn copy_mark <(Self)->Self> (x):
        x

impl Copy for i32:
    fn copy_mark <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
```

## Clone と Copy の両方があれば受理

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#no_prelude

trait Clone:
    #capability clone
    fn clone <(Self)->Self> (x):
        x

trait Copy:
    #capability copy
    fn copy_mark <(Self)->Self> (x):
        x

impl Clone for i32:
    fn clone <(i32)->i32> (x):
        x

impl Copy for i32:
    fn copy_mark <(i32)->i32> (x):
        x

fn main <()->i32> ():
    0
```

## Copy trait 有効時は copy-eligible 型も impl がなければ move 扱い

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core
#no_prelude

trait Clone:
    #capability clone
    fn clone <(Self)->Self> (x):
        x

trait Copy:
    #capability copy
    fn copy_mark <(Self)->Self> (x):
        x

struct Size:
    n <i32>

fn main <()->i32> ():
    let a <Size> Size 10
    let b <Size> a
    let c <Size> a
    0
```

## Copy trait 有効時でも copy-eligible 型に Clone+Copy があれば再利用できる

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#no_prelude

trait Clone:
    #capability clone
    fn clone <(Self)->Self> (x):
        x

trait Copy:
    #capability copy
    fn copy_mark <(Self)->Self> (x):
        x

struct Size:
    n <i32>

impl Clone for Size:
    fn clone <(Size)->Size> (x):
        x

impl Copy for Size:
    fn copy_mark <(Size)->Size> (x):
        x

fn main <()->i32> ():
    let a <Size> Size 10
    let b <Size> a
    let c <Size> a
    0
```

## 同一 trait と同一対象型への impl 重複は拒否

neplg2:test[compile_fail]
diag_code: type.impl.duplicate_for_trait_target
```neplg2
#entry main
#indent 4
#target core

trait Mark:
    fn mark <(Self)->Self> (x):
        x

impl Mark for i32:
    fn mark <(i32)->i32> (x):
        x

impl Mark for i32:
    fn mark <(i32)->i32> (x):
        x
```

## marker trait は #capability copy 未指定なら Copy 扱いしない

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_effect_marker_trait_not_copy\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"marker trait without capability is not copy\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "std/test" as test

trait Marker:
    fn tag <(Self)->Self> (x):
        x

struct LocalToken:
    raw <(i32)->i32>

impl Marker for LocalToken:
    fn tag <(LocalToken)->LocalToken> (x):
        x

fn main <()*>i32> ():
    let actual <i32> 0
    let report:
        test::test_report_new "move_effect_marker_trait_not_copy"
        |> test::test_report_push test::assert_eq_i32 "marker trait without capability is not copy" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## clone 形状の trait も #capability clone 未指定なら Clone 扱いしない

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_effect_clone_shape_not_clone\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"clone shape without capability is not clone\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "std/test" as test

trait Dup:
    fn dup <(Self)->Self> (x):
        x

impl Dup for i32:
    fn dup <(i32)->i32> (x):
        x

fn main <()*>i32> ():
    let actual <i32> 0
    let report:
        test::test_report_new "move_effect_clone_shape_not_clone"
        |> test::test_report_push test::assert_eq_i32 "clone shape without capability is not clone" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## 不明 capability 名は診断ID付きで拒否

neplg2:test[compile_fail]
diag_code: type.trait_capability.unknown
```neplg2
#entry main
#indent 4
#target core

trait BadCap:
    #capability cpoy
    fn f <(Self)->Self> (x):
        x

fn main <()->i32> ():
    0
```

## LocalToken は非Copyとして move 後再利用不可

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn consume <(LocalToken)->i32> (_t):
    0

fn main <()->()> ():
    let t <LocalToken> LocalToken @token_id
    consume t
    let u <LocalToken> t
```

## move 後の borrow は拒否

neplg2:test[compile_fail]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->()> ():
    let t <LocalToken> LocalToken @token_id
    let u <LocalToken> t
    let r <&LocalToken> &t
```

## 分岐で move された可能性のある値の使用は拒否

neplg2:test[compile_fail]
diag_code: resource.cell.possibly_moved
```neplg2
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn consume <(LocalToken)->i32> (_t):
    0

fn main <()->i32> ():
    let t <LocalToken> LocalToken @token_id
    if true:
        then:
            consume t
        else:
            0
    consume t
```

## 非複合型への field access は拒否

neplg2:test[compile_fail]
diag_code: type.field.invalid_access
```neplg2
#entry main
#indent 4
#target core

fn main <()->i32> ():
    let v <i32> 10;
    v.len
```

## Writer は非Copyとして move 後再利用不可

neplg2:test[compile_fail, skip_llvm]
diag_code: resource.cell.moved
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/streamio" as *
#import "std/iotarget" as *

fn consume_writer <(StreamWriter)->i32> (_w):
    0

fn main <()*>i32> ():
    let w <StreamWriter> unwrap_ok open WriteStream::Stdio
    let w2 <StreamWriter> w
    consume_writer w
    0
```

## core/traits/copy 導入後は str の再利用が trait impl で成立する

このケースは、`str` が compiler 固定表ではなく `core/traits/copy` の impl によって Copy として扱われることを確かめます。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_effect_str_copy_trait_impl\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"str reuse through copy trait impl\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "core/traits/copy" as *
#import "std/test" as test

fn main <()*>i32> ():
    let s <str> "abc"
    let t <str> s
    let u <str> s
    let actual <i32> 0
    let report:
        test::test_report_new "move_effect_str_copy_trait_impl"
        |> test::test_report_push test::assert_eq_i32 "str reuse through copy trait impl" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## core/traits/copy 導入後は unit の再利用が trait impl で成立する

このケースは、`()` が compiler 固定表ではなく `core/traits/copy` の impl によって Copy として扱われることを確かめます。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"move_effect_unit_copy_trait_impl\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"unit reuse through copy trait impl\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "core/traits/copy" as *
#import "std/test" as test

fn main <()*>i32> ():
    let u <()> ()
    let a <()> u
    let b <()> u
    let actual <i32> 0
    let report:
        test::test_report_new "move_effect_unit_copy_trait_impl"
        |> test::test_report_push test::assert_eq_i32 "unit reuse through copy trait impl" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
