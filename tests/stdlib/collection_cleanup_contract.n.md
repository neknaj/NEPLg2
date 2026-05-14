# collection cleanup contract

## 概要

[目的/もくてき]:
- 現行 collection のうち `Vec<Option<T>>` / `Vec<T>` storage に[載/の]る派生 collection は、`OwnedBuffer<T>` と element drop traversal が[完成/かんせい]するまで Copy-only 契約で[閉/と]じることを[確認/かくにん]します。
- cleanup/free だけが generic のまま[残/のこ]り、non-Copy payload を[安全/あんぜん]に free できるように[見/み]える退行を[防/ふせ]ぎます。
- 各 doctest は collection family または API ごとに[独立/どくりつ]しており、別 API の trait bound error で[偶然/ぐうぜん]成功することを[避/さ]けます。

## vec_clear_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *

struct CleanupPayload:
    value <i32>

fn close_vec_clear <(Vec<CleanupPayload>)->()> (v):
    let next <Vec<CleanupPayload>> clear<CleanupPayload> v
    ()

fn main <()->i32> ():
    0
```

## vec_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *

struct CleanupPayload:
    value <i32>

fn close_vec_free <(Vec<CleanupPayload>)->()> (v):
    free<CleanupPayload> v

fn main <()->i32> ():
    0
```

## vec_empty_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *

struct CleanupPayload:
    value <i32>

fn make_empty_vec <()->Vec<CleanupPayload>> ():
    vec_empty<CleanupPayload>

fn main <()->i32> ():
    0
```

## stack_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/stack" as *

struct CleanupPayload:
    value <i32>

fn close_stack <(Stack<CleanupPayload>)->()> (s):
    free<CleanupPayload> s

fn main <()->i32> ():
    0
```

## queue_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/queue" as *

struct CleanupPayload:
    value <i32>

fn close_queue <(Queue<CleanupPayload>)->()> (q):
    free<CleanupPayload> q

fn main <()->i32> ():
    0
```

## deque_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/deque" as *

struct CleanupPayload:
    value <i32>

fn close_deque <(Deque<CleanupPayload>)->()> (dq):
    free<CleanupPayload> dq

fn main <()->i32> ():
    0
```

## ringbuffer_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/ringbuffer" as *

struct CleanupPayload:
    value <i32>

fn close_ringbuffer <(RingBuffer<CleanupPayload>)->()> (rb):
    free<CleanupPayload> rb

fn main <()->i32> ():
    0
```

## binary_heap_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *

struct CleanupPayload:
    value <i32>

fn close_binary_heap <(BinaryHeap<CleanupPayload>)->()> (heap):
    free<CleanupPayload> heap

fn main <()->i32> ():
    0
```

## btreeset_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset" as *

struct CleanupPayload:
    value <i32>

fn close_btreeset <(BTreeSet<CleanupPayload>)->()> (set0):
    free<CleanupPayload> set0

fn main <()->i32> ():
    0
```

## btreemap_key_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *

struct CleanupPayload:
    value <i32>

fn close_btreemap_key <(BTreeMap<CleanupPayload, i32>)->()> (hm):
    free<CleanupPayload, i32> hm

fn main <()->i32> ():
    0
```

## btreemap_value_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *

struct CleanupPayload:
    value <i32>

fn close_btreemap_value <(BTreeMap<i32, CleanupPayload>)->()> (hm):
    free<i32, CleanupPayload> hm

fn main <()->i32> ():
    0
```

## list_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/list" as *

struct CleanupPayload:
    value <i32>

fn close_list <(List<CleanupPayload>)->()> (lst):
    free<CleanupPayload> lst

fn main <()->i32> ():
    0
```

## hashmap_value_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *

struct CleanupPayload:
    value <i32>

fn close_hashmap_value <(HashMap<i32, CleanupPayload, DefaultHash32>)->()> (hm):
    free<i32, CleanupPayload, DefaultHash32> hm

fn main <()->i32> ():
    0
```

## hashset_key_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "core/traits/hash" as *
#import "core/traits/hash_key" as *

struct NonCopyHashKey:
    seed <(i32)->i32>

fn id <(i32)->i32> (x):
    x

impl HashKey for NonCopyHashKey:
    fn eq <(NonCopyHashKey,NonCopyHashKey)->bool> (_a, _b):
        true

    fn hash32 <(NonCopyHashKey)->i32> (_self):
        0

fn close_hashset_key <(HashSet<NonCopyHashKey, DefaultHash32>)->()> (hs):
    free<NonCopyHashKey, DefaultHash32> hs

fn main <()->i32> ():
    0
```

## hashset_hasher_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *
#import "core/traits/hash" as *

struct StatefulHasher:
    seed <(i32)->i32>

fn id <(i32)->i32> (x):
    x

impl Hasher<i32> for StatefulHasher:
    fn hash32 <(StatefulHasher,i32)->i32> (_h, key):
        key

fn close_hashset_hasher <(HashSet<i32, StatefulHasher>)->()> (hs):
    free<i32, StatefulHasher> hs

fn main <()->i32> ():
    0
```

## hashmap_key_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *
#import "core/traits/hash_key" as *

struct NonCopyHashKey:
    seed <(i32)->i32>

fn id <(i32)->i32> (x):
    x

impl HashKey for NonCopyHashKey:
    fn eq <(NonCopyHashKey,NonCopyHashKey)->bool> (_a, _b):
        true

    fn hash32 <(NonCopyHashKey)->i32> (_self):
        0

fn close_hashmap_key <(HashMap<NonCopyHashKey, i32, DefaultHash32>)->()> (hm):
    free<NonCopyHashKey, i32, DefaultHash32> hm

fn main <()->i32> ():
    0
```

## hashmap_hasher_free_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *

struct StatefulHasher:
    seed <(i32)->i32>

fn id <(i32)->i32> (x):
    x

impl Hasher<i32> for StatefulHasher:
    fn hash32 <(StatefulHasher,i32)->i32> (_h, key):
        key

fn close_hashmap_hasher <(HashMap<i32, i32, StatefulHasher>)->()> (hm):
    free<i32, i32, StatefulHasher> hm

fn main <()->i32> ():
    0
```

## hashmap_root_facade_hides_storage_allocator

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *

fn main <()->i32> ():
    hashmap_alloc_storage 4
```

## hashset_root_facade_hides_storage_allocator

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset" as *

fn main <()->i32> ():
    hashset_alloc_storage 4
```

## bloom_filter_free_rejects_non_copy_hasher

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/bloom_filter" as *
#import "core/traits/hash" as *

struct StatefulHasher:
    seed <(i32)->i32>

fn id <(i32)->i32> (x):
    x

impl Hasher<i32> for StatefulHasher:
    fn hash32 <(StatefulHasher,i32)->i32> (_h, key):
        key

fn close_bloom_filter <(BloomFilter<i32, StatefulHasher>)->()> (bf):
    free<i32, StatefulHasher> bf

fn main <()->i32> ():
    0
```

## bloom_filter_clear_rejects_non_copy_hasher

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/bloom_filter" as *
#import "core/traits/hash" as *

struct StatefulHasher:
    seed <(i32)->i32>

fn id <(i32)->i32> (x):
    x

impl Hasher<i32> for StatefulHasher:
    fn hash32 <(StatefulHasher,i32)->i32> (_h, key):
        key

fn clear_bloom_filter <(BloomFilter<i32, StatefulHasher>)*>BloomFilter<i32, StatefulHasher>> (bf):
    clear<i32, StatefulHasher> bf

fn main <()->i32> ():
    0
```

## counting_bloom_filter_free_rejects_non_copy_hasher

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/counting_bloom_filter" as *
#import "core/traits/hash" as *

struct StatefulHasher:
    seed <(i32)->i32>

fn id <(i32)->i32> (x):
    x

impl Hasher<i32> for StatefulHasher:
    fn hash32 <(StatefulHasher,i32)->i32> (_h, key):
        key

fn close_counting_bloom_filter <(CountingBloomFilter<i32, StatefulHasher>)->()> (bf):
    free<i32, StatefulHasher> bf

fn main <()->i32> ():
    0
```

## counting_bloom_filter_clear_rejects_non_copy_hasher

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/counting_bloom_filter" as *
#import "core/traits/hash" as *

struct StatefulHasher:
    seed <(i32)->i32>

fn id <(i32)->i32> (x):
    x

impl Hasher<i32> for StatefulHasher:
    fn hash32 <(StatefulHasher,i32)->i32> (_h, key):
        key

fn clear_counting_bloom_filter <(CountingBloomFilter<i32, StatefulHasher>)*>CountingBloomFilter<i32, StatefulHasher>> (bf):
    clear<i32, StatefulHasher> bf

fn main <()->i32> ():
    0
```
