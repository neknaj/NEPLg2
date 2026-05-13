---
id: ISS-20260513T101054155Z-CORE-MEM-ALLOCATION-BYTE-COUNTS-CAN--9B7BDEA4
title: "core/mem allocation byte counts can overflow before allocator bounds"
area: stdlib
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "stdlib/core/mem/layout.nepl, stdlib/core/mem/allocator.nepl, stdlib/core/mem/pointer/region.nepl, tests/stdlib/memory_safety.n.md, nodesrc/test_stdlib_core_mem_boundary.js"
---

# ISS-20260513T101054155Z-CORE-MEM-ALLOCATION-BYTE-COUNTS-CAN--9B7BDEA4: core/mem allocation byte counts can overflow before allocator bounds

## 概要

core/mem computes allocator payload sizes with wrapping i32 arithmetic. alloc_region multiplies count * size_of<T> before checking the allocator payload limit, and alloc_raw aligns size+header without rejecting sizes that make size+header+7 overflow.

## 対象

- `stdlib/core/mem/layout.nepl, stdlib/core/mem/allocator.nepl, stdlib/core/mem/pointer/region.nepl, tests/stdlib/memory_safety.n.md, nodesrc/test_stdlib_core_mem_boundary.js`

## 根拠

- `stdlib/core/mem/pointer/region.nepl` の `alloc_region` は `mul count size_of<T>` を先に実行し、その結果を `alloc_region_bytes` へ渡していた。
- `stdlib/core/mem/allocator.nepl` の `alloc_raw` は `align8 add size header` を実行する前に `size + header + 7` が i32 範囲へ収まることを検査していなかった。
- `align8` 自体は汎用 layout helper であり、allocator payload と header の上限証明は allocator boundary 側で持つ必要がある。
- dealloc/realloc 側の size mismatch / owner extent proof は別問題として `ISS-20260513T101719832Z-DEALLOC-AND-REALLOC-SIZE-ARGUMENTS-N-D7EADBBD` に分離した。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 親 issue: [ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84](./ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md)

## 問題

core/mem computes allocator payload sizes with wrapping i32 arithmetic. alloc_region multiplies count * size_of<T> before checking the allocator payload limit, and alloc_raw aligns size+header without rejecting sizes that make size+header+7 overflow.

## 影響

A large but positive element count or byte size can wrap into a smaller allocator total, allowing RegionToken metadata to claim more storage than the allocator actually reserved or corrupting the free list on deallocation. Resource IR owner/cell checks then reason over a storage extent whose runtime allocation proof is invalid.

## 修正方針

Introduce a central max allocation payload/layout predicate, reject oversized allocation payloads before alignment arithmetic, and make alloc_region prove count * size_of<T> fits before multiplication.

## 検証

node nodesrc/tests.js -i stdlib/core/mem -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/core-mem-allocation-size-proof.json -j 1 --dist web/dist; node nodesrc/test_stdlib_core_mem_boundary.js; node nodesrc/issues.js check --dir issues

## 修正結果

- `stdlib/core/mem/layout.nepl` に `max_alloc_payload_bytes` と `alloc_payload_fits` を追加し、allocator metadata の `size + header + align padding` が i32 範囲に収まる上限を一箇所で定義した。
- `alloc_raw` は `alloc_payload_fits size` を満たさない payload を、`align8(size + header)` の前に拒否する。
- `alloc_region_bytes` は allocator payload 上限を超える byte 数を `Err` にし、`alloc_region` は `count * size_of<T>` の前に `max_alloc_payload_bytes / size_of<T>` で最大 count を証明する。
- `tests/stdlib/memory_safety.n.md` に `alloc_region<i32> 536870909` と `alloc_region_bytes<u8> 2147483633` が `Err` になる regression を追加した。
- `nodesrc/test_stdlib_core_mem_boundary.js` で allocator payload 上限 helper と `alloc_region` の乗算前証明を source policy 化した。

## 検証結果

- `node nodesrc/test_stdlib_core_mem_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/core/mem/layout.nepl -i stdlib/core/mem/allocator.nepl -i stdlib/core/mem/pointer/region.nepl --no-tree -o tmp/core-mem-allocation-size-proof-stdlib-doc.json -j 1 --dist web/dist`: total=15, passed=15
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/core-mem-allocation-size-proof-memory-safety.json -j 1 --dist web/dist`: total=29, passed=29
