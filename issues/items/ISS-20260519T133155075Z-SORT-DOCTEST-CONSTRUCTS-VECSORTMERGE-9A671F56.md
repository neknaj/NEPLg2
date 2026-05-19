---
id: ISS-20260519T133155075Z-SORT-DOCTEST-CONSTRUCTS-VECSORTMERGE-9A671F56
title: "sort doctest constructs VecSortMergeError outside owner aggregate boundary"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-19
target: "tests/stdlib/sort.n.md; stdlib/alloc/collections/vec/sort/merge/api.nepl"
---

# ISS-20260519T133155075Z-SORT-DOCTEST-CONSTRUCTS-VECSORTMERGE-9A671F56: sort doctest constructs VecSortMergeError outside owner aggregate boundary

## 概要

tests/stdlib/sort.n.md includes a normal running doctest that directly constructs VecSortMergeError<i32>. Current owner-backed aggregate policy correctly rejects ordinary source constructors for owner-carrying aggregates, so the doctest fails with type.owner_aggregate.constructor_restricted instead of validating the public error-recovery surface.

## 対象

- `tests/stdlib/sort.n.md; stdlib/alloc/collections/vec/sort/merge/api.nepl`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/sort.n.md -o tmp\agent1-vec-sort-raw-boundary-sort.json --no-tree -j 4` で `tests\stdlib\sort.n.md::doctest#17` が compile failure になった。
- 該当 doctest は `let err <VecSortMergeError<i32>> VecSortMergeError<i32> v1 StdErrorKind::OutOfMemory;` により ordinary source から owner-carrying error aggregate を直接構築している。
- compiler は `type.owner_aggregate.constructor_restricted` を出しており、これは owner-backed aggregate constructor を通常 source に許さない現在のメモリ安全方針と一致する。

## 問題

tests/stdlib/sort.n.md includes a normal running doctest that directly constructs VecSortMergeError<i32>. Current owner-backed aggregate policy correctly rejects ordinary source constructors for owner-carrying aggregates, so the doctest fails with type.owner_aggregate.constructor_restricted instead of validating the public error-recovery surface.

## 影響

The broader sort doctest suite cannot be used as a clean regression signal, and the fixture teaches an invalid owner-backed aggregate construction pattern that conflicts with Stage 6 memory-safety policy.

## 修正方針

Replace the fixture with a valid public API scenario or add a compiler-owned stdlib helper that produces the error payload through the same boundary as sort_merge_ret. Do not weaken owner-backed aggregate constructor restrictions or allow ordinary source to forge VecSortMergeError.

## 検証

Run tests/stdlib/sort.n.md, Vec sort source-policy regressions, and owner-backed aggregate constructor restriction regressions.

## 対応

- `tests/stdlib/sort.n.md` の `sort_merge_ret_error_payload_returns_vec_owner` を、直接 constructor が拒否されることを確認する `compile_fail` regression に変更した。
- `VecSortMergeError` は `sort_merge_ret` の失敗 branch が返す payload であり、ordinary source が直接構築して owner を捏造する使い方は不正だと `merge/api.nepl` のドキュメントに明記した。
- owner-backed aggregate constructor restriction は緩めず、既存の static safety 方針をそのまま維持した。

## 検証結果

- `node nodesrc/tests.js -i tests/stdlib/sort.n.md -o tmp\agent1-sort-merge-error-doctest.json --no-tree -j 4`: total=20, passed=20
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js`: pass
