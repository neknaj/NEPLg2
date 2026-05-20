---
id: ISS-20260520T142914786Z-COLLECTION-OBSERVER-DOCTESTS-STILL-U-19B433D2
title: "Collection observer doctests still use ambiguous observer call forms"
area: stdlib
status: open
resolved: false
priority: P2
type: test
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/alloc/collections/**"
---

# ISS-20260520T142914786Z-COLLECTION-OBSERVER-DOCTESTS-STILL-U-19B433D2: Collection observer doctests still use ambiguous observer call forms

## 概要

Focused execution of collection observer doctests still reports compile failures in examples that call observers inside another prefix call, such as eq len &collection n or if eq len &collection 0 0 1. These examples rely on an ambiguous function-as-argument shape instead of binding the observer result as a typed scalar first. The same focused run also exposes hashset rehash/capacity helper failures that need to be separated from the borrowed observer Copy-only boundary.

## 対象

- `stdlib/alloc/collections/**`

## 根拠

- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/access/header.nepl -i stdlib/alloc/collections/vec/invariant.nepl -i stdlib/alloc/collections/vec/transform/filter/partition/view.nepl -i stdlib/alloc/collections/stack/api.nepl -i stdlib/alloc/collections/queue/api.nepl -i stdlib/alloc/collections/deque/api.nepl -i stdlib/alloc/collections/ringbuffer/api.nepl -i stdlib/alloc/collections/binary_heap/api/observer.nepl -i stdlib/alloc/collections/list/query.nepl -i stdlib/alloc/collections/btreemap/api/observer.nepl -i stdlib/alloc/collections/btreeset/api/observer.nepl -i stdlib/alloc/collections/hashmap/api.nepl -i stdlib/alloc/collections/hashset/api.nepl -i stdlib/alloc/collections/bloom_filter/api.nepl -i stdlib/alloc/collections/counting_bloom_filter/api.nepl --no-tree --dist web/dist -o tmp/agent1-borrowed-collection-observer-docs.json -j 4 --assert-io` が `total=52, passed=29, failed=23` で失敗した。
- 代表失敗は `bloom_filter/api.nepl::doctest#1/#2` の `let ok <bool> eq len &bf 64` / `eq len &bf 32` で、compiler は `len` を呼び出しではなく未定義 identifier として扱っている。
- `btreemap/api/observer.nepl::doctest#1/#3` や `btreeset/api/observer.nepl::doctest#1` でも `eq len<i32, i32> &hm 2` / `eq len<i32> &s 2` が同じ形で失敗した。
- `hashset/api.nepl` の focused doctest では `hashset/rehash.nepl:83` の capacity expression が `type.overload.no_match` / `type.stack.extra_values` を出しており、observer doctest 書式とは別に実装修正が必要か切り分ける必要がある。

## 問題

Focused execution of collection observer doctests still reports compile failures in examples that call observers inside another prefix call, such as eq len &collection n or if eq len &collection 0 0 1. These examples rely on an ambiguous function-as-argument shape instead of binding the observer result as a typed scalar first. The same focused run also exposes hashset rehash/capacity helper failures that need to be separated from the borrowed observer Copy-only boundary.

## 影響

Collection documentation cannot be used as a reliable executable contract for observer APIs. Safe Copy-only boundary changes become harder to verify because unrelated legacy doctest failures are mixed into focused observer runs.

## 修正方針

Audit collection observer doctests and rewrite observer examples to bind typed scalar results before comparison. Separately review the hashset rehash capacity helper failure and fix the implementation or test fixture at the root cause instead of hiding it.

## 検証

Run focused collection observer doctests for the affected stdlib/alloc/collections files and keep node nodesrc/test_stdlib_collection_cleanup_contract.js passing.
