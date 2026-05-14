---
id: ISS-20260514T223843320Z-STD-IO-DOCTEST-OMITS-EXPLICIT-IOTARG-12D221C3
title: "std/io doctest omits explicit iotarget import"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/std/io.nepl, stdlib/std/iotarget.nepl, tests/stdlib/io.n.md, nodesrc/test_stdlib_io_target_facade.js"
---

# ISS-20260514T223843320Z-STD-IO-DOCTEST-OMITS-EXPLICIT-IOTARG-12D221C3: std/io doctest omits explicit iotarget import

## 概要

The std/io facade doctest imports std/io and core/result, then constructs WriteStream::Stdio directly. WriteStream is defined in std/iotarget and is not exported by std/io, so the doctest fails with resolve.identifier.undefined when run as a module doctest.

## 対象

- `stdlib/std/io.nepl, stdlib/std/iotarget.nepl, tests/stdlib/io.n.md, nodesrc/test_stdlib_io_target_facade.js`

## 根拠

- `std/io` の `read` / `write` / `flush` / `close` API は `ReadStream` / `WriteStream` を public signature に含む。
- `tests/stdlib/io.n.md` は `#import "std/io" as *` だけで `ReadStream` / `WriteStream` を使う既存 contract になっている。
- `stdlib/std/io.nepl::doctest#1` は root module doctest として同じ contract を例示していたが、root facade が `std/iotarget` を public re-export していないため `resolve.identifier.undefined` で失敗した。

## 問題

The std/io facade doctest imports std/io and core/result, then constructs WriteStream::Stdio directly. WriteStream is defined in std/iotarget and is not exported by std/io, so the doctest fails with resolve.identifier.undefined when run as a module doctest.

## 影響

Focused std/io verification fails for an import drift that is unrelated to the checked text conversion boundary. This obscures real std/io regressions and leaves documentation examples inaccurate about where target enums come from.

## 修正方針

Either make std/io intentionally re-export the target enum surface, or update the doctest to import std/iotarget explicitly after confirming the facade design. Do not add implicit raw or unrelated stdlib exports.

## 検証

Run stdlib/std/io.nepl focused doctest and issue validation.

## 結果

- `std/io` root が `std/iotarget` を public re-export する設計にした。
- `ReadStream` / `WriteStream` の定義は `std/iotarget` に残し、`std/io` に enum 定義を重複させない。
- `std/iotarget` は target vocabulary module として維持し、実行関数・trait・raw memory authority を持たないことを source policy で固定した。
- `tests/stdlib/io.n.md` の missing-file case は、実際に使う `alloc/string` と `core/result` を明示 import するようにした。`std/io` は string utility や Result constructor の facade ではないため、ここは root re-export で広げない。

## 検証結果

- `node nodesrc/test_stdlib_io_target_facade.js`: pass
- `node nodesrc/tests.js -i stdlib/std/io.nepl --no-tree -o tmp/agent1-std-io-iotarget-facade-module.json -j 1 --dist web/dist --assert-io`: pass
- `node nodesrc/tests.js -i tests/stdlib/io.n.md --no-tree -o tmp/agent1-std-io-iotarget-facade-suite.json -j 1 --dist web/dist --assert-io`: pass
