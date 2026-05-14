---
id: ISS-20260514T221807506Z-STDLIB-STRING-DOCTESTS-RETAIN-STALE--CC9D6303
title: "stdlib string doctests retain stale import assumptions"
area: stdlib
status: open
resolved: false
priority: P2
type: test
created: 2026-05-14
updated: 2026-05-14
target: stdlib/tests/string.n.md
---

# ISS-20260514T221807506Z-STDLIB-STRING-DOCTESTS-RETAIN-STALE--CC9D6303: stdlib string doctests retain stale import assumptions

## 概要

Focused string regression runs still fail two doctests that are unrelated to the alloc/string raw facade split: string_find_byte_index references Result without importing core/result, and string_to_f64_parser references cast without importing core/cast under the current module/facade boundaries.

## 対象

- `stdlib/tests/string.n.md`

## 根拠

- 未記入

## 問題

Focused string regression runs still fail two doctests that are unrelated to the alloc/string raw facade split: string_find_byte_index references Result without importing core/result, and string_to_f64_parser references cast without importing core/cast under the current module/facade boundaries.

## 影響

The stale fixtures make broad string verification report 7/9 instead of a clean pass, which can hide real raw-boundary or parser regressions during Stage 6 work.

## 修正方針

Add the explicit stdlib imports required by the doctests and rerun stdlib/tests/string.n.md plus the relevant string source policies.

## 検証

Run node nodesrc/tests.js -i stdlib/tests/string.n.md --no-tree -o tmp/string-doctest-import-drift.json -j 1 --dist web/dist --assert-io and confirm all doctests pass.
