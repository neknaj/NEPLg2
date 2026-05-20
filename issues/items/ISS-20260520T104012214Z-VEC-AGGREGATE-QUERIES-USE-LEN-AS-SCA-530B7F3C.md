---
id: ISS-20260520T104012214Z-VEC-AGGREGATE-QUERIES-USE-LEN-AS-SCA-530B7F3C
title: "Vec aggregate queries use len as scan bound before proving OwnedBuffer invariant"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/alloc/collections/vec/query/**, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260520T104012214Z-VEC-AGGREGATE-QUERIES-USE-LEN-AS-SCA-530B7F3C: Vec aggregate queries use len as scan bound before proving OwnedBuffer invariant

## 概要

Vec count/fold/reduce/find/any/all read OwnedBuffer.len and use it as a loop bound before proving the current Copy-only OwnedBuffer invariant. get() now prevents raw load from malformed metadata, but malformed len can still drive an unbounded scan and leaves query semantics dependent on invalid metadata.

## 対象

- `stdlib/alloc/collections/vec/query/**, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- `count` / `fold` / `reduce` / `find` / `any` / `all` は `OwnedBuffer.len` を先に読み、loop bound として使っていた。
- `get<T>` は raw load 前に invariant を確認するようになったが、query 側が malformed `len` に従って loop を回すため、invalid metadata が traversal decision に残っていた。
- `len` が `cap` や `initialized_len` と矛盾している場合、raw memory access は止まっても、compile/runtime の挙動と計算量が不正 metadata に依存する。

## 問題

Vec count/fold/reduce/find/any/all read OwnedBuffer.len and use it as a loop bound before proving the current Copy-only OwnedBuffer invariant. get() now prevents raw load from malformed metadata, but malformed len can still drive an unbounded scan and leaves query semantics dependent on invalid metadata.

## 影響

Stage D collection safety still lets invalid owner aggregate metadata influence control flow before the Vec invariant boundary. This weakens the contract Resource IR and self-host code should rely on: malformed storage must be rejected before both raw memory access and traversal decisions.

## 修正方針

Add a common invariant guard to Vec aggregate/predicate query entry points before reading len as a scan bound. Invalid Vec owners should return neutral non-success results without entering a loop, while valid empty Vec behavior remains unchanged.

## 検証

Add source policy coverage for query entry invariant guards, run focused Vec query doctests, vec source policy, and issues check.

## 2026-05-20 Agent 1 修正

`count` / `fold` / `reduce` / `find` / `any` / `all` の entry で、`OwnedBuffer.len` を loop bound として読む前に `vec_buffer_current_copy_invariant<T>` を確認するようにした。

invalid owner aggregate の戻り値:

- `count`: `0`
- `fold`: 初期 accumulator
- `reduce`: `None`
- `find`: `None`
- `any`: `false`
- `all`: `false`

valid empty `Vec` では従来どおり `all == true`、`fold == acc`、`reduce/find == None` になる。invalid metadata は empty とは扱わず、走査に入らない非成功結果へ落とす。

回帰テスト:

- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に、各 aggregate/predicate query が invariant を `src_len` 読み取りより前に行うことを検査する source policy を追加した。

focused verification:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/query/aggregate.nepl --no-tree --dist web/dist -o tmp/agent1-vec-query-aggregate-invariant.json -j 1 --assert-io`: 3/3 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/query/predicate.nepl --no-tree --dist web/dist -o tmp/agent1-vec-query-predicate-invariant.json -j 1 --assert-io`: 3/3 passed
