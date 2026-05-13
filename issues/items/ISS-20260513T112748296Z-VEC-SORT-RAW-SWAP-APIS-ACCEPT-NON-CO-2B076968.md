---
id: ISS-20260513T112748296Z-VEC-SORT-RAW-SWAP-APIS-ACCEPT-NON-CO-2B076968
title: "Vec sort raw swap APIs accept non-Copy payloads"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "stdlib/alloc/collections/vec/sort/**"
---

# ISS-20260513T112748296Z-VEC-SORT-RAW-SWAP-APIS-ACCEPT-NON-CO-2B076968: Vec sort raw swap APIs accept non-Copy payloads

## 概要

Vec sort helpers and public sort APIs use unchecked raw load/store and swap, but many signatures require only Ord or no payload bound. For non-Copy payloads, sorting can shallow-move values through raw memory without initialized-cell/drop traversal proof.

## 対象

- `stdlib/alloc/collections/vec/sort/**`

## 根拠

- `sort/common` の unchecked helper は raw `load<T>` / `store<T>` と swap を直接使う。
- `sort_quick` / `sort_heap` / simple sort / merge sort はこれらの helper を通じて要素を一時値へ読み出して別 slot へ書き戻す。
- 変更前の署名は `.T: Ord` または無制約 `.T` が多く、non-Copy payload でも API 上は sort 対象にできる形だった。
- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` の Stage D は、non-Copy payload の collection 操作を `OwnedBuffer<T>` と initialized prefix / drop traversal 上で再設計する方針にしている。

## 問題

Vec sort helpers and public sort APIs use unchecked raw load/store and swap, but many signatures require only Ord or no payload bound. For non-Copy payloads, sorting can shallow-move values through raw memory without initialized-cell/drop traversal proof.

## 影響

Non-Copy Vec sorting can violate owner discipline and hides the need for OwnedBuffer<T> plus move/drop-aware sort implementation.

## 修正方針

Until OwnedBuffer<T> and initialized prefix traversal exist, restrict raw sort helpers to T: Copy and public sort APIs to T: Ord&Copy. Add compile-fail regression and source policy coverage.

## 検証

Run focused vec sort doctests, vec source policy, issue check, and diff check.

## 修正内容

- `sort_get_unchecked` / `sort_set_unchecked` / `sort_swap` / scratch buffer helper を `.T: Copy` に限定した。
- public sort algorithms を `.T: Ord&Copy` に限定した。
- `sort.nepl` の doc comment に、現行 sort family が Copy-only である理由を明記した。
- `NonCopyOrd` の `sort` が `type.trait_bound.unsatisfied` で compile-fail になる doctest を追加した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に、sort family が無制約 `.T` や `.T: Ord` のまま残らない source policy を追加した。

## 検証結果

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort.nepl --no-tree -o tmp/agent1-vec-sort-copy-bound-sort-root.json -j 1 --dist web/dist`: total=3, passed=3
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort --no-tree -o tmp/agent1-vec-sort-copy-bound-sort-dir.json -j 4 --dist web/dist`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec --no-tree -o tmp/agent1-vec-sort-copy-bound-vec.json -j 4 --dist web/dist`: total=34, passed=34

## 親issueとの関係

- `ISS-20260425T000000Z-RV-STDLIB-004-91534828` の Stage D 残件のうち、raw swap based sort が non-Copy payload を受け入れる入口を閉じた。
- non-Copy payload sort はこの issue では実装しない。`OwnedBuffer<T>`、move-out aware swap、drop traversal、borrowed comparison API が揃った段階で別途再設計する。
