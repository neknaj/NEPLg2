---
id: ISS-20260512T034125476Z-PIPE-COLLECTIONS-LIST-DOCTEST-STILL--8AA21971
title: "pipe_collections List doctest still calls borrowed observers by value"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: "tests/stdlib/pipe_collections.n.md, stdlib/alloc/collections/list.nepl, nodesrc/test_stdlib_list_no_unsafe_unwraps.js"
---

# ISS-20260512T034125476Z-PIPE-COLLECTIONS-LIST-DOCTEST-STILL--8AA21971: pipe_collections List doctest still calls borrowed observers by value

## 概要

tests/stdlib/pipe_collections.n.md::pipe_list_alias_chain still calls List borrowed observers as len<i32> xs0 and get<i32> xs1 1. After List observer APIs were corrected to borrow the owner, the fixture fails with type.overload.no_match and hides pipe coverage for List.

## 対象

- `tests/stdlib/pipe_collections.n.md, stdlib/alloc/collections/list.nepl, nodesrc/test_stdlib_list_no_unsafe_unwraps.js`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/pipe-collections-after-queue-split.json -j 1 --dist web/dist` で `doctest#1` が `error[type.overload.no_match]` になった。
- compile error location は `/virtual/entry.nepl:17` の `len<i32> xs0` と `/virtual/entry.nepl:23` の `get<i32> xs1 1`。
- `stdlib/alloc/collections/list/query.nepl` の observer は `len <.T> <(&List<.T>)->i32>`、`get <.T: Copy> <(&List<.T>,i32)->Option<.T>>` で、owner を借用する契約へ移行済み。

## 問題

tests/stdlib/pipe_collections.n.md::pipe_list_alias_chain still calls List borrowed observers as len<i32> xs0 and get<i32> xs1 1. After List observer APIs were corrected to borrow the owner, the fixture fails with type.overload.no_match and hides pipe coverage for List.

## 影響

The pipe_collections suite no longer gives a clean signal for collection pipe coverage. It also keeps an unsafe ownership style in documentation-like tests, encouraging owner moves for read-only List observation under the current memory-safety model.

## 修正方針

Rewrite the List pipe doctest to call len/get through explicit borrows, free observed owners after use, and extend the List source-policy test so pipe_collections cannot reintroduce by-value List observer calls.

## 検証

Run node nodesrc/run_doctest.js -i tests/stdlib/pipe_collections.n.md -n 1 --dist web/dist and node nodesrc/test_stdlib_list_no_unsafe_unwraps.js.
