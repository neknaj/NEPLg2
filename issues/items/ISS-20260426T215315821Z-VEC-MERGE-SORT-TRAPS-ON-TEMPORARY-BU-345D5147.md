---
id: ISS-20260426T215315821Z-VEC-MERGE-SORT-TRAPS-ON-TEMPORARY-BU-345D5147
title: "vec merge sort traps on temporary buffer allocation failure"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "stdlib/alloc/collections/vec/sort.nepl, tests/stdlib/sort.n.md, nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js"
---

# ISS-20260426T215315821Z-VEC-MERGE-SORT-TRAPS-ON-TEMPORARY-BU-345D5147: vec merge sort traps on temporary buffer allocation failure

## 概要

sort_merge and sort_merge_ret allocate an O(n) temporary buffer with unwrap_ok, so ordinary allocation failure becomes unreachable instead of a Result error. The comments explicitly document the trap because the public API does not return Result.

## 対象

- `stdlib/alloc/collections/vec/sort.nepl, tests/stdlib/sort.n.md, nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js`

## 根拠

- 未記入

## 問題

sort_merge and sort_merge_ret allocate an O(n) temporary buffer with unwrap_ok, so ordinary allocation failure becomes unreachable instead of a Result error. The comments explicitly document the trap because the public API does not return Result.

## 影響

Self-host data-structure sorting cannot report OutOfMemory and still depends on unsafe Result helpers in normal stdlib code, keeping RV-STDLIB-010 open for a core collection utility.

## 修正方針

Change allocation-bearing merge sort APIs to return Result values, propagate alloc/dealloc errors with match, update callers/tests, and add a source regression that forbids unsafe unwraps in the merge-sort implementation blocks.

## 検証

Run focused sort doctests, tests/stdlib/sort.n.md, stdlib doctests, source regression, issues check, and diff check.

## 対応 2026-04-27

`sort_merge` と `sort_merge_ret` を allocation-bearing API として `Result` を返す形に変更した。
`alloc_ptr` の `Err(str)` は `StdErrorKind::OutOfMemory`、`dealloc_ptr` の `Err(str)` は `StdErrorKind::Failure` に正規化し、`unwrap_ok` / `uwok` / `unreachable` へ落とさない。
`sort_merge_ret` は成功時だけ `Ok(Vec<.T>)` として元の buffer を返し、失敗時は `Err(StdErrorKind)` を返す。

呼び出し側の sort doctest は Result API を明示的に扱う形へ更新した。
また `nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js` を追加し、`sort_merge` / `sort_merge_ret` の実装 block に unsafe unwrap が戻らないことを CI source policy regression で確認する。

検証:

- `node nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/sort.n.md --no-tree -o tmp/sort-merge-result-focused-2.json -j 1`: 22/22 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort.nepl --no-tree -o tmp/sort-merge-doc-focused-2.json -j 1`: 3/3 passed
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-sort-merge-result.json -j 4`: 282/282 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-sort-merge-result.json -j 4`: 414/414 passed
- `node nodesrc/test_stdlib_match_decision_trees.js; node nodesrc/test_stdlib_sha256_no_unsafe_unwraps.js; node nodesrc/test_run_test_wasi_tmp_dir.js; node nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
- remote main の `252f1d4 core: enforce borrow exclusivity for Copy values` へ rebase 後:
  - `node nodesrc/tests.js -i tests/stdlib/sort.n.md --no-tree -o tmp/sort-merge-result-after-borrow-rebase.json -j 1`: 22/22 passed
  - `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort.nepl --no-tree -o tmp/sort-merge-doc-after-borrow-rebase.json -j 1`: 3/3 passed
  - source policy regressions / `node nodesrc/issues.js check` / `git diff --check`: pass
