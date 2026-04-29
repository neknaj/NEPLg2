---
id: ISS-20260429T204604331Z-LIST-NODE-ALLOCATOR-CALL-SITES-OMIT--57F7856C
title: "List reverse source policy still required allocation-copy reverse"
area: stdlib
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "stdlib/alloc/collections/list.nepl, nodesrc/test_stdlib_list_no_unsafe_unwraps.js, stdlib/tests/list.n.md, tests/stdlib/list_collections.n.md"
---

# ISS-20260429T204604331Z-LIST-NODE-ALLOCATOR-CALL-SITES-OMIT--57F7856C: List reverse source policy still required allocation-copy reverse

## 概要

CI source policy still required `reverse` to allocate replacement nodes through `list_alloc_node`, but ResourceIR now exposes that this design copies payloads and leaves the consumed source list's raw node owners open.

## 対象

- `stdlib/alloc/collections/list.nepl, nodesrc/test_stdlib_list_no_unsafe_unwraps.js, stdlib/tests/list.n.md, tests/stdlib/list_collections.n.md`

## 根拠

- main CI failed in `nodesrc/test_stdlib_list_no_unsafe_unwraps.js` because the source policy expected `reverse` to call `list_alloc_node` with a context label.
- Focused compile of `stdlib/tests/list.n.md::doctest#1` then reported `resource.raw.ownership_violation` for `l3r0` / `l3r1` source list nodes and `reverse`'s `new_head`.
- The old `reverse` implementation loaded payloads from raw nodes, allocated a new node chain, and never closed the consumed input chain. For non-Copy payloads this also duplicated ownership by value.

## 問題

CI source policy required the wrong invariant: `reverse` should not allocate/copy nodes at all. It must consume the input owner and move the existing node chain into reversed order.

## 影響

List reverse could leak the source chain and duplicate owned payloads. Main CI also stopped before later jobs because the source policy was stale relative to the memory-safety requirement.

## 修正方針

Replace allocation-copy reverse with in-place node relinking, make `reverse` return `List<T>` directly, and update source policy/tests so `reverse` cannot reintroduce allocation or payload duplication.

## 修正内容

- `reverse` を `Result<List<T>, Diag>` から `List<T>` へ変更し、node allocation を行わない API にした。
- 入力 `List` の `ptr` を消費し、各 node の next pointer を `store_i32` で前 node へ付け替える relink 実装にした。
- `nodesrc/test_stdlib_list_no_unsafe_unwraps.js` は、`reverse` が `list_alloc_node` / payload `load<T> cur` / `Result::Err` を含まないことを検査する source policy へ更新した。
- `stdlib/tests/list.n.md` と `tests/stdlib/list_collections.n.md` の reverse call sites を Result unwrap/match から直接 `List` を受け取る形に更新した。
- 検証中に残った map/filter raw-node traversal timeout は `ISS-20260429T211515214Z-LIST-MAP-AND-FILTER-RAW-NODE-TRAVERS-C52D497C` として分離した。

## 検証

- `node nodesrc/test_stdlib_list_no_unsafe_unwraps.js`: passed
- `node nodesrc/run_doctest.js -i stdlib/tests/list.n.md -n 1 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/list_collections.n.md -n 1 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/list_collections.n.md -n 2 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/list.nepl -n 2 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/alloc/collections/list.nepl -n 11 --dist web/dist`: passed
