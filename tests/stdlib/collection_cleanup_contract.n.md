# collection cleanup contract

## 概要

[目的/もくてき]:
- 現行 collection のうち `Vec Option T` / `Vec T` storage に[載/の]る派生 collection は、`OwnedBuffer T` と element drop traversal が[完成/かんせい]するまで Copy-only 契約で[閉/と]じることを[確認/かくにん]します。
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
    value %i32

fn close_vec_clear %fn Vec CleanupPayload unit \v:
    let next %Vec CleanupPayload clear v
    unit

fn main %fn void i32 \void:
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
    value %i32

fn close_vec_free %fn Vec CleanupPayload unit \v:
    free v

fn main %fn void i32 \void:
    0
```

## vec_empty_allows_non_copy_payload_metadata_observation

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *

struct CleanupPayload:
    value %i32

fn main %fn void i32 \void:
    let v %Vec CleanupPayload vec_empty
    let ok %bool is_empty &v
    if ok 0 1
```

## vec_new_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/result" as *

struct CleanupPayload:
    value %i32

fn make_vec_new %fn void Result Vec CleanupPayload StdErrorKind \void:
    let r %Result Vec CleanupPayload StdErrorKind new
    r

fn main %fn void i32 \void:
    0
```

## vec_with_capacity_rejects_plain_payload_without_copy_or_drop

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/result" as *

struct CleanupPayload:
    value %i32

fn make_vec_with_capacity %fn void Result Vec CleanupPayload StdErrorKind \void:
    let r %Result Vec CleanupPayload StdErrorKind with_capacity 4
    r

fn main %fn void i32 \void:
    0
```

## vec_push_accepts_drop_payload

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/math" as *
#import "core/result" as *
#import "core/traits/drop" as *

struct DropPayload:
    value %i32

impl Drop for DropPayload:
    fn drop %impure fn &DropPayload unit \_self:
        unit

fn main %impure fn void i32 \void:
    let new_result %Result Vec DropPayload StdErrorKind new
    let v0 %Vec DropPayload unwrap_ok new_result
    let push_result %Result Vec DropPayload VecPushError DropPayload push v0 (DropPayload 7)
    let v1 %Vec DropPayload unwrap_ok push_result
    let ok %bool eq len &v1 1
    free v1
    if ok 0 1
```

## vec_push_rejects_plain_payload_without_copy_or_drop

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/result" as *

struct CleanupPayload:
    value %i32

fn push_plain_vec %fn Vec CleanupPayload fn CleanupPayload Result Vec CleanupPayload VecPushError CleanupPayload \v\item:
    push v item

fn main %fn void i32 \void:
    0
```

## vec_get_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "core/option" as *

struct CleanupPayload:
    value %i32

fn read_non_copy_vec %fn &Vec CleanupPayload fn i32 Option CleanupPayload \v\idx:
    get v idx

fn main %fn void i32 \void:
    0
```

## vec_len_allows_non_copy_payload_metadata_observation

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *

struct CleanupPayload:
    value %i32

fn read_non_copy_vec_len %fn &Vec CleanupPayload i32 \v:
    len v

fn main %fn void i32 \void:
    0
```

## vec_cap_allows_non_copy_payload_metadata_observation

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *

struct CleanupPayload:
    value %i32

fn read_non_copy_vec_cap %fn &Vec CleanupPayload i32 \v:
    cap v

fn main %fn void i32 \void:
    0
```

## vec_invariant_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "alloc/collections/vec/invariant" as *

struct CleanupPayload:
    value %i32

fn prove_non_copy_vec_invariant %fn &Vec CleanupPayload VecCopyInvariant \v:
    vec_current_copy_invariant v

fn main %fn void i32 \void:
    0
```

## vec_partition_len_allows_non_copy_payload_metadata_observation

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *

struct CleanupPayload:
    value %i32

fn read_non_copy_partition_len %fn &VecPartition CleanupPayload i32 \parts:
    vec_partition_matched_len parts

fn main %fn void i32 \void:
    0
```

## vec_pop_vec_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *

struct CleanupPayload:
    value %i32

fn recover_vec_from_pop %fn VecPop CleanupPayload Vec CleanupPayload \p:
    vec_pop_vec p

fn main %fn void i32 \void:
    0
```

## binary_heap_pop_heap_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *

struct CleanupPayload:
    value %i32

fn recover_heap_from_pop %fn BinaryHeapPop CleanupPayload BinaryHeap CleanupPayload \p:
    binary_heap_pop_heap p

fn main %fn void i32 \void:
    0
```

## vec_push_error_vec_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *

struct CleanupPayload:
    value %i32

fn recover_vec_from_push_error %fn VecPushError CleanupPayload Vec CleanupPayload \e:
    vec_push_error_vec e

fn main %fn void i32 \void:
    0
```

## vec_transform_error_vec_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *

struct CleanupPayload:
    value %i32

fn recover_vec_from_transform_error %fn VecTransformError CleanupPayload Vec CleanupPayload \e:
    vec_transform_error_vec e

fn main %fn void i32 \void:
    0
```

## vec_sort_merge_error_vec_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *
#import "alloc/collections/vec/sort/merge" as *

struct CleanupPayload:
    value %i32

fn recover_vec_from_sort_merge_error %fn VecSortMergeError CleanupPayload Vec CleanupPayload \e:
    vec_sort_merge_error_vec e

fn main %fn void i32 \void:
    0
```

## vec_root_facade_hides_alloc_empty_helper

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *

fn main %fn void i32 \void:
    vec_alloc_empty 4
```

## vec_root_facade_hides_storage_cleanup_helper

neplg2:test[compile_fail]
diag_code: resolve.identifier.undefined
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/vec" as *

fn main %fn void i32 \void:
    vec_free_storage VecStorage<i32>::Empty
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
    value %i32

fn close_stack %fn Stack CleanupPayload unit \s:
    free s

fn main %fn void i32 \void:
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
    value %i32

fn close_queue %fn Queue CleanupPayload unit \q:
    free q

fn main %fn void i32 \void:
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
    value %i32

fn close_deque %fn Deque CleanupPayload unit \dq:
    free dq

fn main %fn void i32 \void:
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
    value %i32

fn close_ringbuffer %fn RingBuffer CleanupPayload unit \rb:
    free rb

fn main %fn void i32 \void:
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
    value %i32

fn close_binary_heap %fn BinaryHeap CleanupPayload unit \heap:
    free heap

fn main %fn void i32 \void:
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
    value %i32

fn close_btreeset %fn BTreeSet CleanupPayload unit \set0:
    free set0

fn main %fn void i32 \void:
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
    value %i32

fn close_btreemap_key %fn BTreeMap CleanupPayload i32 unit \hm:
    free hm

fn main %fn void i32 \void:
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
    value %i32

fn close_btreemap_value %fn BTreeMap i32 CleanupPayload unit \hm:
    free hm

fn main %fn void i32 \void:
    0
```

## btreemap_insert_value_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *
#import "core/result" as *

struct CleanupPayload:
    value %i32

fn insert_btreemap_value %impure fn BTreeMap i32 CleanupPayload impure fn CleanupPayload Result BTreeMap i32 CleanupPayload BTreeMapInsertError i32 CleanupPayload \hm\value:
    insert hm 1 value

fn main %fn void i32 \void:
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
    value %i32

fn close_list %fn List CleanupPayload unit \lst:
    free lst

fn main %fn void i32 \void:
    0
```

## queue_len_allows_non_copy_payload_metadata_observation

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/queue" as *

struct CleanupPayload:
    value %i32

fn read_queue_len %fn &Queue CleanupPayload i32 \q:
    len q

fn main %fn void i32 \void:
    0
```

## deque_len_allows_non_copy_payload_metadata_observation

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/deque" as *

struct CleanupPayload:
    value %i32

fn read_deque_len %fn &Deque CleanupPayload i32 \dq:
    len dq

fn main %fn void i32 \void:
    0
```

## binary_heap_len_allows_non_copy_payload_metadata_observation

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/binary_heap" as *

struct CleanupPayload:
    value %i32

fn read_heap_len %fn &BinaryHeap CleanupPayload i32 \heap:
    len heap

fn main %fn void i32 \void:
    0
```

## list_len_allows_non_copy_payload_metadata_observation

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/list" as *

struct CleanupPayload:
    value %i32

fn read_list_len %fn &List CleanupPayload i32 \lst:
    len lst

fn main %fn void i32 \void:
    0
```

## list_transform_error_list_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/list" as *

struct CleanupPayload:
    value %i32

fn recover_list_from_transform_error %fn ListTransformError CleanupPayload List CleanupPayload \e:
    list_transform_error_list e

fn main %fn void i32 \void:
    0
```

## btreemap_len_allows_non_copy_value_metadata_observation

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap" as *

struct CleanupPayload:
    value %i32

fn read_btreemap_len %fn &BTreeMap i32 CleanupPayload i32 \hm:
    len hm

fn main %fn void i32 \void:
    0
```

## hashmap_len_allows_non_copy_value_metadata_observation

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *

struct CleanupPayload:
    value %i32

fn read_hashmap_len %fn &HashMap i32 CleanupPayload DefaultHash32 i32 \hm:
    len hm

fn main %fn void i32 \void:
    0
```

## hashmap_insert_value_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *
#import "core/result" as *

struct CleanupPayload:
    value %i32

fn insert_hashmap_value %impure fn HashMap i32 CleanupPayload DefaultHash32 impure fn CleanupPayload Result HashMap i32 CleanupPayload DefaultHash32 HashMapUpdateError i32 CleanupPayload DefaultHash32 \hm\value:
    insert hm 1 value

fn main %fn void i32 \void:
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
    value %i32

fn close_hashmap_value %fn HashMap i32 CleanupPayload DefaultHash32 unit \hm:
    free hm

fn main %fn void i32 \void:
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
    seed %fn i32 i32

fn id %fn i32 i32 \x:
    x

impl HashKey for NonCopyHashKey:
    fn eq %fn NonCopyHashKey fn NonCopyHashKey bool \_a\_b:
        true

    fn hash32 %fn NonCopyHashKey i32 \_self:
        0

fn close_hashset_key %fn HashSet NonCopyHashKey DefaultHash32 unit \hs:
    free hs

fn main %fn void i32 \void:
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
    seed %fn i32 i32

fn id %fn i32 i32 \x:
    x

impl Hasher<i32> for StatefulHasher:
    fn hash32 %fn StatefulHasher fn i32 i32 \_h\key:
        key

fn close_hashset_hasher %fn HashSet i32 StatefulHasher unit \hs:
    free hs

fn main %fn void i32 \void:
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
    seed %fn i32 i32

fn id %fn i32 i32 \x:
    x

impl HashKey for NonCopyHashKey:
    fn eq %fn NonCopyHashKey fn NonCopyHashKey bool \_a\_b:
        true

    fn hash32 %fn NonCopyHashKey i32 \_self:
        0

fn close_hashmap_key %fn HashMap NonCopyHashKey i32 DefaultHash32 unit \hm:
    free hm

fn main %fn void i32 \void:
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
    seed %fn i32 i32

fn id %fn i32 i32 \x:
    x

impl Hasher<i32> for StatefulHasher:
    fn hash32 %fn StatefulHasher fn i32 i32 \_h\key:
        key

fn close_hashmap_hasher %fn HashMap i32 i32 StatefulHasher unit \hm:
    free hm

fn main %fn void i32 \void:
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

fn main %fn void i32 \void:
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

fn main %fn void i32 \void:
    hashset_alloc_storage 4
```

## btreemap_storage_key_view_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap/storage" as *
#import "alloc/collections/btreemap/types" as *
#import "alloc/collections/vec" as *
#import "core/option" as *

struct CleanupPayload:
    value %i32

fn borrow_btreemap_keys %fn &BTreeMapStorage CleanupPayload i32 &Vec Option CleanupPayload \storage:
    btreemap_storage_keys storage

fn main %fn void i32 \void:
    0
```

## btreemap_storage_value_view_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreemap/storage" as *
#import "alloc/collections/btreemap/types" as *
#import "alloc/collections/vec" as *
#import "core/option" as *

struct CleanupPayload:
    value %i32

fn borrow_btreemap_values %fn &BTreeMapStorage i32 CleanupPayload &Vec Option CleanupPayload \storage:
    btreemap_storage_values storage

fn main %fn void i32 \void:
    0
```

## btreeset_storage_key_view_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/btreeset/storage" as *
#import "alloc/collections/btreeset/types" as *
#import "alloc/collections/vec" as *
#import "core/option" as *

struct CleanupPayload:
    value %i32

fn borrow_btreeset_keys %fn &BTreeSetStorage CleanupPayload &Vec Option CleanupPayload \storage:
    btreeset_storage_keys storage

fn main %fn void i32 \void:
    0
```

## hashmap_storage_key_view_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap/storage" as *
#import "alloc/collections/hashmap/types" as *
#import "alloc/collections/vec" as *
#import "core/option" as *

struct CleanupPayload:
    value %i32

fn borrow_hashmap_keys %fn &HashMapStorage CleanupPayload i32 &Vec Option CleanupPayload \storage:
    hashmap_storage_keys storage

fn main %fn void i32 \void:
    0
```

## hashmap_storage_value_view_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashmap/storage" as *
#import "alloc/collections/hashmap/types" as *
#import "alloc/collections/vec" as *
#import "core/option" as *

struct CleanupPayload:
    value %i32

fn borrow_hashmap_values %fn &HashMapStorage i32 CleanupPayload &Vec Option CleanupPayload \storage:
    hashmap_storage_values storage

fn main %fn void i32 \void:
    0
```

## hashset_storage_key_view_rejects_non_copy_payload

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target std

#import "alloc/collections/hashset/storage" as *
#import "alloc/collections/hashset/types" as *
#import "alloc/collections/vec" as *
#import "core/option" as *

struct CleanupPayload:
    value %i32

fn borrow_hashset_keys %fn &HashSetStorage CleanupPayload &Vec Option CleanupPayload \storage:
    hashset_storage_keys storage

fn main %fn void i32 \void:
    0
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
    seed %fn i32 i32

fn id %fn i32 i32 \x:
    x

impl Hasher<i32> for StatefulHasher:
    fn hash32 %fn StatefulHasher fn i32 i32 \_h\key:
        key

fn close_bloom_filter %fn BloomFilter i32 StatefulHasher unit \bf:
    free bf

fn main %fn void i32 \void:
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
    seed %fn i32 i32

fn id %fn i32 i32 \x:
    x

impl Hasher<i32> for StatefulHasher:
    fn hash32 %fn StatefulHasher fn i32 i32 \_h\key:
        key

fn clear_bloom_filter %impure fn BloomFilter i32 StatefulHasher BloomFilter i32 StatefulHasher \bf:
    clear bf

fn main %fn void i32 \void:
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
    seed %fn i32 i32

fn id %fn i32 i32 \x:
    x

impl Hasher<i32> for StatefulHasher:
    fn hash32 %fn StatefulHasher fn i32 i32 \_h\key:
        key

fn close_counting_bloom_filter %fn CountingBloomFilter i32 StatefulHasher unit \bf:
    free bf

fn main %fn void i32 \void:
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
    seed %fn i32 i32

fn id %fn i32 i32 \x:
    x

impl Hasher<i32> for StatefulHasher:
    fn hash32 %fn StatefulHasher fn i32 i32 \_h\key:
        key

fn clear_counting_bloom_filter %impure fn CountingBloomFilter i32 StatefulHasher CountingBloomFilter i32 StatefulHasher \bf:
    clear bf

fn main %fn void i32 \void:
    0
```
