---
id: ISS-20260517T030536827Z-DEQUE-POP-DROPS-UPDATED-OWNER-INSTEA-2B452BAF
title: "Deque pop drops updated owner instead of returning owner-preserving result"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "stdlib/alloc/collections/deque/**; stdlib/tests/deque.n.md; nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js"
---

# ISS-20260517T030536827Z-DEQUE-POP-DROPS-UPDATED-OWNER-INSTEA-2B452BAF: Deque pop drops updated owner instead of returning owner-preserving result

## 概要

Deque pop_front/pop_back consume the Deque owner, peek the item, free the whole container, and return only Option<T>. This keeps owned remove/pop and container drop conflated while Queue and RingBuffer already return owner-preserving pop result structs.

## 対象

- `stdlib/alloc/collections/deque/**; stdlib/tests/deque.n.md; nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`

## 根拠

- `stdlib/alloc/collections/deque/api.nepl` の旧 `pop_front` / `pop_back` は `peek_*` で item を読んだあと `free` で deque owner を閉じ、`Option<T>` だけを返していた。
- `Queue` / `RingBuffer` は `QueuePop<T>` / `RingBufferPop<T>` により、更新後 owner と item を同時に返していた。
- 親 issue `ISS-20260425T000000Z-RV-STDLIB-004-91534828` は、owned remove/pop と container drop を API と型制約で分けることを残件にしている。

## 問題

Deque pop_front/pop_back consume the Deque owner, peek the item, free the whole container, and return only Option<T>. This keeps owned remove/pop and container drop conflated while Queue and RingBuffer already return owner-preserving pop result structs.

## 影響

Callers cannot continue with the updated Deque owner after removing one element, and the collection cleanup contract still has a public pop surface that hides owner transfer behind terminal cleanup.

## 修正方針

Add DequePop<T> with deque/item fields, make pop_front/pop_back clear the consumed slot and return the updated owner plus item, add accessors and policy/doctest coverage.

## 検証

Run focused deque doctest and queue/deque source policy.

## 解決内容

- `DequePop<T>` を追加し、`deque <Deque<T>>` と `item <Option<T>>` を持つ owner-preserving pop result にした。
- `pop_front` / `pop_back` は、空でない場合に対象 slot を `None` へ戻し、更新後 `Deque` owner と item を `DequePop<T>` で返す。
- `deque_pop_item` / `deque_pop_deque` を追加し、caller が pop result から item と更新後 owner を分解できるようにした。
- `deque_next_index` を追加し、front pop の head 更新を circular index helper に閉じた。
- `stdlib/tests/deque.n.md` と `nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js` に回帰テストを追加した。

## 検証結果

- `node nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`: pass
- `node nodesrc/run_doctest.js -i stdlib/tests/deque.n.md -n 2`: pass
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/deque/index.nepl -n 3`: pass
- `node nodesrc/tests.js -i stdlib/tests/deque.n.md --no-tree -o tmp/deque-tests.json -j 1`: pass
