---
id: ISS-20260514T003227548Z-DEBUG-U8-IMPL-MISSES-CAST-IMPORT-CC1EFBAA
title: "Debug u8 impl misses cast import"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/core/traits/debug.nepl, tests/stdlib/traits_text.n.md"
---

# ISS-20260514T003227548Z-DEBUG-U8-IMPL-MISSES-CAST-IMPORT-CC1EFBAA: Debug u8 impl misses cast import

## 概要

core/traits/debug.nepl defines Debug for u8 using cast but does not import core/cast. A doctest that instantiates Debug for u8 fails with resolve.identifier.undefined at stdlib/core/traits/debug.nepl.

## 対象

- `stdlib/core/traits/debug.nepl, tests/stdlib/traits_text.n.md`

## 根拠

- `node nodesrc\tests.js -i tests\stdlib\traits_text.n.md --no-tree -o tmp\agent1-traits-text-before.json -j 1 --assert-io --dist web/dist` で `tests/stdlib/traits_text.n.md::doctest#3` が失敗した。
- compiler diagnostic は `/stdlib/core/traits/debug.nepl:75` の `from_i32 cast x` に対する `resolve.identifier.undefined` で、`core/traits/debug.nepl` が `core/cast` を import していないことが原因だった。

## 問題

core/traits/debug.nepl defines Debug for u8 using cast but does not import core/cast. A doctest that instantiates Debug for u8 fails with resolve.identifier.undefined at stdlib/core/traits/debug.nepl.

## 影響

Debug trait coverage is incomplete for u8, and stdout report migration for traits_text cannot verify the current trait surface without exposing this compile failure.

## 修正方針

Import core/cast in core/traits/debug.nepl and add/update focused doctest coverage so Debug u8 instantiation compiles through the public trait API.

## 検証

node nodesrc/tests.js -i tests/stdlib/traits_text.n.md --no-tree -o tmp/agent1-traits-text-report-tests.json -j 1 --assert-io --dist web/dist

## 解決

`core/traits/debug.nepl` に `core/cast` の明示 import を追加し、`Debug for u8` の実装が定義元 module の依存だけで解決できるようにした。これにより `tests/stdlib/traits_text.n.md` と `stdlib/core/traits/debug.nepl` の focused doctest が通る。

検証:

- `node nodesrc\tests.js -i tests\stdlib\traits_text.n.md --no-tree -o tmp\agent1-traits-text-debug-import.json -j 1 --assert-io --dist web/dist`: total=3, passed=3
- `node nodesrc\tests.js -i stdlib\core\traits\debug.nepl --no-tree -o tmp\agent1-debug-trait-doctests.json -j 1 --assert-io --dist web/dist`: total=1, passed=1
