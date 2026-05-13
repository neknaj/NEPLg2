---
id: ISS-20260513T042454474Z-COLLECTION-DIAGNOSTIC-MODULES-RELY-O-F02EB391
title: "collection diagnostic modules rely on transitive StdErrorKind import"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "stdlib/alloc/collections/*/api/diagnostic.nepl"
---

# ISS-20260513T042454474Z-COLLECTION-DIAGNOSTIC-MODULES-RELY-O-F02EB391: collection diagnostic modules rely on transitive StdErrorKind import

## 概要

Collection diagnostic helper modules call diag_error with StdErrorKind variants but do not import core/result directly. After import visibility became an explicit typecheck authority, relying on alloc/diag/error transitive dependencies leaves StdErrorKind undefined and blocks the affected collection doctests before their own behavior can be tested.

## 対象

- `stdlib/alloc/collections/*/api/diagnostic.nepl`

## 根拠

- `stdlib/alloc/collections/bitset/api/diagnostic.nepl` は `StdErrorKind::CapacityExceeded` / `StdErrorKind::IndexOutOfBounds` を使うが、`StdErrorKind` を定義する `core/result` を直接 import していなかった。
- 同じ diagnostic helper 分割を持つ `adjacency_matrix` / `disjoint_set` / `fenwick` / `segment_tree` / `sparse_set` も同じ transitive import 前提になっていた。
- import visibility が typecheck authority になった現在、module が名前として使う型・enum は、その module 自身が直接 import する必要がある。

## 問題

Collection diagnostic helper modules call diag_error with StdErrorKind variants but do not import core/result directly. After import visibility became an explicit typecheck authority, relying on alloc/diag/error transitive dependencies leaves StdErrorKind undefined and blocks the affected collection doctests before their own behavior can be tested.

## 影響

BitSet, adjacency matrix, disjoint set, fenwick, segment tree, and sparse set diagnostic constructors can fail during compile, hiding actual collection behavior and weakening the stdlib module boundary discipline.

## 修正方針

Add explicit core/result imports to each diagnostic helper module that names StdErrorKind, so every module owns the dependencies it uses directly.

## 検証

Run focused BitSet doctests and issue metadata checks.

## 2026-05-13 修正

対象の collection diagnostic helper 6 件へ `#import "core/result" as *` を追加し、`Diag` constructor module が `StdErrorKind` の定義元を直接参照するようにした。

これは `alloc/diag/error` の内部実装が `core/result` を使っていることに便乗するのではなく、各 module が自身の public helper signature/body で使う enum 依存を明示する修正である。import visibility 強化後の module boundary と一致し、将来 `alloc/diag/error` の re-export 構造を変えても collection diagnostics が壊れない。

検証:

- `node nodesrc\run_doctest.js -i stdlib\tests\bitset.n.md -n 1`: pass
