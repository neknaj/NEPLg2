---
id: ISS-20260518T062457553Z-VEC-DATA-MEM-PTR-BOUNDARY-LACKS-DIRE-DF309C77
title: "Vec data_mem_ptr boundary lacks direct checked-store regression"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/memory_safety.n.md, issues/items/ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md"
---

# ISS-20260518T062457553Z-VEC-DATA-MEM-PTR-BOUNDARY-LACKS-DIRE-DF309C77: Vec data_mem_ptr boundary lacks direct checked-store regression

## 概要

The Stage 6 raw-memory-backed API migration relies on Resource IR rejecting checked MemPtr writes through Vec.data_mem_ptr in ordinary source, but the regression suite does not directly cover root Vec facade -> data_mem_ptr -> store_i32.

## 対象

- `tests/stdlib/memory_safety.n.md, issues/items/ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md`

## 根拠

- `alloc/collections/vec` root facade は `data_mem_ptr<T>(&Vec<T>) -> MemPtr<T>` を再公開している。
- `MemPtr<T>` は non-owning pointer view であり、`Vec<T>` の backing storage owner は `OwnedBuffer<T>` / `VecStorage<T>::Owned(RegionToken<T>)` 側にある。
- `store_i32` などの checked memory wrapper は、allocator / RegionToken 由来の証明済み provenance がある場合だけ許可されるべきで、ordinary source が `&Vec<T>` observer から得た backing view を storage mutation authority として使えてはいけない。
- 既存の `memory_safety.n.md` は owner-backed aggregate constructor / field projection の拒否を持つが、root `Vec` facade -> `data_mem_ptr` -> checked store の直接 regression はなかった。

## 問題

The Stage 6 raw-memory-backed API migration relies on Resource IR rejecting checked MemPtr writes through Vec.data_mem_ptr in ordinary source, but the regression suite does not directly cover root Vec facade -> data_mem_ptr -> store_i32.

## 影響

A future summary/provenance change could accidentally allow ordinary user code to mutate Vec backing storage through a non-owning MemPtr, bypassing collection mutation APIs and initialized/drop state discipline.

## 修正方針

Add a compile-fail regression that imports the safe Vec root facade, obtains data_mem_ptr from &Vec<i32>, and proves store_i32 is rejected with resource.raw.memory_outside_boundary. Record progress on the raw-memory-backed API parent issue.

## 検証

Run focused memory_safety doctest and issues check.

## 解決

`tests/stdlib/memory_safety.n.md` に、通常 source が safe `alloc/collections/vec` facade から `data_mem_ptr<i32>(&Vec<i32>)` を取得し、それを `store_i32` に渡すと `resource.raw.memory_outside_boundary` で拒否される compile-fail regression を追加した。

この regression は `data_mem_ptr` の存在自体を final API として承認するものではない。現行 Stage 6 では `MemPtr` は non-owning view であり、`Vec` の storage mutation は `push` / `replace` / sort など collection API または compiler-owned raw boundary 内に閉じる必要がある。将来 `OwnedBuffer<T>` / initialized cell / drop traversal を完成させる場合も、通常 source が backing storage pointer を mutation authority として使える設計へ戻してはいけない。

## 検証結果

- `node nodesrc/run_doctest.js -i tests\stdlib\memory_safety.n.md -n 37 --dist web\dist`: passed
- `node nodesrc/issues.js check --dir issues`: passed
