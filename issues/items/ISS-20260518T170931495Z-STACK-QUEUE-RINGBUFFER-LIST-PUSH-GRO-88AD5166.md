---
id: ISS-20260518T170931495Z-STACK-QUEUE-RINGBUFFER-LIST-PUSH-GRO-88AD5166
title: "Stack Queue RingBuffer List push grow failure destroys owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/collections/{stack,queue,ringbuffer,list}/**"
---

# ISS-20260518T170931495Z-STACK-QUEUE-RINGBUFFER-LIST-PUSH-GRO-88AD5166: Stack Queue RingBuffer List push grow failure destroys owner

## 概要

Stack.push, Queue.push, RingBuffer.push, and List.cons/push consume the collection owner but return Result<Collection<T>, Diag>. On grow/allocation failure they free or discard the returned storage owner internally and expose only Diag, so caller cannot recover cleanup/retry ownership and Resource IR cannot prove owner transfer from the API type.

## 対象

- `stdlib/alloc/collections/{stack,queue,ringbuffer,list}/**`

## 根拠

- `Stack.push` / `Queue.push` / `RingBuffer.push` の grow failure branch が旧 `Vec<Option<T>>` owner を破棄して `Diag` だけを返していた。
- `List.cons` / `List.push` が `Vec.push` failure payload から戻された `Vec<T>` owner を破棄して `Diag` だけを返していた。
- 検証中に、`QueuePop` / `RingBufferPop` の外部利用が owner-backed aggregate field projection に依存し、現行 compiler の owner field restriction と衝突することも確認した。

## 問題

Stack.push, Queue.push, RingBuffer.push, and List.cons/push consume the collection owner but return Result<Collection<T>, Diag>. On grow/allocation failure they free or discard the returned storage owner internally and expose only Diag, so caller cannot recover cleanup/retry ownership and Resource IR cannot prove owner transfer from the API type.

## 影響

Collection owner loss on fallible update keeps RV-STDLIB-004 open and can hide leak/double-drop responsibility behind implementation discipline instead of typed owner-preserving errors.

## 修正方針

Introduce owner-preserving push error payloads for Stack, Queue, RingBuffer, and List. Return Result<Collection<T>, PushError<T>>, keep Diag as borrowed Copy metadata, and expose Copy-bounded owner accessors so failure paths return the consumed owner to the caller.

## 実装結果

- `StackPushError<T>` / `QueuePushError<T>` / `RingBufferPushError<T>` / `ListPushError<T>` を追加し、push/cons の失敗 payload が消費済み collection owner と `Diag` を保持するようにした。
- grow failure branch は旧 storage を内部で `free` せず、owner を error payload に戻す。
- `QueuePop` / `RingBufferPop` に public accessor を追加し、外部 doctest と `pop` helper を direct field projection から切り離した。
- examples の stack push failure path は `StackPushError` から owner を回収して cleanup してから user-facing error に変換する。

## 検証

- `node nodesrc/test_stdlib_stack_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_queue_deque_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_ringbuffer_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_list_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/tests.js -i stdlib/tests/stack.n.md --no-tree -o tmp/agent1-stack-stdlib-push-owner.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/stack_collections.n.md --no-tree -o tmp/agent1-stack-collections-push-owner.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/tests/queue.n.md --no-tree -o tmp/agent1-queue-push-owner.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/queue_collections.n.md --no-tree -o tmp/agent1-queue-collections-push-owner.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/tests/ringbuffer.n.md --no-tree -o tmp/agent1-ringbuffer-push-owner.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/ringbuffer_collections.n.md --no-tree -o tmp/agent1-ringbuffer-collections-push-owner.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/tests/list.n.md --no-tree -o tmp/agent1-list-push-owner.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/list_collections.n.md --no-tree -o tmp/agent1-list-collections-push-owner.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/agent1-pipe-collections-push-owner.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i examples/rpn.nepl -i examples/rpn_legacy.nepl -i examples/bf.nepl --no-tree -o tmp/agent1-stack-examples-push-owner.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/test_stdlib_documentation_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/stack/types.nepl --no-tree -o tmp/agent1-stack-types-push-owner-docs.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/collections/queue/types.nepl --no-tree -o tmp/agent1-queue-types-push-owner-docs.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/collections/ringbuffer/types.nepl --no-tree -o tmp/agent1-ringbuffer-types-push-owner-docs.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/collections/list/types.nepl --no-tree -o tmp/agent1-list-types-push-owner-docs.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
