---
id: ISS-20260518T022816487Z-BINARYHEAPPOP-EXPOSES-OWNER-FIELDS-W-9B47EE6B
title: "BinaryHeapPop exposes owner fields without public accessors"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/collections/binary_heap/**, stdlib/tests/binary_heap.n.md, nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js"
---

# ISS-20260518T022816487Z-BINARYHEAPPOP-EXPOSES-OWNER-FIELDS-W-9B47EE6B: BinaryHeapPop exposes owner fields without public accessors

## 概要

BinaryHeapPop carries the updated BinaryHeap owner and popped item, but binary heap exposes no public accessors. Tests and callers must project the owner field directly, so owner-preserving pop discipline is enforced by convention instead of API shape.

## 対象

- `stdlib/alloc/collections/binary_heap/**, stdlib/tests/binary_heap.n.md, nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js`

## 根拠

- `stdlib/alloc/collections/binary_heap/api/pop.nepl` の `pop` が `BinaryHeapPop` から `item` / `heap` field を直接 projection していた。
- `stdlib/tests/binary_heap.n.md` の `pop_max` doctest も `field::get_ref` / `field::get` で `BinaryHeapPop` の内部 layout に直接依存していた。
- `nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js` は `BinaryHeapPop` accessor の存在や direct projection 禁止を監視していなかった。

## 問題

BinaryHeapPop carries the updated BinaryHeap owner and popped item, but binary heap exposes no public accessors. Tests and callers must project the owner field directly, so owner-preserving pop discipline is enforced by convention instead of API shape.

## 影響

Direct field projection leaks collection owner layout into users and weakens Resource IR/static-check review: future non-Copy or owner-backed heap payload work can bypass accessor-level Copy and ownership constraints.

## 修正方針

Add borrowed item and consuming heap accessors with Copy bounds, route pop/tests through them, and extend the source policy to reject direct BinaryHeapPop field projection.

## 検証

Run the binary heap source policy, focused binary_heap doctests, and issue checks.

## 対応結果

- `binary_heap_pop_item<T>(&BinaryHeapPop<T>) -> Option<T>` を追加し、`BinaryHeapPop` を借用した item read を public API に閉じた。
- `binary_heap_pop_heap<T>(BinaryHeapPop<T>) -> BinaryHeap<T>` を追加し、更新後 heap owner の取り出しを consuming accessor に閉じた。
- 両 accessor は現行の Copy-only collection 契約に合わせて `.T: Copy` とした。non-Copy payload の move-out / drop traversal は `OwnedBuffer<T>` と Resource IR の initialized/moved state 完成後に扱う。
- `pop` と binary heap doctest から direct field projection を削除し、accessor 経由に統一した。
- source policy に accessor signature、`pop` の accessor 経由 cleanup、doctest の direct projection 禁止を追加した。
- `collection_cleanup_contract.n.md` に `binary_heap_pop_heap<CleanupPayload>` が `type.trait_bound.unsatisfied` で拒否される compile-fail regression を追加した。

## 検証結果

- `node nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js`
- `node nodesrc/tests.js -i stdlib\tests\binary_heap.n.md -i tests\stdlib\collection_cleanup_contract.n.md --no-tree -o tmp\agent1-binary-heap-pop-accessors.json -j 1 --dist web\dist --assert-io`: total=32, passed=32
