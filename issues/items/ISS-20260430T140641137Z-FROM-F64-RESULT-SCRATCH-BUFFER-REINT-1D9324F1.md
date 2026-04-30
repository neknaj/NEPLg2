---
id: ISS-20260430T140641137Z-FROM-F64-RESULT-SCRATCH-BUFFER-REINT-1D9324F1
title: "from_f64_result scratch buffer reintroduces Resource IR moved-cell failures"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/string.nepl, stdlib/alloc/collections/hashmap.nepl, stdlib/alloc/collections/hashset.nepl, GitHub Actions run 25157230630 stdlib-test"
---

# ISS-20260430T140641137Z-FROM-F64-RESULT-SCRATCH-BUFFER-REINT-1D9324F1: from_f64_result scratch buffer reintroduces Resource IR moved-cell failures

## 概要

GitHub Actions run 25157230630 stdlib-test reports resource.cell.possibly_moved in from_f64_result__f64__Result_T_E_str_i32__pure. HashMap/HashSet doctests fail before exercising collection logic because from_f64_result allocates a six-byte scratch buffer, passes the MemPtr through string_from_mem_unchecked_result, then reads/trims/deallocates via the raw scratch address in a way Resource IR sees as MaybeMoved.

## 対象

- `stdlib/alloc/string.nepl, stdlib/alloc/collections/hashmap.nepl, stdlib/alloc/collections/hashset.nepl, GitHub Actions run 25157230630 stdlib-test`

## 根拠

- `gh run view 25157230630 --job 73742416567 --log-failed` で `stdlib-test` が `415 total / 232 passed / 173 failed / 10 errored` で失敗していることを確認した。
- `gh run download 25157230630 -n stdlib-tests` で取得した `stdlib-tests.json` では、`stdlib/alloc/collections/hashmap.nepl::doctest#1..#3` と `stdlib/alloc/collections/hashset.nepl::doctest#1..#6` が `from_f64_result__f64__Result_T_E_str_i32__pure` の `resource.cell.possibly_moved` で compile failure になっている。
- 該当実装は `alloc_ptr<u8> 6` で scratch を確保し、`scratch_raw = mem_ptr_addr scratch` を使って digit を書いた後、`string_from_mem_unchecked_result scratch trim` と `dealloc_raw scratch_raw 6` の境界を跨いでいる。

## 問題

GitHub Actions run 25157230630 stdlib-test reports resource.cell.possibly_moved in from_f64_result__f64__Result_T_E_str_i32__pure. HashMap/HashSet doctests fail before exercising collection logic because from_f64_result allocates a six-byte scratch buffer, passes the MemPtr through string_from_mem_unchecked_result, then reads/trims/deallocates via the raw scratch address in a way Resource IR sees as MaybeMoved.

## 影響

The stdlib-test job cannot be used as a clean collection regression signal: string f64 formatting failures mask HashMap/HashSet doctests and can encourage weakening Resource IR cell checks. This is also a regression risk for selfhost diagnostics or JSON/report paths that need numeric formatting.

## 修正方針

Do not weaken Resource IR. Redesign from_f64_result so fractional digit generation does not reuse a raw scratch owner after a conversion boundary. Prefer direct StringBuilder or owned string region construction, or keep scratch storage behind a clear borrowed view and exactly-once free path that Resource IR can prove. Add a source policy/regression that prevents reintroducing scratch_raw MaybeMoved paths in string numeric formatting.

## 検証

Use gh to confirm the affected Actions stdlib-test failure disappears. For implementation pre-commit checks, run focused string/hash collection doctests, node nodesrc/test_stdlib_string_no_unsafe_unwraps.js, node nodesrc/issues.js check, and git diff --check.
