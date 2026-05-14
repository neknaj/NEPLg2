---
id: ISS-20260514T175345899Z-VEC-IN-PLACE-SORT-APIS-KEEP-PURE-EFF-6D4F3ABB
title: "Vec in-place sort APIs keep pure effect signatures"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/alloc/collections/vec/sort/**"
---

# ISS-20260514T175345899Z-VEC-IN-PLACE-SORT-APIS-KEEP-PURE-EFF-6D4F3ABB: Vec in-place sort APIs keep pure effect signatures

## 概要

Vec sort helpers and public in-place sort APIs mutate backing storage through raw stores, but many signatures still use pure -> function types. Resource IR can still diagnose some callers from body effects, yet the stdlib API contract and source policy do not encode mutability at the function type boundary.

## 対象

- `stdlib/alloc/collections/vec/sort/**`

## 根拠

- `stdlib/alloc/collections/vec/sort/common.nepl` の `sort_set_unchecked` / `sort_swap` 系 helper は `store<T>` による backing storage mutation を行う。
- `stdlib/alloc/collections/vec/sort/quick.nepl` / `heap.nepl` / `simple/*.nepl` の public in-place sort は `&Vec<T>` から得た raw view を通じて storage を書き換える。
- `sort_merge` / `sort_merge_ret` は既に impure `*>` signature であり、他の in-place sort family だけが effect contract と不整合だった。

## 問題

Vec sort helpers and public in-place sort APIs mutate backing storage through raw stores, but many signatures still use pure -> function types. Resource IR can still diagnose some callers from body effects, yet the stdlib API contract and source policy do not encode mutability at the function type boundary.

## 影響

Effect safety remains harder to audit: callers and future self-host code can see a pure-looking sort API for an in-place mutation, and future refactors can accidentally rely on signature-level purity instead of the Resource IR raw-effect proof.

## 修正方針

Change raw write/swap sort helpers, in-place Vec sort APIs, raw slice sort adapters, and owner-returning sort wrappers to impure *> signatures where they mutate storage. Keep pure observer helpers pure. Add doctest and source policy coverage so mutating sort signatures cannot return to ->.

## 検証

Run the vec sort source policy plus focused sort doctests for sort.n.md and sort_simple.n.md, then issues check.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection / memory / string static safety design](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)

## 2026-05-15 Agent 1 解決

mutating sort family の型契約を実装の effect と揃えた。`sort_set_unchecked` / `sort_swap` / `sort_quick_range_data` / `sort_heap_sift_down_data` / `sort_merge_range_data` / `sort_buf_set` のような raw storage write helper と、`sort_quick` / `sort_heap` / simple sort / `sort_i32` / `sort_quick_ret` / `sort_heap_ret` / default `sort` は、いずれも impure `*>` signature へ変更した。

純粋に読むだけの `sort_get_unchecked`、比較 helper、`sort_is_sorted` は pure のまま残した。これにより、API 型から「観察」と「破壊的更新」の境界が読める。

回帰として `tests/stdlib/sort.n.md` に pure function から `sort` を呼ぶ compile-fail を追加し、`nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に mutating sort signatures が `*>` であることを検査する source policy を追加した。

検証:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/tests.js -i tests/stdlib/sort.n.md --no-tree -o tmp/agent1-vec-sort-effect-contract-sort.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/sort_simple.n.md --no-tree -o tmp/agent1-vec-sort-effect-contract-sort-simple.json -j 1 --dist web/dist --assert-io`
