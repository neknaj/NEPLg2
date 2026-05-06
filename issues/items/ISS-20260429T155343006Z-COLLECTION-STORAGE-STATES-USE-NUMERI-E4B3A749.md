---
id: ISS-20260429T155343006Z-COLLECTION-STORAGE-STATES-USE-NUMERI-E4B3A749
title: "collection storage states use numeric/null sentinels instead of enum owner state"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-05-06
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

## 2026-04-30 Vec negative capacity guard 部分進捗

`stdlib/alloc/collections/vec.nepl` は raw `MemPtr<T>` storage をまだ持つが、public capacity API の境界で負 capacity が allocator に到達しないようにした。

進捗:

- `with_capacity` は `cap < 0` を allocation 前に `StdErrorKind::InvalidOperation` として拒否する。
- `cap = 0` は従来どおり empty Vec を返し、`cap > 0` のみ allocator へ進む。
- `tests/stdlib/vec_collections.n.md` に negative capacity regression を追加した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` で negative capacity guard を source policy として固定した。

検証:

- `node nodesrc/tests.js -i tests/stdlib/vec_collections.n.md --no-tree -o tmp/vec-negative-capacity-main-merge.json -j 1 --dist web/dist`: total=3, passed=3
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/issues.js check`: passed (`files=440`)
- `git diff --check`: passed

残件:

- `Vec` 本体の `MemPtr<T>` / `mem_ptr_wrap 0` storage design は未解決であり、最終的には typed owner state へ移す必要がある。
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

## 2026-04-30 BinaryHeap 部分進捗

`stdlib/alloc/collections/binary_heap.nepl` は旧 raw header + raw element pointer layout を廃止し、typed `Vec<Option<T>>` storage へ移行した。

進捗:

- BinaryHeap 本体は `len/cap/items` を直接持ち、`items` は `Vec<Option<T>>` として全 slot を初期化する。
- live slot は `Some(value)`、inactive slot は `None` で表すため、push / grow / pop_max / sift が raw memory cell の初期化状態に依存しない。
- payload は現行 stdlib の drop traversal 未整備に合わせて `.T: Copy` に限定した。非 Copy payload は collection-wide drop 設計で扱う。
- `BinaryHeapPop<T>` と `pop_max` を追加し、更新後 owner と取り出した `Option<T>` を同時に返せるようにした。
- terminal `len` / `cap` / `is_empty` / `peek` / `pop` は by-value で owner を消費し、内部 storage を閉じる。owner を残す読み取りは `*_ref` API を使う。
- `new` / `with_capacity` / `push` / `pop_max` / `pop` は allocation や slot replacement を伴うため impure API として明示し、fallible API は collection 側の現在の契約に合わせて `Result<_, Diag>` を返す。
- `heap_sift_down` は初期化済み slot を `match` で確認しながら左右の子を直接比較する形にし、best-index の不要な中間値と if 式による曖昧な Resource IR 経路をなくした。
- `nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js` を更新し、BinaryHeap が raw header / raw element storage へ戻らないことを source policy で固定した。

検証:

- `node nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/binary_heap.nepl --no-tree -o tmp/binary-heap-typed-storage-docs.json -j 1 --dist web/dist`: total=6, passed=6
- `node nodesrc/tests.js -i stdlib/tests/binary_heap.n.md --no-tree -o tmp/binary-heap-typed-storage-tests.json -j 1 --dist web/dist`: total=5, passed=5
- `node nodesrc/tests.js -i tests/stdlib/binary_heap_collections.n.md --no-tree -o tmp/binary-heap-collections-typed-storage.json -j 1 --dist web/dist`: total=3, passed=3
- `node nodesrc/tests.js -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/pipe-collections-after-binary-heap.json -j 1 --dist web/dist`: total=8, passed=8

残件:

- `Vec` の null/raw owner sentinel 設計は引き続き typed owner state へ移す必要がある。
- typed `Vec<Option<T>>` storage に移行した collection は Copy payload 前提であり、非 Copy payload の drop traversal は collection-wide drop 設計で扱う。

## 2026-04-30 stdlib collection/mem/string 静的検査再レビュー

`doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` を remote main `bbaf2a5` 基準で更新し、collection / mem / string と Resource IR の現状、理想設計、self-host 利用制限を再整理した。

判定:

- `HashMap` / `HashSet` は enum bucket state と typed storage へ移行済みで、数値 status への退行を source policy で防げている。
- `Queue` / `Deque` / `RingBuffer` / `Stack` / `BinaryHeap` / `BTreeMap` / `BTreeSet` は raw header / raw pointer layout から `Vec<Option<T>>` slot state へ移行済みである。
- `Vec` は依然として `len/cap/data: MemPtr<T>` と `mem_ptr_wrap 0` に依存しており、collection storage owner state の根本残件である。
- `List` は owner flow が改善されたが、node chain 自体は raw address discipline に残る。
- bitset / bloom / adjacency matrix / Fenwick / SegmentTree / DisjointSet / SparseSet は Copy payload 中心だが、storage owner は `MemPtr` / raw pointer に残る。
- `core/mem` と string / byte builder は過渡的に改善しているが、`MemPtr` が owner と view を兼ねる設計を最終形としては採用しない。

この issue の残件は `Vec` / raw byte or numeric collection / List node storage の owner state を、`OwnedBuffer<T>` / `OwnedBytes` / `StorageState<T>` / enum + `match` へ移すことに絞る。

## 2026-04-30 SparseSet 部分進捗

`stdlib/alloc/collections/sparse_set.nepl` は旧 `hdr <i32>` raw header と header 内 raw dense/sparse address layout を廃止し、`n/len0/dense/sparse` typed fields へ移行した。

進捗:

- `SparseSet` 本体は `n`, `len0`, `dense`, `sparse` を直接持ち、dense/sparse owner を ResourceIR が field owner として追跡できる形にした。
- `len` / `universe_len` / `contains` は `&SparseSet` を受け取る read API に揃え、読み取りで storage owner を移動しない。
- `insert` / `remove` / `clear` は consumed owner の fields を値で取り出し、成功時は新しい `SparseSet` へ owner を移す。
- `insert` / `remove` の範囲外 Err path は consumed owner の dense/sparse storage を `sparse_set_free_arrays` で解放してから `Err(Diag)` を返す。
- `nodesrc/test_stdlib_sparse_set_no_unsafe_unwraps.js` と `nodesrc/test_stdlib_sparse_set_borrowed_observers.js` で raw header 回帰と by-value observer 回帰を防止する。

検証:

- `node nodesrc/tests.js -i stdlib/tests/sparse_set.n.md --no-tree -o tmp/sparse-set-stdlib-borrowed-observers.json -j 1`: total=2, passed=2
- `node nodesrc/tests.js -i tests/stdlib/sparse_set_collections.n.md --no-tree -o tmp/sparse-set-collections-borrowed-observers.json -j 1`: total=3, passed=3
- `node nodesrc/tests.js -i stdlib/alloc/collections/sparse_set.nepl --no-tree -o tmp/sparse-set-doctest-borrowed-observers.json -j 1`: total=7, passed=7
- `node nodesrc/test_stdlib_sparse_set_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_sparse_set_borrowed_observers.js`: passed

残件:

- `SparseSet` の dense/sparse payload は引き続き raw `MemPtr<i32>` array であり、最終的な `OwnedBuffer<i32>` / typed buffer state 設計へ移す余地がある。
- `Vec` / raw byte collection / List node storage の owner state は引き続き残る。

## 2026-05-06 Vec / sort 部分進捗

`stdlib/alloc/collections/vec.nepl` は `cap = 0` の empty owner state を `mem_ptr_wrap 0` だけで表す設計をやめ、`VecStorageState::{Empty, Owned}` を追加した。

進捗:

- `Vec` 本体は `len/cap/storage/data` を持つ。owner state は `VecStorageState` enum、実 pointer は `data <MemPtr<T>>` として分離した。
- enum payload に `MemPtr<T>` を入れる案は Resource IR が raw memory cell 初期化状態を追えなかったため破棄した。最終設計は enum で owner state を静的に分岐しつつ、pointer field は Resource IR が追跡できる直接 field として残す。
- `new` / `with_capacity` / `filled` / `push` / `pop` / `clear` / `free` / `map` / `filter` / `partition` / `take_while` / `drop_while` は `VecStorageState` の `match` と `vec_free_storage` を通る。
- `diag` / `kpgraph` / `tui` / `std/fs` / selfhost text の push failure fallback は `v::vec_empty<T>` へ統一し、`v::Vec<T> 0 0 mem_ptr_wrap 0` の直接構築を廃止した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` は、`VecStorageState` と `Vec` layout、direct null owner sentinel 禁止、`VecStorageState` による cleanup を固定するよう更新した。
- `sort` の in-place API は owner を消費しない `&Vec<T>` へ移し、`sort_i32` / raw slice helper も raw `i32` address ではなく `MemPtr<T>` を受け取る形にした。

検証:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl --no-tree -o tmp/vec-storage-state-docs2.json -j 1`: total=37, passed=37
- `node nodesrc/tests.js -i tests/stdlib/vec_collections.n.md --no-tree -o tmp/vec-storage-state-tests2.json -j 1`: total=3, passed=3
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort.nepl --no-tree -o tmp/vec-storage-state-sort-docs4.json -j 1`: total=3, passed=3
- `node nodesrc/tests.js -i tests/stdlib/sort.n.md --no-tree -o tmp/vec-storage-state-sort-tests6.json -j 1`: total=22, passed=22
- `node nodesrc/tests.js -i stdlib/tests/queue.n.md -i stdlib/tests/stack.n.md -i stdlib/tests/deque.n.md -i stdlib/tests/ringbuffer.n.md --no-tree -o tmp/vec-storage-state-dependent-collections2.json -j 1`: total=15, passed=15
- `node nodesrc/tests.js -i tests/stdlib/binary_heap_collections.n.md -i stdlib/tests/binary_heap.n.md --no-tree -o tmp/vec-storage-state-binary-heap2.json -j 1`: total=8, passed=8

残件:

- List node storage、Bloom / CountingBloom などの raw byte storage owner は引き続き typed owner state / OwnedBuffer 設計へ移す余地がある。

## 2026-05-06 AdjacencyMatrix typed Vec storage 部分進捗

`stdlib/alloc/collections/adjacency_matrix.nepl` は matrix byte payload を raw `MemPtr<u8>` array から `Vec<u8>` owner へ移行した。

進捗:

- `AdjacencyMatrix` 本体は `nverts/nbytes/bits` を持ち、`bits` は `Vec<u8>` として保持する。
- `new` は `alloc_ptr<u8>` と raw zero-fill loop を使わず、`vec::filled<u8>` で初期化済み byte storage を確保する。
- matrix byte の読み書きは `vec::get` / `vec::replace` に集約し、`AdjacencyMatrix` から `MemPtr` / `mem_ptr_wrap` / `mem_ptr_addr` / `alloc_ptr` / `load_u8` / `store_u8` / `dealloc_raw` を排除した。
- `clear` は typed storage mutation として `*` effect を明示した。
- `free` は `vec::free<u8>` で matrix storage owner を閉じる。
- `nodesrc/test_stdlib_adjacency_matrix_no_unsafe_unwraps.js` は typed `Vec<u8>` storage、Vec helper 経由の byte read/write、raw MemPtr / raw byte load-store 禁止を固定するよう更新した。

検証:

- `node nodesrc/tests.js -i stdlib/alloc/collections/adjacency_matrix.nepl --no-tree -o tmp/adjacency-matrix-vec-storage-doctest.json -j 1`: total=6, passed=6
- `node nodesrc/tests.js -i stdlib/tests/adjacency_matrix.n.md -i tests/stdlib/adjacency_matrix_collections.n.md --no-tree -o tmp/adjacency-matrix-vec-storage-focused.json -j 1`: total=7, passed=7
- `node nodesrc/test_stdlib_adjacency_matrix_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_adjacency_matrix_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_adjacency_matrix_update_error_owner.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed

残件:

- List node storage、Bloom / CountingBloom などの raw byte storage owner は引き続き typed owner state / OwnedBuffer 設計へ移す余地がある。

## 2026-05-06 DisjointSet typed Vec storage 部分進捗

`stdlib/alloc/collections/disjoint_set.nepl` は parent / sizes payload を raw `MemPtr<i32>` array から `Vec<i32>` owner へ移行した。

進捗:

- `DisjointSet` 本体は `n/parent/sizes` を持ち、`parent` / `sizes` は `Vec<i32>` として保持する。
- `new 0` は `mem_ptr_wrap 0` の null raw pointer sentinel ではなく、empty `Vec<i32>` storage を持つ空 union-find として表現する。
- `new` は `vec::filled<i32>` で parent / sizes を初期化し、parent の `parent[i] = i` 初期化だけを `vec::replace` 経由で行う。
- parent / sizes cell の読み書きは `vec::get` / `vec::replace` に集約し、`DisjointSet` から `MemPtr` / `mem_ptr_wrap` / `mem_ptr_addr` / `alloc_ptr` / `load_i32` / `store_i32` / `dealloc_raw` を排除した。
- `union` は `parent` / `sizes` の field borrow と mutation を inner block に閉じてから owner を返す形にし、shared borrow と owner move の重なりを避けた。
- `free` は `vec::free<i32>` で 2 本の storage owner を閉じる。
- `nodesrc/test_stdlib_disjoint_set_no_unsafe_unwraps.js` は typed `Vec<i32>` storage、empty Vec state、Vec helper 経由の cell read/write、raw MemPtr / raw i32 load-store 禁止を固定するよう更新した。

検証:

- `node nodesrc/tests.js -i stdlib/alloc/collections/disjoint_set.nepl --no-tree -o tmp/disjoint-set-vec-storage-doctest2.json -j 1`: total=6, passed=6
- `node nodesrc/tests.js -i stdlib/tests/disjoint_set.n.md -i tests/stdlib/disjoint_set_collections.n.md --no-tree -o tmp/disjoint-set-vec-storage-focused2.json -j 1`: total=7, passed=7
- `node nodesrc/test_stdlib_disjoint_set_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_disjoint_set_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_disjoint_set_union_error_owner.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed

残件:

- List node storage、bloom / adjacency matrix などの raw byte storage owner は引き続き typed owner state / OwnedBuffer 設計へ移す余地がある。

## 2026-05-06 SegmentTree typed Vec storage 部分進捗

`stdlib/alloc/collections/segment_tree.nepl` は tree payload を raw `MemPtr<i32>` array から `Vec<i32>` owner へ移行した。

進捗:

- `SegmentTree` 本体は `n/base/data` を持ち、`data` は `Vec<i32>` として保持する。
- `new` は既存どおり負の length を `StdErrorKind::CapacityExceeded` の `Diag` として拒否し、`vec::filled<i32>` で `2 * base` 個の初期化済み tree cell を確保する。
- tree cell の読み書きは `vec::get` / `vec::replace` に集約し、`SegmentTree` から `MemPtr` / `mem_ptr_addr` / `alloc_ptr` / `load_i32` / `store_i32` / `dealloc_raw` を排除した。
- `free` は `vec::free<i32>` で tree storage owner を閉じる。
- `nodesrc/test_stdlib_segment_tree_no_unsafe_unwraps.js` は typed `Vec<i32>` storage、負 length guard、Vec helper 経由の cell read/write、raw MemPtr / raw i32 load-store 禁止を固定するよう更新した。

検証:

- `node nodesrc/tests.js -i stdlib/alloc/collections/segment_tree.nepl --no-tree -o tmp/segment-tree-vec-storage-doctest.json -j 1`: total=5, passed=5
- `node nodesrc/tests.js -i stdlib/tests/segment_tree.n.md -i tests/stdlib/segment_tree_collections.n.md --no-tree -o tmp/segment-tree-vec-storage-focused.json -j 1`: total=6, passed=6
- `node nodesrc/test_stdlib_segment_tree_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_segment_tree_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_segment_tree_update_error_owner.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed

残件:

- List node storage、bloom / adjacency matrix / DisjointSet などの raw byte/numeric storage owner は引き続き typed owner state / OwnedBuffer 設計へ移す余地がある。

## 2026-05-06 Fenwick typed Vec storage 部分進捗

`stdlib/alloc/collections/fenwick.nepl` は 1-indexed Fenwick tree payload を raw `MemPtr<i32>` array から `Vec<i32>` owner へ移行した。

進捗:

- `Fenwick` 本体は `n/bit` を持ち、`bit` は `Vec<i32>` として保持する。
- `new` は負の length を allocator に渡さず `StdErrorKind::CapacityExceeded` の `Diag` として拒否する。
- `new` は `alloc_ptr<i32>` と raw zero-fill loop を使わず、`vec::filled<i32>` で `n + 1` 個の初期化済み tree cell を確保する。
- tree cell の読み書きは `vec::get` / `vec::replace` に集約し、`Fenwick` から `MemPtr` / `mem_ptr_addr` / `alloc_ptr` / `load_i32` / `store_i32` / `dealloc_raw` を排除した。
- `free` は `vec::free<i32>` で tree storage owner を閉じる。
- `nodesrc/test_stdlib_fenwick_no_unsafe_unwraps.js` は typed `Vec<i32>` storage、負 length guard、Vec helper 経由の cell read/write、raw MemPtr / raw i32 load-store 禁止を固定するよう更新した。

検証:

- `node nodesrc/tests.js -i stdlib/alloc/collections/fenwick.nepl --no-tree -o tmp/fenwick-vec-storage-doctest.json -j 1`: total=5, passed=5
- `node nodesrc/tests.js -i stdlib/tests/fenwick.n.md -i tests/stdlib/fenwick_collections.n.md --no-tree -o tmp/fenwick-vec-storage-focused.json -j 1`: total=6, passed=6
- `node nodesrc/test_stdlib_fenwick_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_fenwick_borrowed_queries.js`: passed
- `node nodesrc/test_stdlib_fenwick_add_error_owner.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed

残件:

- List node storage、bloom / adjacency matrix / SegmentTree / DisjointSet などの raw byte/numeric storage owner は引き続き typed owner state / OwnedBuffer 設計へ移す余地がある。

## 2026-05-06 SparseSet typed Vec storage 部分進捗

`stdlib/alloc/collections/sparse_set.nepl` は dense/sparse payload を raw `MemPtr<i32>` array から `Vec<i32>` owner へ移行した。

進捗:

- `SparseSet` 本体は `n/len0/dense/sparse` を持ち、`dense` / `sparse` は `Vec<i32>` として保持する。
- `new 0` は `mem_ptr_wrap 0` を `SparseSet` の owner state として直接構築せず、`vec::filled<i32> 0 0` で empty `Vec` storage を持つ。
- dense/sparse の読み書きは `vec::get` / `vec::replace` に集約し、`SparseSet` から `MemPtr` / `mem_ptr_addr` / `alloc_ptr` / `load_i32` / `store_i32` / `dealloc_raw` を排除した。
- `insert` / `remove` の範囲外 Err path と `free` は `sparse_set_free_arrays` 経由で 2 本の `Vec<i32>` owner を必ず閉じる。
- `nodesrc/test_stdlib_sparse_set_no_unsafe_unwraps.js` は、typed `Vec<i32>` storage、Vec helper 経由の slot 更新、raw MemPtr / null sentinel / raw load-store 禁止を固定するよう更新した。

検証:

- `node nodesrc/tests.js -i stdlib/alloc/collections/sparse_set.nepl --no-tree -o tmp/sparse-set-vec-storage-doctest.json -j 1`: total=7, passed=7
- `node nodesrc/tests.js -i stdlib/tests/sparse_set.n.md -i tests/stdlib/sparse_set_collections.n.md --no-tree -o tmp/sparse-set-vec-storage-tests.json -j 1`: total=5, passed=5
- `node nodesrc/test_stdlib_sparse_set_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_sparse_set_borrowed_observers.js`: passed

残件:

- List node storage、bloom / adjacency matrix / Fenwick / SegmentTree / DisjointSet などの raw byte/numeric storage owner は引き続き typed owner state / OwnedBuffer 設計へ移す余地がある。

## 2026-05-06 BitSet typed Vec storage 部分進捗

`stdlib/alloc/collections/bitset.nepl` は bit payload を raw `MemPtr<u8>` byte array から `Vec<u8>` owner へ移行した。

進捗:

- `BitSet` 本体は `nbits/nbytes/bits` を持ち、`bits` は `Vec<u8>` として保持する。
- `new` は `alloc_ptr<u8>` と raw zero-fill loop を使わず、`vec::filled<u8>` で初期化済み byte storage を確保する。
- bit byte の読み書きは `vec::get` / `vec::replace` に集約し、`BitSet` から `MemPtr` / `mem_ptr_addr` / `alloc_ptr` / `load_u8` / `store_u8` / `dealloc_raw` を排除した。
- `clear` / `fill` は typed storage を更新する API として `*` effect を明示した。
- `free` は `vec::free<u8>` で bit storage owner を閉じる。
- `nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js` は typed `Vec<u8>` storage、Vec helper 経由の byte read/write、raw MemPtr / raw byte load-store 禁止を固定するよう更新した。
- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` の BitSet / SparseSet 進捗を実装後の状態へ更新した。

検証:

- `node nodesrc/tests.js -i stdlib/alloc/collections/bitset.nepl --no-tree -o tmp/bitset-vec-storage-doctest2.json -j 1`: total=7, passed=7
- `node nodesrc/tests.js -i tests/stdlib/bitset_collections.n.md --no-tree -o tmp/bitset-vec-storage-tests.json -j 1`: total=3, passed=3
- `node nodesrc/test_stdlib_bitset_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_bitset_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_bitset_update_error_owner.js`: passed

残件:

- List node storage、bloom / adjacency matrix / Fenwick / SegmentTree / DisjointSet などの raw byte/numeric storage owner は引き続き typed owner state / OwnedBuffer 設計へ移す余地がある。
