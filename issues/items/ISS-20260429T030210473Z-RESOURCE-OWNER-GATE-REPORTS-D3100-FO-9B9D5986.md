---
id: ISS-20260429T030210473Z-RESOURCE-OWNER-GATE-REPORTS-D3100-FO-9B9D5986
title: "Resource owner gate reports D3100 for stdlib byte scanner helper doctest temporaries"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource, stdlib/alloc/string.nepl"
---

# ISS-20260429T030210473Z-RESOURCE-OWNER-GATE-REPORTS-D3100-FO-9B9D5986: Resource owner gate reports D3100 for stdlib byte scanner helper doctest temporaries

## 概要

A focused alloc/string doctest for str_find_byte_range, str_line_end, str_next_line_pos, str_trim_suffix_cr, and ASCII byte classification fails in compile phase with D3100 owner obligation leaks on main temporaries, even though the helpers operate on Copy scalar offsets and borrowed str views.

## 対象

- `nepl-core/src/resource, stdlib/alloc/string.nepl`

## 根拠

- `node nodesrc/tests.js -i stdlib\alloc\string.nepl --no-tree -o tmp\stdlib-byte-scanner-string-4.json -j 1` は 8 件中 7 passed / 1 failed。
- 失敗した追加 doctest は `str_find_byte_range`、`str_line_end`、`str_next_line_pos`、`str_trim_suffix_cr`、ASCII byte classification を `main` 内で確認するだけだが、compile phase で `D3100 resource ir owner obligation may leak in function 'main__unit__i32__imp'` になった。
- `node nodesrc/test_stdlib_byte_scanner_helpers_boundary.js` は成功し、source 構造上は helper が追加され call site 置換も済んでいるため、挙動検証を阻む Resource owner gate 残件として切り分ける。

## 問題

A focused alloc/string doctest for str_find_byte_range, str_line_end, str_next_line_pos, str_trim_suffix_cr, and ASCII byte classification fails in compile phase with D3100 owner obligation leaks on main temporaries, even though the helpers operate on Copy scalar offsets and borrowed str views.

## 影響

New stdlib scanner helpers cannot be behaviorally verified inside NEPL doctests; scanner refactors must rely on structural tests until Resource IR owner flow handles these pure temporaries correctly.

## 修正方針

Trace Resource IR owner obligations for pure i32/bool return temporaries and borrowed str results in nested conditionals without weakening D3100 for real owner leaks. Add the byte scanner doctest as a regression when fixed.

## 対応

remote main の Resource owner summary 修正により、copy-like return temporary と consumed owner parameter の扱いが改善された。`str_find_byte_range` / `str_line_end` / `str_next_line_pos` / `str_trim_suffix_cr` / ASCII byte classification の semantic sample は D3100 なしで compile/run できるようになったため、skip を外して通常 doctest に戻した。

## 検証

- `trunk build`: passed
- `node nodesrc/tests.js -i stdlib\alloc\string.nepl --no-tree -o tmp\stdlib-byte-scanner-string-unskip.json -j 1`: total=8 passed=8
