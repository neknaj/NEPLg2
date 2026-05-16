---
id: ISS-20260516T005642964Z-VEC-STORES-BACKING-STORAGE-DIRECTLY--2407B1D0
title: "Vec stores backing storage directly instead of OwnedBuffer"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-16
updated: 2026-05-16
target: "stdlib/alloc/collections/vec/**, doc/neplg2/stdlib_collection_mem_string_static_safety_design.md"
---

# ISS-20260516T005642964Z-VEC-STORES-BACKING-STORAGE-DIRECTLY--2407B1D0: Vec stores backing storage directly instead of OwnedBuffer

## 概要

Vec<T> still owns len/cap/storage fields directly. Even after VecStorage<T>::Empty|Owned(RegionToken<T>) tied the tag and owner token together, the collection facade remains the storage owner rather than delegating backing storage and initialized prefix metadata to a separate OwnedBuffer<T> abstraction.

## 対象

- `stdlib/alloc/collections/vec/**, doc/neplg2/stdlib_collection_mem_string_static_safety_design.md`

## 根拠

- `stdlib/alloc/collections/vec/types.nepl` では `VecStorage<T>::Empty | Owned(RegionToken<T>)` と `Vec<T> { len, cap, storage }` が定義されており、tag と free obligation owner の相関は enum で表せている。
- `stdlib/alloc/collections/vec/storage/cleanup.nepl` の `vec_free_storage<T: Copy>` は `VecStorage<T>` を消費して `Empty` / `Owned` を match するため、空 storage と allocated storage の cleanup 分岐は source type で追える。
- しかし `Vec<T>` facade 自体が `len` / `cap` / `storage` を直接保持しているため、collection API と backing storage owner / initialized prefix / moved slot state の責務境界がまだ同じ型に残っている。
- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` は、現行 `Vec<T>` を「改善済みの過渡」とし、`OwnedBuffer<T>` + initialized prefix へ移行する方針を明記している。
- `Vec` の allocation constructor、raw data observer、raw element helper、`push` / `pop` / `sort` / `clear` / `free` / `vec_free_storage` は `.T: Copy` に限定されており、これは final design ではなく non-Copy payload を安全に扱うための `OwnedBuffer<T>` / drop traversal が未実装であることを示している。

## 問題

Vec<T> still owns len/cap/storage fields directly. Even after VecStorage<T>::Empty|Owned(RegionToken<T>) tied the tag and owner token together, the collection facade remains the storage owner rather than delegating backing storage and initialized prefix metadata to a separate OwnedBuffer<T> abstraction.

## 影響

Stage 6 cannot complete non-Copy payload collection support while Vec itself is the raw-backed storage carrier. The compiler and stdlib must continue reasoning about Vec fields directly, which blocks a clean separation between collection facade, free obligation owner, initialized prefix state, and later drop traversal.

## 修正方針

Introduce OwnedBuffer<T> as the backing storage owner for Vec, move len/cap/storage into it, and make Vec<T> a facade over that buffer. Keep current Copy-only public Vec operations until initialized prefix/drop traversal is implemented, but make future non-Copy support attach to OwnedBuffer rather than the public Vec facade.

この issue は `VecStorage<T>::Owned(RegionToken<T>)` を否定するものではなく、次の段階で storage owner と collection facade を分けるための追跡 issue とする。`OwnedBuffer<T>` は少なくとも free obligation owner、capacity、initialized prefix を束ね、将来の move-out / replace / drop traversal が `Vec<T>` の public field ではなく buffer state に接続できる形にする。

`OwnedBuffer` 化に入る前に、compiler 側では Resource primitive classification を個別 string 判定から typed registry へ集約する必要がある。`OwnedBuffer` を追加するたび Resource IR の複数箇所へ名前判定を増やす設計は避ける。

## 解決内容

2026-05-16 に Stage 6 / Stage D の一部として修正した。

- `OwnedBuffer<T>` を追加し、`len` / `cap` / `storage` を backing storage owner 側へ移した。
- `Vec<T>` は `buffer: OwnedBuffer<T>` だけを持つ public facade になった。
- `VecStorage<T>::Empty | Owned(RegionToken<T>)` は維持し、storage tag と free obligation owner の相関は enum / match で表す。
- storage allocation、borrowed observer、query、mutation、transform、sort wrapper はすべて `OwnedBuffer<T>` 経由で storage owner を参照または消費する。
- source policy は旧 `Vec<T> len cap storage` constructor と `Vec<T>` 直下の `len/cap/storage` field を退行として拒否する。

この修正は `OwnedBuffer<T>` と collection facade の責務分離を完了させるものだが、non-Copy payload collection の完成ではない。initialized prefix、moved slot、drop traversal、compiler-issued owner token は `RV-STDLIB-004` と raw-memory-backed stdlib API 親 issue の残件として扱う。

## 検証

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`
- `node nodesrc/test_stdlib_vec_sort_module_split.js`
- `node nodesrc/test_stdlib_memptr_owner_field_policy.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec -o tmp/vec-owned-buffer-stdlib-tests.json --dist web/dist --no-tree -j 1`
- `node nodesrc/tests.js -i stdlib/tests/vec.n.md -i tests/stdlib/vec_collections.n.md -o tmp/vec-owned-buffer-nmd-tests.json --dist web/dist --no-tree -j 1`
