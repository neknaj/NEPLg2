---
id: ISS-20260519T134548652Z-VEC-MERGE-SORT-RAW-HELPERS-ARE-DIREC-18BA8A0F
title: "Vec merge sort raw helpers are directly importable"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-19
target: "stdlib/alloc/collections/vec/sort/merge/*.nepl, nodesrc/test_stdlib_vec_sort_module_split.js, nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js"
---

# ISS-20260519T134548652Z-VEC-MERGE-SORT-RAW-HELPERS-ARE-DIREC-18BA8A0F: Vec merge sort raw helpers are directly importable

## 概要

sort/merge/buffer.nepl and sort/merge/range.nepl define public MemPtr-based helpers. The facade no longer re-exports them, but ordinary source can still import those exact modules and call unchecked raw buffer/range operations.

## 対象

- `stdlib/alloc/collections/vec/sort/merge/*.nepl, nodesrc/test_stdlib_vec_sort_module_split.js, nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js`

## 根拠

- `stdlib/alloc/collections/vec/sort/merge/buffer.nepl` は `pub fn sort_buf_get` / `pub fn sort_buf_set` を持ち、ordinary source が direct import できた。
- `stdlib/alloc/collections/vec/sort/merge/range.nepl` は `pub fn sort_merge_range_data` を持ち、`MemPtr` と unchecked range を caller から受け取っていた。
- Stage 6 の方針では、raw traversal は len / storage view / scratch owner を検査する public wrapper と同じ source file 内の private boundary に閉じる必要がある。

## 問題

sort/merge/buffer.nepl and sort/merge/range.nepl define public MemPtr-based helpers. The facade no longer re-exports them, but ordinary source can still import those exact modules and call unchecked raw buffer/range operations.

## 影響

This preserves a raw memory authority escape in Stage 6: safe callers can bypass Vec length/storage proofs and use arbitrary MemPtr plus unchecked indexes.

## 修正方針

Move merge scratch buffer access and range traversal into the checked merge API implementation as private helpers, remove the direct-importable helper modules, and update source-policy regressions.

## 検証

Focused vec sort policy tests, merge doctests, direct import compile-fail probe, issues check, and diff check.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-6-stdlib--self-host-buffer-%E7%A7%BB%E8%A1%8C)
- [stdlib collection / mem / string static safety design](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md)

## 2026-05-19 Agent 1 解決

`sort/merge/buffer.nepl` と `sort/merge/range.nepl` を削除し、`sort_merge_buffer_get` / `sort_merge_buffer_set` / `sort_merge_range_data` を `sort/merge/api.nepl` の private helper に統合した。

これにより `sort/merge` facade だけでなく、explicit `alloc/collections/vec/sort/merge/buffer` / `range` import からも unchecked raw helper を呼べない。`sort_merge` / `sort_merge_ret` は引き続き `Vec` owner から得た data view と scratch `RegionToken` owner だけを使い、allocation / cleanup failure は `Result` payload で返す。

検証:

- `node nodesrc/test_stdlib_vec_sort_module_split.js`
- `node nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`
- `node nodesrc/test_stdlib_documentation_contract.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort/merge/api.nepl -o tmp\agent1-vec-merge-api-private-helper-doc.json --no-tree -j 4`
- `node nodesrc/tests.js -i tests/stdlib/sort.n.md -o tmp\agent1-vec-merge-private-boundary-sort.json --no-tree -j 4`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort -o tmp\agent1-vec-merge-private-boundary-sort-modules.json --no-tree -j 4`
- temporary compile-fail probe: direct `alloc/collections/vec/sort/merge/buffer` import is rejected after helper module deletion
- `cargo test -p nepl-core --test effects loader_marks_configured_stdlib_implementation_boundaries_as_raw_memory_boundary`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
