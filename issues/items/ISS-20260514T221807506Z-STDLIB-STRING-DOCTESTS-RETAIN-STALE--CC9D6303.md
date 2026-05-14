---
id: ISS-20260514T221807506Z-STDLIB-STRING-DOCTESTS-RETAIN-STALE--CC9D6303
title: "stdlib string doctests retain stale import assumptions"
area: stdlib
status: fixed
resolved: true
priority: P2
type: test
created: 2026-05-14
updated: 2026-05-15
target: stdlib/tests/string.n.md
---

# ISS-20260514T221807506Z-STDLIB-STRING-DOCTESTS-RETAIN-STALE--CC9D6303: stdlib string doctests retain stale import assumptions

## 概要

Focused string regression runs still fail two doctests that are unrelated to the alloc/string raw facade split: string_find_byte_index references Result without importing core/result, and string_to_f64_parser references cast without importing core/cast under the current module/facade boundaries.

## 対象

- `stdlib/tests/string.n.md`

## 根拠

- `stdlib/tests/string.n.md` focused run は `alloc/string` raw facade split 後に 9 件中 7 件だけ pass し、残る failure は raw facade 変更箇所ではなく doctest 自体の不足 import だった。
- `string_find_byte_index` は helper signature と branch で `Result<(),str>` を使うが `core/result` を import していない。
- `string_to_f64_parser` は期待値構築で `cast` を使うが `core/cast` を import していない。

## 問題

Focused string regression runs still fail two doctests that are unrelated to the alloc/string raw facade split: string_find_byte_index references Result without importing core/result, and string_to_f64_parser references cast without importing core/cast under the current module/facade boundaries.

## 影響

The stale fixtures make broad string verification report 7/9 instead of a clean pass, which can hide real raw-boundary or parser regressions during Stage 6 work.

## 修正方針

Add the explicit stdlib imports required by the doctests and rerun stdlib/tests/string.n.md plus the relevant string source policies.

## 検証

Run node nodesrc/tests.js -i stdlib/tests/string.n.md --no-tree -o tmp/string-doctest-import-drift.json -j 1 --dist web/dist --assert-io and confirm all doctests pass.

## 解決内容

- `string_find_byte_index` doctest に `#import "core/result" as *` を追加した。
- `string_to_f64_parser` doctest に `#import "core/cast" as *` を追加した。
- `alloc/string` root facade や `std/test` から暗黙に型・変換 helper が見える前提を残さず、doctest が使う module を明示する形に揃えた。

## 検証結果

- `node nodesrc/tests.js -i stdlib/tests/string.n.md --no-tree -o tmp/agent1-string-doctest-import-drift.json -j 1 --dist web/dist --assert-io`: 9/9 passed
