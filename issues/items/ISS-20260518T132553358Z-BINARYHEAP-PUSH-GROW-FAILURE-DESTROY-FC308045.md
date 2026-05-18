---
id: ISS-20260518T132553358Z-BINARYHEAP-PUSH-GROW-FAILURE-DESTROY-FC308045
title: "BinaryHeap.push grow failure destroys the consumed heap owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/collections/binary_heap/**"
---

# ISS-20260518T132553358Z-BINARYHEAP-PUSH-GROW-FAILURE-DESTROY-FC308045: BinaryHeap.push grow failure destroys the consumed heap owner

## 概要

BinaryHeap.push consumes BinaryHeap<T>, then on grow allocation failure frees the old Vec<Option<T>> storage and returns Diag only. This is inconsistent with Stage 6 owner-preserving fallible update design and prevents callers from deciding whether to keep, retry, or explicitly cleanup the original heap owner.

## 対象

- `stdlib/alloc/collections/binary_heap/**`

## 根拠

- `push<T>` の旧 signature は `Result<BinaryHeap<T>, Diag>` であり、grow allocation failure で caller に戻るのは diagnostic だけだった。
- 旧 grow failure branch は `heap_alloc_slots<T>` の `Result::Err` で `vec::free<Option<T>> items` を実行してから `err<BinaryHeap<T>, Diag> e` を返していた。
- これは `BTreeMapInsertError` / `BTreeSetInsertError` などで進めている Stage 6 の owner-preserving fallible update と不整合で、失敗時の collection owner flow を API 型で証明できない。

## 問題

BinaryHeap.push consumes BinaryHeap<T>, then on grow allocation failure frees the old Vec<Option<T>> storage and returns Diag only. This is inconsistent with Stage 6 owner-preserving fallible update design and prevents callers from deciding whether to keep, retry, or explicitly cleanup the original heap owner.

## 影響

Collection owner flow remains harder for Resource IR to prove, and future non-Copy/OwnedBuffer work would inherit a failure path that hides the consumed owner instead of making ownership transfer explicit in the result type.

## 修正方針

Introduce a named BinaryHeapPushError<T> payload carrying the consumed BinaryHeap<T> owner and Diag. Change push to return Result<BinaryHeap<T>, BinaryHeapPushError<T>>, return the reconstructed heap owner on grow allocation failure, and update source policy/doctests to reject the old free-and-Diag path.

## 修正内容

- `BinaryHeapPushError<T>` を追加し、`heap: BinaryHeap<T>` と `diag: Diag` を error payload として保持するようにした。
- `binary_heap_push_error_diag<T>(&BinaryHeapPushError<T>) -> Diag` と `binary_heap_push_error_heap<T: Copy>(BinaryHeapPushError<T>) -> BinaryHeap<T>` を追加し、diagnostic borrow と heap owner extraction を分離した。
- `push<T>` を `Result<BinaryHeap<T>, BinaryHeapPushError<T>>` へ変更し、grow allocation failure では旧 `Vec<Option<T>>` storage を破棄せず、`BinaryHeap<T> len0 cap0 items` を `BinaryHeapPushError<T>` に戻す。
- 成功 branch は typed `Result::Ok` constructor を使い、owner-bearing payload が helper 経由で曖昧化しないようにした。
- BinaryHeap の doctest と focused fixture を新しい error type へ更新し、`nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js` で旧 free-and-Diag failure path の再導入を拒否する。

## 検証

- `node nodesrc/test_stdlib_binary_heap_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/binary_heap/types.nepl -i stdlib/alloc/collections/binary_heap/api/push.nepl -i stdlib/alloc/collections/binary_heap/api/observer.nepl -i stdlib/alloc/collections/binary_heap/api/pop.nepl -i stdlib/tests/binary_heap.n.md -i tests/stdlib/binary_heap_collections.n.md --no-tree -o tmp/agent1-binary-heap-push-owner-3.json -j 1 --dist web/dist --assert-io`: total=14, passed=14
- `node nodesrc/test_stdlib_documentation_contract.js`

## 関連

- `doc/neplg2/static_check_complexity_reduction_plan.md` の Stage 6。
- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` の collection fallible update 方針。
- 親 issue: `ISS-20260425T000000Z-RV-STDLIB-004-91534828`
