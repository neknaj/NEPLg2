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

残件:

- `stdlib/alloc/collections/hashset.nepl` はまだ 0/1/2 status と raw header/entries layout を持つ。
- Vec / Stack / BinaryHeap などの null/raw owner sentinel 設計は別途段階的に typed owner state へ移す必要がある。

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
