---
id: ISS-20260430T064223548Z-NM-BYTE-SCANNER-POLICY-STILL-EXPECTS-18CC8784
title: "NM byte scanner policy still expects older CR trimming pattern"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/nm/html_gen.nepl, nodesrc/test_stdlib_byte_scanner_helpers_boundary.js"
---

# ISS-20260430T064223548Z-NM-BYTE-SCANNER-POLICY-STILL-EXPECTS-18CC8784: NM byte scanner policy still expects older CR trimming pattern

## 概要

nodesrc/test_stdlib_byte_scanner_helpers_boundary.js expects nm parser/html_gen to call str_trim_suffix_cr directly, but the current string API has str_slice_trim_suffix_cr to combine slicing and CR trimming without an intermediate string. The parser already uses the newer helper while html_gen still composes str_trim_suffix_cr over str_slice.

## 対象

- `stdlib/nm/html_gen.nepl, nodesrc/test_stdlib_byte_scanner_helpers_boundary.js`

## 根拠

- `stdlib/alloc/string.nepl` には `str_slice_trim_suffix_cr` があり、`str_trim_suffix_cr str_slice ...` と同じ結果を不要な中間 `str` なしで返す。
- `stdlib/nm/parser.nepl` は既に `str_slice_trim_suffix_cr` を使っている。
- `stdlib/nm/html_gen.nepl` は `str_trim_suffix_cr str_slice ...` の古い合成 pattern を残していた。
- `nodesrc/test_stdlib_byte_scanner_helpers_boundary.js` は古い `str_trim_suffix_cr` 直接呼び出しだけを要求しており、現行 parser を誤って拒否していた。

## 問題

nodesrc/test_stdlib_byte_scanner_helpers_boundary.js expects nm parser/html_gen to call str_trim_suffix_cr directly, but the current string API has str_slice_trim_suffix_cr to combine slicing and CR trimming without an intermediate string. The parser already uses the newer helper while html_gen still composes str_trim_suffix_cr over str_slice.

## 影響

The source policy cannot be added to the aggregate runner while it fails against current parser code. Keeping html_gen on the older composed pattern also leaves an unnecessary intermediate string in repeated line scanning.

## 修正方針

Update html_gen line reads to use str_slice_trim_suffix_cr and update the policy to require that helper in both parser and html_gen while still keeping scanner line navigation checks.

## 検証

Run node nodesrc/test_stdlib_byte_scanner_helpers_boundary.js, node nodesrc/run_source_policy_regressions.js, node nodesrc/issues.js check, and git diff --check.

実行済み:

- `node nodesrc/test_stdlib_byte_scanner_helpers_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/nm/html_gen.nepl --no-tree -o tmp/nm-html-gen-byte-scanner-policy.json -j 1 --dist web/dist`: `total=2`, `passed=2`
