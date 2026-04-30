---
id: ISS-20260430T024859462Z-FENWICK-SUM-OBSERVER-APIS-CONSUME-OW-2BB59F00
title: "Fenwick sum observer APIs consume owners by value instead of borrowing"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: stdlib/alloc/collections/fenwick.nepl
---

# ISS-20260430T024859462Z-FENWICK-SUM-OBSERVER-APIS-CONSUME-OW-2BB59F00: Fenwick sum observer APIs consume owners by value instead of borrowing

## 概要

Fenwick len/sum_prefix/sum_range take Fenwick by value although they only read the internal tree. A range query moves the tree owner and prevents later cleanup, which is the same root ownership-design defect fixed for BitSet.

## 対象

- `stdlib/alloc/collections/fenwick.nepl`

## 根拠

- `stdlib/alloc/collections/fenwick.nepl` に `fn len <(Fenwick)->i32>`、`fn sum_prefix <(Fenwick,i32)*>Result<i32, Diag>>`、`fn sum_range <(Fenwick,i32,i32)*>Result<i32, Diag>>` が残っている。
- BitSet の owner-consuming observer 修正中に raw-array collection を確認し、internal tree を読むだけの Fenwick query API も値 receiver のままだと判明した。

## 問題

Fenwick len/sum_prefix/sum_range take Fenwick by value although they only read the internal tree. A range query moves the tree owner and prevents later cleanup, which is the same root ownership-design defect fixed for BitSet.

## 影響

Self-host numeric analysis code cannot safely run multiple Fenwick queries or free the tree after a query without bypassing the public APIs. This undermines mandatory memory-safety checking.

## 修正方針

Change Fenwick observer/query APIs to take &Fenwick, read Copy fields from borrowed references, update doctests/tests to query through borrowed receivers, and keep add as the owner-consuming update API.

## 検証

Add tests that perform several borrowed sum queries on one Fenwick value and then free it without resource diagnostics.

確認済み:

- `node nodesrc/tests.js -i stdlib/tests/fenwick.n.md --no-tree -o tmp/fenwick-stdlib-borrowed-queries.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/fenwick_collections.n.md --no-tree -o tmp/fenwick-collections-borrowed-queries.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/alloc/collections/fenwick.nepl --no-tree -o tmp/fenwick-doctest-borrowed-queries.json -j 1` (`total=5`, `passed=5`, `failed=0`)
- `node nodesrc/test_stdlib_fenwick_borrowed_queries.js`: passed
- `node nodesrc/test_stdlib_fenwick_no_unsafe_unwraps.js`: passed
- `node nodesrc/issues.js check`: passed

## 修正内容

- `Fenwick.len` / `Fenwick.sum_prefix` / `Fenwick.sum_range` を `&Fenwick` receiver に変更し、query で owner を移動しない公開 API にした。
- 未使用だった by-value private check helpers を削除し、同じ owner-consuming read pattern が内部に残らないようにした。
- Fenwick doctest / `.n.md` tests を、同じ tree に複数回 borrowed query を呼び、その後 `free` する形に直した。
- `nodesrc/test_stdlib_fenwick_borrowed_queries.js` を追加し、by-value query signature と by-value test usage が戻らないよう source policy に登録した。

## 関連して追加した issue

- `ISS-20260430T031656331Z-FENWICK-ADD-ERROR-PATH-CONSUMES-OWNE-10D232BB`: `add` の範囲外 Err path が入力 owner を返さず cleanup もしない別設計問題。
