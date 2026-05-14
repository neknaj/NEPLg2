---
id: ISS-20260514T102108865Z-VEC-SORT-MERGE-RET-ERR-PATH-LOSES-CO-98B83660
title: "Vec sort_merge_ret Err path loses consumed Vec owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/alloc/collections/vec/sort/merge/api.nepl, tests/stdlib/sort.n.md, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260514T102108865Z-VEC-SORT-MERGE-RET-ERR-PATH-LOSES-CO-98B83660: Vec sort_merge_ret Err path loses consumed Vec owner

## 概要

sort_merge_ret consumes Vec<T> but returns Result<Vec<T>, StdErrorKind>. On scratch allocation failure or cleanup error the Err branch cannot return the input Vec owner, so the API type does not prove caller cleanup ownership. The function also still treats typed scratch dealloc failure as unreachable, reintroducing an impossible branch in normal collection internals.

## 対象

- `stdlib/alloc/collections/vec/sort/merge/api.nepl, tests/stdlib/sort.n.md, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- 未記入

## 問題

sort_merge_ret consumes Vec<T> but returns Result<Vec<T>, StdErrorKind>. On scratch allocation failure or cleanup error the Err branch cannot return the input Vec owner, so the API type does not prove caller cleanup ownership. The function also still treats typed scratch dealloc failure as unreachable, reintroducing an impossible branch in normal collection internals.

## 影響

The central Vec sorting API violates the Stage 6 owner-preserving fallible update rule. Callers cannot recover the collection owner on failure, and Resource IR has to rely on implementation discipline instead of a result type that carries ownership explicitly.

## 修正方針

Introduce an owner-returning VecSortMergeError<T> and change sort_merge_ret to return Result<Vec<T>, VecSortMergeError<T>>. Construct Err with the original Vec owner and StdErrorKind. Replace unreachable scratch cleanup branches with explicit Result errors and update doctests/source policies to require owner-preserving error handling.

## 検証

Run focused sort and vec collection doctests, source policy regressions for Vec sort merge, issues check, and diff check.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)

## 2026-05-14 Agent 1 修正結果

`sort_merge_ret<T>` の失敗 payload を `StdErrorKind` から `VecSortMergeError<T>` に変更した。

- `VecSortMergeError<T>` は `vec: Vec<T>` と `error: StdErrorKind` を保持する。
- `vec_sort_merge_error_vec<T>` で失敗時の `Vec` owner を回収できる。
- `vec_sort_merge_error_kind<T>` で owner を動かさず error kind を読める。
- scratch buffer は `alloc_ptr` / `dealloc_ptr` ではなく `alloc_region<T>` / `dealloc_region<T>` で所有し、`MemPtr<T>` は `region_ptr &buf_region` から得る non-owning view に限定した。
- `sort_merge` / `sort_merge_ret` の scratch cleanup は `unreachable` へ落とさず、`StdErrorKind::InvalidOperation` として明示的な `Result` error にする。

この修正により、Stage 6 の「fallible owning collection update は失敗時にも owner を型で返す」規則に `sort_merge_ret` を合わせた。静的検査は緩めず、Resource IR が検出した scratch buffer owner leak は `RegionToken` owner 境界へ移すことで解消した。

## 回帰テスト

- `node nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort.nepl -i tests/stdlib/sort.n.md -i tests/stdlib/vec_collections.n.md --no-tree -o tmp/agent1-vec-sort-merge-owner-error-4.json -j 4 --dist web/dist`: total=29, passed=29
