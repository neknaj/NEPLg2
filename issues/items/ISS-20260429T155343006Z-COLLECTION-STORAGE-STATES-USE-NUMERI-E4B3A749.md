---
id: ISS-20260429T155343006Z-COLLECTION-STORAGE-STATES-USE-NUMERI-E4B3A749
title: "collection storage states use numeric/null sentinels instead of enum owner state"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-30
target: "stdlib/alloc/collections/hashmap.nepl, stdlib/alloc/collections/hashset.nepl, stdlib/alloc/collections/**"
---

# ISS-20260429T155343006Z-COLLECTION-STORAGE-STATES-USE-NUMERI-E4B3A749: collection storage states use numeric/null sentinels instead of enum owner state

## 概要

HashMap/HashSet bucket state is encoded as 0/1/2 and many collection storage owners use null MemPtr sentinels or raw header addresses. These states are safety-relevant but are not represented as enum/typed owner wrappers, so match exhaustiveness and Resource IR ownership checks cannot share the same structural invariant.

## 対象

- `stdlib/alloc/collections/hashmap.nepl, stdlib/alloc/collections/hashset.nepl, stdlib/alloc/collections/**`

## 根拠

- `stdlib/alloc/collections/hashmap.nepl` は module comment と implementation の両方で bucket state を `0 = empty`, `1 = full`, `2 = tombstone` として扱い、`load_i32` / `store_i32` で status を読み書きしている。
- `stdlib/alloc/collections/hashset.nepl` も同じ 0/1/2 status discipline を持つ。
- `stdlib/alloc/collections/vec.nepl` は `Vec<T>` の storage owner を `data <MemPtr<T>>` として持ち、`with_capacity 0` / `filled n<=0` では `mem_ptr_wrap 0` を empty sentinel として返す。
- `stdlib/alloc/collections/stack.nepl` / `binary_heap.nepl` などは header raw storage に len/cap/data pointer を詰め、header pointer 自体が owner token と raw layout を兼ねる。
- `ByteBuf` / `ByteBuilder` / `StringBuilder` は同根の null owning pointer 問題を `Option<MemPtr<u8>>` へ移して改善済みであり、collection だけ古い pattern が残っている。
- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` のレビューでは、self-host の typecheck / ResourceIR stage がこの numeric/null storage discipline を引き継ぐべきでないと整理した。

## 問題

HashMap/HashSet bucket state is encoded as 0/1/2 and many collection storage owners use null MemPtr sentinels or raw header addresses. These states are safety-relevant but are not represented as enum/typed owner wrappers, so match exhaustiveness and Resource IR ownership checks cannot share the same structural invariant.

## 影響

Self-host type/resource stages would inherit magic-number storage discipline, making memory safety dependent on comments and source policy instead of static checks. Adding more Resource IR aliases would hide the stdlib design problem rather than fixing it.

## 修正方針

Introduce enum-based storage and bucket states, e.g. StorageState/OwnedBuffer and BucketState Empty/Full/Tombstone, then migrate Vec-derived and hash collections so owner presence, initialized payload, tombstone, and empty states are visible to typecheck and Resource IR.

## 検証

Add source policy tests rejecting numeric bucket state comments/branches and null owning MemPtr sentinels in collection public storage. Add doctests/compile_fail cases for exhaustive BucketState match, owner-preserving fallible update failures, and non-Copy payload cleanup before storage free.

## 2026-04-30 HashMap 部分進捗

`stdlib/alloc/collections/hashmap.nepl` は旧 raw header + `0/1/2` status layout を廃止し、`HashMapBucketState` enum と `HashMapStorage<K,V>` へ移行した。

進捗:

- HashMap bucket state は `Empty` / `Full` / `Tombstone` の enum になり、探索・挿入・rehash の分岐は `match` で網羅的に扱う。
- key/value payload は `Vec<Option<K>>` / `Vec<Option<V>>` で初期化済み状態を表す。
- HashMap の raw header pointer と entries pointer は public storage から消えた。
- `cargo test -p nepl-core --test neplg2 -- --nocapture` と HashMap focused `.n.md` は通過した。

この時点の残件:

- `stdlib/alloc/collections/hashset.nepl` の同根問題は 2026-04-30 の HashSet 部分進捗で typed storage へ移行済み。
- Vec / Stack / BinaryHeap などの null/raw owner sentinel 設計は別途段階的に typed owner state へ移す必要がある。

## 2026-04-30 HashSet 部分進捗

`stdlib/alloc/collections/hashset.nepl` も旧 raw header + `0/1/2` status layout を廃止し、`HashSetBucketState` enum と `HashSetStorage<T>` へ移行した。

進捗:

- HashSet bucket state は `Empty` / `Full` / `Tombstone` の enum になり、探索・挿入・rehash の分岐は `match` で網羅的に扱う。
- key payload は `Vec<Option<T>>` で初期化済み状態を表す。
- HashSet の raw header pointer と entries pointer は public storage から消えた。
- `contains` / `len` は `&HashSet` を受け取る read API に揃え、fixture は観測後に `free` するよう更新した。
- `stdlib/tests/hashset.n.md` / `stdlib/tests/hashset_str.n.md`、`tests/stdlib/hash_collection_rehash.n.md`、`tests/stdlib/traits_hash.n.md` の HashSet focused tests は通過した。

残件:

- Vec / Stack / BinaryHeap などの null/raw owner sentinel 設計は別途段階的に typed owner state へ移す必要がある。
- `tests/stdlib/collections_diag.n.md` の HashMap/HashSet missing-key diagnostic fixture は `Diag.message` owner contract 問題で失敗する。これは `ISS-20260429T190939510Z-DIAG-IS-COPY-WHILE-CARRYING-OWNED-ST-F1284BFF` へ分離済み。

## 2026-04-30 HashMap source policy 追加

`nodesrc/test_stdlib_hashmap_storage_contract.js` を追加し、HashMap が typed storage から raw header / numeric sentinel へ戻らないことを CI の source policy として固定した。

固定した契約:

- bucket state は `HashMapBucketState::Empty/Full/Tombstone` enum で表す。
- insertion slot state は `HashMapInsertSlotState::EmptySlot/TombstoneSlot` enum で表す。
- backing storage は `HashMapStorage<K,V>` の `Vec<HashMapBucketState>` / `Vec<Option<K>>` / `Vec<Option<V>>` owner として保持する。
- HashMap 本体は `count/cap/tombstones/storage/hasher` を持ち、`MemPtr` / `alloc_raw` / `load_i32` / `store_i32` を HashMap 実装に戻さない。
- lookup / insertion slot search は `match` で bucket state を網羅する。
- `get` / `contains` / `len` は `&HashMap` を受け取る read API のままにする。
- rehash / insert / free は storage owner を明示的に移動または解放する。

検証:

- `node nodesrc/test_stdlib_hashmap_storage_contract.js`: passed

## 2026-04-30 HashSet 部分進捗

`stdlib/alloc/collections/hashset.nepl` は旧 raw header + entries layout と `0/1/2` bucket status を廃止し、`HashSetBucketState` enum と `HashSetStorage<T>` へ移行した。

進捗:

- HashSet bucket state は `Empty` / `Full` / `Tombstone` の enum になり、探索・挿入 slot 探索・rehash は `match` で網羅的に扱う。
- key payload は `Vec<Option<T>>` で初期化済み状態を表す。
- HashSet 本体は `count/cap/tombstones/storage/hasher` を直接持ち、raw header pointer と entries pointer を持たない。
- `contains` / `len` は `&HashSet` を受け取る read API になり、読み取りで storage owner を移動しない。
- `remove` の missing key path は入力 HashSet を消費し、storage を解放してから `Err(Diag)` を返す。
- HashSet call site と doctest は borrow read API と明示 `free` に合わせて更新した。

残件:

- `Vec` / `Queue` / `Deque` / `BinaryHeap` などの null/raw owner sentinel 設計は引き続き typed owner state へ移す必要がある。
- HashMap / HashSet は Copy payload 前提であり、非 Copy payload の drop traversal は別設計で扱う。
- `tests/stdlib/collections_diag.n.md` の HashMap/HashSet missing-key diagnostic fixture は `Diag.message` owner contract 問題で失敗する。これは `ISS-20260429T190939510Z-DIAG-IS-COPY-WHILE-CARRYING-OWNED-ST-F1284BFF` へ分離済み。

## 2026-04-30 HashSet source policy 追加

`nodesrc/test_stdlib_hashset_storage_contract.js` を追加し、HashSet が typed storage から raw header / numeric sentinel へ戻らないことを CI の source policy として固定した。

固定した契約:

- bucket state は `HashSetBucketState::Empty/Full/Tombstone` enum で表す。
- insertion slot state は `HashSetInsertSlotState::EmptySlot/TombstoneSlot` enum で表す。
- backing storage は `HashSetStorage<T>` の `Vec<HashSetBucketState>` / `Vec<Option<T>>` owner として保持する。
- HashSet 本体は `count/cap/tombstones/storage/hasher` を持ち、`MemPtr` / `alloc_raw` / `load_i32` / `store_i32` を HashSet 実装に戻さない。
- lookup / insertion slot search は `match` で bucket state を網羅する。
- `contains` / `len` は `&HashSet` を受け取る read API のままにする。
- rehash / insert / free は storage owner を明示的に移動または解放する。

検証:

- `node nodesrc/test_stdlib_hashset_storage_contract.js`: passed
- `node nodesrc/tests.js -i stdlib/tests/hashset.n.md -i stdlib/tests/hashset_str.n.md --no-tree -o tmp/hashset-main-after-rebase.json -j 1 --dist web/dist`: total=4, passed=4
- `node nodesrc/run_doctest.js -i tests/stdlib/hash_collection_rehash.n.md -n 2 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/hash_collection_rehash.n.md -n 4 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/hash_collection_rehash.n.md -n 6 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/pipe_collections.n.md -n 6 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/traits_hash.n.md -n 6 --dist web/dist`: passed
- `node nodesrc/issues.js check`: passed

## 2026-04-30 Queue / RingBuffer 部分進捗

`stdlib/alloc/collections/queue.nepl` と `stdlib/alloc/collections/ringbuffer.nepl` は、旧 `[len, cap, head, data_ptr]` raw header と未初期化 element buffer を廃止し、typed `Vec<Option<T>>` storage へ移行した。

進捗:

- Queue / RingBuffer 本体は `len/cap/head/items` を直接持ち、`items` は `Vec<Option<T>>` として全 slot を初期化する。
- live slot は `Some(value)`、inactive slot は `None` で表すため、先頭取得・grow・clear が raw memory cell の初期化状態に依存しない。
- payload は現行 stdlib の drop traversal 未整備に合わせて `.T: Copy` に限定した。非 Copy payload は collection-wide drop 設計で扱う。
- `pop_front` は `QueuePop<T>` / `RingBufferPop<T>` として更新後 owner と `Option<T>` を同時に返し、取り出した slot を `None` に戻す。
- terminal `pop` / `peek` / `len` / `cap` / `is_empty` は by-value で owner を消費し、内部 storage を閉じる。owner を残す読み取りは `*_ref` API を使う。
- `nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js` と `nodesrc/test_stdlib_ringbuffer_no_unsafe_unwraps.js` を更新し、Queue / RingBuffer が raw header / raw element storage へ戻らないことを source policy で固定した。

検証:

- `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 3 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/collections_diag.n.md -n 4 --dist web/dist`: passed
- `node nodesrc/tests.js -i tests/stdlib/collections_diag.n.md --no-tree -o tmp/collections-diag-queue-ringbuffer-typed-storage.json -j 1 --dist web/dist`: total=4, passed=4
- `node nodesrc/tests.js -i stdlib/tests/queue.n.md -i stdlib/tests/ringbuffer.n.md --no-tree -o tmp/queue-ringbuffer-pop-front-regression.json -j 1 --dist web/dist`: total=4, passed=4
- `node nodesrc/tests.js -i tests/stdlib/queue_collections.n.md -i tests/stdlib/ringbuffer_collections.n.md --no-tree -o tmp/queue-ringbuffer-pop-front-tests.json -j 1 --dist web/dist`: total=4, passed=4
- `node nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_ringbuffer_no_unsafe_unwraps.js`: passed

残件:

- `Vec` / `Stack` / `BinaryHeap` などの raw owner sentinel 設計は引き続き typed owner state へ移す必要がある。
- `tests/stdlib/pipe_collections.n.md` 全体では Stack / BTreeMap / BTreeSet の既存 raw owner state 残件が残る。Queue / RingBuffer の focused doctest は passed。

## 2026-04-30 Stack 部分進捗

`stdlib/alloc/collections/stack.nepl` は、旧 raw header + data pointer owner を廃止し、typed `Vec<Option<T>>` storage へ移行した。

進捗:

- Stack 本体は `len/cap/items` を直接持ち、`items` は `Vec<Option<T>>` として全 slot を初期化する。
- live slot は `Some(value)`、inactive slot は `None` で表すため、push / pop / clear / grow が raw memory cell の初期化状態に依存しない。
- payload は現行 stdlib の drop traversal 未整備に合わせて `.T: Copy` に限定した。
- `StackPop<T>` と `pop_top` を追加し、更新後 owner と `Option<T>` を同時に返せるようにした。
- terminal `pop` / `peek` / `len` / `is_empty` は by-value で owner を消費し、内部 storage を閉じる。owner を残す読み取りは `*_ref` API を使う。
- 旧 `push_ref` / `pop_ref` の borrowed destructive update は raw header に依存していたため廃止し、owner を返す `push` / `pop_top` を正規 API にした。
- `nodesrc/test_stdlib_stack_no_unsafe_unwraps.js` を更新し、Stack が raw header / raw element storage へ戻らないことを source policy で固定した。

検証:

- `node nodesrc/run_doctest.js -i tests/stdlib/pipe_collections.n.md -n 2 --dist web/dist`: passed
- `node nodesrc/tests.js -i stdlib/tests/stack.n.md --no-tree -o tmp/stack-typed-storage-stdlib.json -j 1 --dist web/dist`: total=9, passed=9
- `node nodesrc/tests.js -i tests/stdlib/stack_collections.n.md --no-tree -o tmp/stack-typed-storage-tests.json -j 1 --dist web/dist`: total=9, passed=9
- `node nodesrc/test_stdlib_stack_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/pipe-collections-after-stack-typed-storage.json -j 1 --dist web/dist`: total=8, passed=6, failed=2

残件:

- `BTreeMap` / `BTreeSet` の raw storage owner state が `tests/stdlib/pipe_collections.n.md` で残る。
- `Deque` / `Vec` / `BinaryHeap` などの raw owner sentinel 設計は引き続き typed owner state へ移す必要がある。

## 2026-04-30 Deque 部分進捗

`stdlib/alloc/collections/deque.nepl` は旧 `[len, cap, head, data_ptr]` raw header と raw element buffer を廃止し、typed `Vec<Option<T>>` storage へ移行した。

進捗:

- Deque 本体は `len/cap/head/items` を直接持ち、`items` は `Vec<Option<T>>` として全 slot を初期化する。
- live slot は `Some(value)`、inactive slot は `None` で表すため、front/back 取得・grow・clear が raw memory cell の初期化状態に依存しない。
- payload は現行 stdlib の drop traversal 未整備に合わせて `.T: Copy` に限定した。非 Copy payload は collection-wide drop 設計で扱う。
- terminal `pop_front` / `pop_back` / `peek_front` / `peek_back` / `len` / `cap` / `is_empty` は by-value で owner を消費し、内部 storage を閉じる。owner を残す読み取りは `*_ref` API を使う。
- `nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js` を更新し、Deque が raw header / raw element storage へ戻らないことを source policy で固定した。

検証:

- `node nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/tests/deque.n.md -i tests/stdlib/deque_collections.n.md --no-tree -o tmp/deque-typed-storage-focused.json -j 1 --dist web/dist`: `total=4`, `passed=4`
- `node nodesrc/tests.js -i stdlib/alloc/collections/deque.nepl --no-tree -o tmp/deque-typed-storage-docs.json -j 1 --dist web/dist`: `total=2`, `passed=2`

残件:

- `BTreeMap` / `BTreeSet` の raw storage owner state は後続の BTreeMap / BTreeSet 部分進捗で typed storage へ移行済み。
- `Vec` / `BinaryHeap` などの raw owner sentinel 設計は引き続き typed owner state へ移す必要がある。
- Deque は Copy payload 前提であり、非 Copy payload の drop traversal は collection-wide drop 設計で扱う。

## 2026-04-30 BTreeMap / BTreeSet 部分進捗

`stdlib/alloc/collections/btreemap.nepl` と `stdlib/alloc/collections/btreeset.nepl` は、旧 raw header + raw key/value pointer layout を廃止し、typed `Vec<Option<T>>` storage へ移行した。

進捗:

- `BTreeMapStorage<K,V>` / `BTreeSetStorage<T>` を追加し、backing storage owner を struct field として保持するようにした。
- live slot は `Some(value)`、inactive slot は `None` で表し、lower_bound / insert shift / remove shift / clear は `match` で slot state を扱う。
- payload は現行 stdlib の drop traversal 未整備に合わせて `.K: Copy`, `.V: Copy`, `.T: Copy` に限定した。
- `len_ref` / `contains_ref` / `get_ref` を追加し、owner を残す読み取りと terminal by-value 読み取りを分離した。
- by-value `len` / `contains` / `get` は観測後に storage を `free` して owner を閉じる。
- `nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js` を更新し、BTreeMap / BTreeSet が raw header / raw pointer storage へ戻らないことを source policy で固定した。

検証:

- `node nodesrc/test_stdlib_btree_insert_no_unsafe_grow_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/btreemap.nepl --no-tree -o tmp/btreemap-typed-storage-docs.json -j 1 --dist web/dist`: total=8, passed=8
- `node nodesrc/tests.js -i stdlib/alloc/collections/btreeset.nepl --no-tree -o tmp/btreeset-typed-storage-docs.json -j 1 --dist web/dist`: total=7, passed=7
- `node nodesrc/tests.js -i stdlib/tests/btreemap.n.md --no-tree -o tmp/btreemap-typed-storage-tests.json -j 1 --dist web/dist`: total=5, passed=5
- `node nodesrc/tests.js -i stdlib/tests/btreeset.n.md --no-tree -o tmp/btreeset-typed-storage-tests.json -j 1 --dist web/dist`: total=5, passed=5
- `node nodesrc/tests.js -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/pipe-collections-btree-typed-storage.json -j 1 --dist web/dist`: total=8, passed=8

残件:

- `Vec` / `BinaryHeap` などの raw owner sentinel 設計は引き続き typed owner state へ移す必要がある。
- `tests.js` を複数プロセスで同時実行したとき、partial JSON のまま終了する harness 問題を `ISS-20260429T210219258Z-TESTS-JS-CONCURRENT-RUNS-CAN-LEAVE-P-77B8E3E7` として分離した。
