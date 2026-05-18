---
id: ISS-20260518T134512597Z-DEQUE-PUSH-GROW-FAILURE-DESTROYS-THE-081F1BD4
title: "Deque push grow failure destroys the consumed owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/collections/deque/**"
---

# ISS-20260518T134512597Z-DEQUE-PUSH-GROW-FAILURE-DESTROYS-THE-081F1BD4: Deque push grow failure destroys the consumed owner

## 概要

Deque.push_front and push_back consume Deque<T>, but on grow allocation failure they free the old Vec<Option<T>> storage and return Diag only. This hides the consumed owner instead of returning it to the caller.

## 対象

- `stdlib/alloc/collections/deque/**`

## 根拠

- `stdlib/alloc/collections/deque/api.nepl` の旧 `push_front` / `push_back` は `Deque<T>` owner を消費した後、grow allocation failure branch で `vec::free<Option<T>> items` を実行して `Diag` だけを返していた。
- これは `pop_front` / `pop_back` が `DequePop<T>` で更新後 owner を返す Stage 6 方針、および `BinaryHeap.push` の `BinaryHeapPushError<T>` 化と不整合だった。
- `T: Copy` 制約により追加 item の破棄は許容できるが、consumed collection owner は caller が cleanup / retry を選べるよう API 型に戻す必要がある。

## 問題

Deque.push_front and push_back consume Deque<T>, but on grow allocation failure they free the old Vec<Option<T>> storage and return Diag only. This hides the consumed owner instead of returning it to the caller.

## 影響

Stage 6 owner-preserving fallible collection updates remain inconsistent, and Resource IR cannot prove failure-path owner transfer from the API type.

## 修正方針

Introduce DequePushError<T> carrying Deque<T> and Diag, change push_front/push_back to return Result<Deque<T>, DequePushError<T>>, and update policies/doctests to reject the old free-and-Diag failure path.

## 検証

Run queue/deque source policy and focused Deque doctests, then run issue check.

## 修正内容

- `DequePushError<T>` を追加し、`deque: Deque<T>` と `diag: Diag` を error payload として保持するようにした。
- `push_front<T>` / `push_back<T>` を `Result<Deque<T>, DequePushError<T>>` に変更し、grow allocation failure branch では旧 `Vec<Option<T>>` storage を内部で `free` せず、元の `Deque<T>` owner を error payload に戻す。
- `deque_push_error_diag<T>` / `deque_push_error_deque<T>` を追加し、caller が diagnostic を読んだ後に deque owner を取り出して cleanup / retry を選べるようにした。
- Deque doctest と source policy を更新し、旧 `vec::free<Option<T>> items` + `err<Deque<T>, Diag>` failure path への退行を拒否する。

## 検証結果

- `node nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/deque.nepl -i stdlib/alloc/collections/deque/types.nepl -i stdlib/alloc/collections/deque/api.nepl -i stdlib/tests/deque.n.md -i tests/stdlib/deque_collections.n.md --no-tree -o tmp/agent1-deque-push-owner-second.json -j 1 --dist web/dist --assert-io`: total=9, passed=9
