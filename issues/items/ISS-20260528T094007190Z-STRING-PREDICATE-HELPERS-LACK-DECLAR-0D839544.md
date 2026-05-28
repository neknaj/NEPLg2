---
id: ISS-20260528T094007190Z-STRING-PREDICATE-HELPERS-LACK-DECLAR-0D839544
title: "String predicate helpers lack declaration doctests"
area: stdlib
status: verified
resolved: true
priority: P1
type: doc
created: 2026-05-28
updated: 2026-05-28
target: "stdlib/alloc/string/byte_index.nepl, stdlib/alloc/string/search/compare.nepl"
---

# ISS-20260528T094007190Z-STRING-PREDICATE-HELPERS-LACK-DECLAR-0D839544: String predicate helpers lack declaration doctests

## 概要

The global stdlib documentation contract reported declarationNoDoctest=1065 while the frozen baseline is 1062. The missing executable examples came from string byte predicate helpers added during performance work.

## 対象

- `stdlib/alloc/string/byte_index.nepl, stdlib/alloc/string/search/compare.nepl`

## 根拠

- subagent 調査で、`declarationNoDoc` は baseline 以下のままであり、説明コメント不足ではなく `neplg2:test` 不足だけが `1065 > 1062` の原因だと確認した。
- `string_byte_or_invalid` と `string_byte_is_ascii_space` は `28f8a0e7 perf: dedup resource path states and trim byte checks` で追加され、説明はあるが declaration doctest がなかった。
- `str_byte_is_ascii_space_at` は `68ce2a4c perf: route string trim through search predicate` で追加され、説明はあるが declaration doctest がなかった。
- 3 件とも範囲外 index を `false` または private sentinel 経由の非一致として扱う low-level predicate であり、境界挙動を executable documentation として固定する価値がある。

## 問題

The global stdlib documentation contract reported declarationNoDoctest=1065 while the frozen baseline is 1062. The missing executable examples came from string byte predicate helpers added during performance work.

## 影響

Source policy remains warn-only for a documentation regression even though the affected helpers are low-level bounds-checked string predicates that should have executable examples for in-range and out-of-range behavior.

## 修正方針

Add meaningful NEPL doc tests for string_byte_or_invalid through public behavior, string_byte_is_ascii_space, and str_byte_is_ascii_space_at. Do not relax the documentation baseline or remove comments.

## 検証

node nodesrc/test_stdlib_documentation_contract.js; node nodesrc/run_doctest.js -i stdlib/alloc/string/byte_index.nepl -n 4 --dist web/dist; node nodesrc/run_doctest.js -i stdlib/alloc/string/byte_index.nepl -n 5 --dist web/dist; node nodesrc/run_doctest.js -i stdlib/alloc/string/search/compare.nepl -n 1 --dist web/dist; node nodesrc/test_stdlib_string_access_boundary.js; node nodesrc/test_stdlib_string_search_boundary.js; node nodesrc/issues.js check --dir issues; git diff --check

## 2026-05-28 修正

- `string_byte_or_invalid` には private sentinel を直接公開せず、`string_byte_eq` を通じて範囲内一致と範囲外非一致を固定する doctest を追加した。
- `string_byte_is_ascii_space` には space / tab の true case と、通常 byte / 範囲外 index の false case を固定する doctest を追加した。
- `str_byte_is_ascii_space_at` には root `alloc/string` facade 経由で space / tab / LF / CR の true case と、通常 byte / negative index / end index の false case を固定する std/test doctest を追加した。
- baseline 緩和やコメント削除は行っていない。
