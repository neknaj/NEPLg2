---
id: ISS-20260513T044704155Z-STD-TEST-AND-STDIO-NUMERIC-ADD-RESOL-9CABF45B
title: "std-test and stdio numeric add resolution is polluted by collection add exports"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "stdlib/std/{test/report.nepl,stdio/write/fd.nepl,stdio/read/buffer.nepl}"
---

# ISS-20260513T044704155Z-STD-TEST-AND-STDIO-NUMERIC-ADD-RESOL-9CABF45B: std-test and stdio numeric add resolution is polluted by collection add exports

## 概要

Fenwick exposes a public add API. When a doctest imports Fenwick together with std/test stdout reporting, std/test/report.nepl and std/stdio internal unqualified add expressions can resolve against the collection add surface instead of the intended core/math addition, causing type.overload.no_match in unrelated stdio/report code.

## 対象

- `stdlib/std/test/report.nepl`
- `stdlib/std/stdio/write/fd.nepl`
- `stdlib/std/stdio/read/buffer.nepl`

## 根拠

- `alloc/collections/fenwick` は public `add` API を持つ。
- Fenwick と `std/test` を同じ doctest で使うと、`std/test/report.nepl` の `TestReport name0 add count0 1 ...` と `std/stdio` の iovec offset 計算が `core/math::add` ではなく collection `add` の影響を受け、`type.overload.no_match` を出していた。
- `std/test` / `std/stdio` は利用者が import した facade の public function 名に依存せず、自身の内部 arithmetic を明示的に解決する必要がある。

## 問題

Fenwick exposes a public add API. When a doctest imports Fenwick together with std/test stdout reporting, std/test/report.nepl and std/stdio internal unqualified add expressions can resolve against the collection add surface instead of the intended core/math addition, causing type.overload.no_match in unrelated stdio/report code.

## 影響

Canonical stdout report migration cannot cover collection modules that expose add-like APIs, and safe user code can make std/test or stdio fail by importing an unrelated collection facade.

## 修正方針

Make std/test report and stdio raw buffer/write internals qualify their numeric arithmetic through a core/math module alias instead of relying on wildcard unqualified add/eq/lt helpers.

## 検証

Focused doctest that imports alloc/collections/fenwick and std/test together must compile and print a canonical report; Fenwick stdout-report migration tests must pass.

## 2026-05-13 修正

`std/test/report.nepl`、`std/stdio/write/fd.nepl`、`std/stdio/read/buffer.nepl` の `core/math` import を wildcard から `math` alias へ切り替え、内部の `add` / `sub` / `eq` / `lt` / `le` / `gt` / `ge` / `ne` / `or` を `math::...` として明示した。

これにより、利用者側や collection facade が `add` などの一般名を public API として持っていても、stdio の raw scratch layout と std/test report の count 加算が汚染されない。

回帰テストとして `tests/stdlib/std_test_namespace_resolution.n.md` を追加し、Fenwick の `fw::add` と `std/test` の stdout report を同じ doctest で使えることを固定した。

あわせて `tests/stdlib/std_test_collect.n.md` の doctest が `std/test` 内部の `alloc/string` import に便乗して `concat` を使っていたため、doctest 自身へ `alloc/string` の直接 import を追加した。これにより、`std/test` の内部依存を利用者の名前空間契約として漏らさない。
