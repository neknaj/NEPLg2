---
id: ISS-20260520T134621232Z-COLLECTION-OWNER-PRODUCING-APIS-LACK-8845B15D
title: "Collection owner-producing APIs lack cross-family Copy-only policy"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-20
updated: 2026-05-20
target: "nodesrc/test_stdlib_collection_cleanup_contract.js, tests/stdlib/collection_cleanup_contract.n.md, issues/items/ISS-20260425T000000Z-RV-STDLIB-004-91534828.md"
---

# ISS-20260520T134621232Z-COLLECTION-OWNER-PRODUCING-APIS-LACK-8845B15D: Collection owner-producing APIs lack cross-family Copy-only policy

## 概要

RV-STDLIB-004 is mitigated by Copy-only cleanup and owner recovery boundaries, but the cross-collection policy does not yet structurally cover generic public APIs that create, update, expose, or return collection owners and payload views before non-Copy element drop traversal exists.

## 対象

- `nodesrc/test_stdlib_collection_cleanup_contract.js, tests/stdlib/collection_cleanup_contract.n.md, issues/items/ISS-20260425T000000Z-RV-STDLIB-004-91534828.md`

## 根拠

- `nodesrc/test_stdlib_collection_cleanup_contract.js` は generic cleanup、owner-returning error accessor、pop owner accessor、owner-consuming fallible API を横断検査していたが、constructor / update / observer / typed data-view のように collection owner や payload view を作る API surface までは一括検査していなかった。
- `tests/stdlib/collection_cleanup_contract.n.md` は cleanup/free の compile-fail が中心で、`Vec.new` / `Vec.push` / `HashMap.insert` など owner-producing API が non-Copy payload を拒否する代表例が不足していた。

## 問題

RV-STDLIB-004 is mitigated by Copy-only cleanup and owner recovery boundaries, but the cross-collection policy does not yet structurally cover generic public APIs that create, update, expose, or return collection owners and payload views before non-Copy element drop traversal exists.

## 影響

A future collection constructor, push/insert/remove/get, or typed data-view API could reopen non-Copy payload ownership before OwnedBuffer initialized cells, moved slots, drop traversal, and compiler-issued owner tokens are connected.

## 修正方針

Extend the collection cleanup contract policy to detect owner-producing and owner-updating generic collection API surfaces by function type shape and require the collection payload generics to carry Copy until non-Copy drop traversal exists. Add representative compile-fail doctests for Vec and map insert surfaces.

## 検証

Run the collection cleanup source policy, focused collection cleanup doctests, source policy regressions, issue check, and diff whitespace checks.

## 修正内容

- collection source policy に `ownerSurfaceInspected` を追加し、関数型から `Vec` / `HashMap` / `HashSet` / `BTreeMap` / `BTreeSet` / `Stack` / `Queue` / `Deque` / `RingBuffer` / `BinaryHeap` / `List` / Bloom filter 系などの generic owner surface を検出するようにした。
- 検出対象は `new` / `with_capacity` / `vec_empty` / `data_mem_view` / `get` / `push` / `insert` / `remove` / pop/error recovery などで、owner aggregate や payload view を作る・返す・更新する generic parameter に `Copy` bound を要求する。
- `&Vec<Option<T>>` のような borrowed storage view は owner/payload を移動しないため、borrowed return はこの owner-producing policy の対象外にした。
- `collection_cleanup_contract.n.md` に `Vec.new`、`Vec.with_capacity`、`Vec.push`、`Vec.get`、`BTreeMap.insert`、`HashMap.insert` の non-Copy payload compile-fail を追加した。

## 検証結果

- `node nodesrc/test_stdlib_collection_cleanup_contract.js`: passed
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/collection_cleanup_contract.n.md --no-tree --dist web/dist -o tmp/agent1-collection-owner-surface-contract.json -j 4 --assert-io`: 37/37 passed
