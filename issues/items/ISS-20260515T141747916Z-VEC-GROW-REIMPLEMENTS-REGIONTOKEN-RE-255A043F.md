---
id: ISS-20260515T141747916Z-VEC-GROW-REIMPLEMENTS-REGIONTOKEN-RE-255A043F
title: "Vec grow reimplements RegionToken realloc and unchecked capacity doubling"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/core/mem/pointer/region.nepl, stdlib/alloc/collections/vec/mutation/push.nepl, stdlib/alloc/io/bytebuilder/storage.nepl"
---

# ISS-20260515T141747916Z-VEC-GROW-REIMPLEMENTS-REGIONTOKEN-RE-255A043F: Vec grow reimplements RegionToken realloc and unchecked capacity doubling

## 概要

Vec push computes grown capacity with unchecked cap*2 and Vec/ByteBuilder reimplement RegionToken realloc by splitting token internals into ptr/size. This keeps owner-token reallocation discipline outside core/mem and can misclassify capacity overflow as allocation failure while relying on duplicated owner boundary code.

## 対象

- `stdlib/core/mem/pointer/region.nepl, stdlib/alloc/collections/vec/mutation/push.nepl, stdlib/alloc/io/bytebuilder/storage.nepl`

## 根拠

- `Vec.push` の grow path は `let grown_cap <i32> if eq v_cap 0 1 mul v_cap 2` で直接 capacity を倍加していた。allocator に渡す前に element size と `max_alloc_payload_bytes` から上限を証明していないため、巨大 capacity では overflow 後の値が allocation failure と混同される。
- `vec_realloc_region_or_keep` と `byte_builder_realloc_region_or_free` は `RegionToken` から `ptr` / `size` を取り出して `realloc_ptr` を直接呼び、core/mem が持つべき free obligation owner の再確保境界を各 stdlib module が再実装していた。
- Stage 6 の memory model では `MemPtr = non-owning pointer`、`RegionToken` / storage wrapper = free obligation owner と分ける方針であるため、realloc も `RegionToken` owner を受け取り、成功/失敗のどちらでも owner の行方を型で返す API に集約する必要がある。

## 問題

Vec push computes grown capacity with unchecked cap*2 and Vec/ByteBuilder reimplement RegionToken realloc by splitting token internals into ptr/size. This keeps owner-token reallocation discipline outside core/mem and can misclassify capacity overflow as allocation failure while relying on duplicated owner boundary code.

## 影響

A huge or inconsistent Vec capacity can overflow the grow calculation before allocation checks, and future RegionToken owner proof changes must be duplicated across Vec and ByteBuilder. This weakens Stage 6 memory-safety design where RegionToken/OwnedRegion owns free obligations and MemPtr remains only a non-owning projection.

## 修正方針

Add a core/mem owner-preserving RegionToken realloc helper that returns the old token in an error payload, route Vec and ByteBuilder grow through it, and add Vec capacity growth logic that proves the next capacity is positive and within allocator payload bounds before reallocating.

## 対応内容

- `stdlib/core/mem/pointer/region.nepl` に `RegionReallocError<T>` と `realloc_region_bytes_keep<T>(RegionToken<T>, i32) -> Result<RegionToken<T>, RegionReallocError<T>>` を追加した。
- `realloc_region_bytes_keep` は `alloc_payload_fits` で新 byte 数を検査し、`RegionToken` 参照から non-owning `MemPtr` view と旧 extent を借りて `realloc_ptr` を呼ぶ。成功時は新 `RegionToken`、失敗時は旧 `RegionToken` を error payload に戻す。
- `Vec.push` の capacity grow を `vec_next_capacity<T>` へ分離し、`.T` の `size_of` と `max_alloc_payload_bytes` から最大 element count を計算して、unchecked `cap * 2` を push hot path から削除した。
- `Vec` の grow helper は `realloc_region_bytes_keep<T>` を使い、capacity proof failure は `InvalidOperation` / `CapacityExceeded`、raw realloc failure は `OutOfMemory` として `VecReallocRegionError<T>` に旧 owner を戻す。
- `ByteBuilder` の grow helper も core/mem の `realloc_region_bytes_keep<u8>` へ委譲し、`RegionToken` の `ptr` / `size` 直接分解と `realloc_ptr<u8>` 直接呼び出しをやめた。
- Resource IR owner checker では、非所有 raw address view を含む `MemPtr` / `RegionToken` 集約値の `let` / `read` / `assign` で raw-address alias を保持するようにした。これにより `region_ptr(&region)` 由来の non-owning `MemPtr` を `old_ptr` 経由で `realloc_ptr` に渡しても、成功 variant の owner return が元の `RegionToken` owner source へ解決される。
- `nodesrc` の回帰テストを更新し、core/mem が owner-preserving realloc を所有すること、Vec/ByteBuilder が RegionToken realloc を再実装しないこと、Vec が unchecked `cap * 2` に戻らないことを監視する。
- `nepl-core/tests/resource_ir.rs` に `realloc_region_bytes_keep` の Ok / Err 両 variant が caller へ free obligation owner を返す regression を追加した。

## 検証

- `cargo fmt -p nepl-core --check`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_preserves_region_realloc_result_owner`
- `cargo test -p nepl-core --test resource_ir realloc -- --nocapture`: 11 passed
- `node --check nodesrc/test_stdlib_core_mem_boundary.js`
- `node --check nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node --check nodesrc/source_policy/stdlib_builder_owner.js`
- `node nodesrc/test_stdlib_core_mem_boundary.js`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_builder_owner_boundary.js`
- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`
- `trunk build`
- `node nodesrc/tests.js -i stdlib/core/mem/pointer/region.nepl -i stdlib/alloc/io/bytebuilder/storage.nepl -i stdlib/alloc/collections/vec/mutation/push.nepl --no-tree -o tmp/agent1-region-realloc-vec-grow.json -j 1 --dist web/dist --assert-io`: 15 passed
- `node nodesrc/test_stdlib_documentation_contract.js`: `declarationNoDoctest=1032`
- `node nodesrc/issues.js check`
