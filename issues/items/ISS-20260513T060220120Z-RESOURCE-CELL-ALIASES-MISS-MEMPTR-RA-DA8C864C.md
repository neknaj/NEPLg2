---
id: ISS-20260513T060220120Z-RESOURCE-CELL-ALIASES-MISS-MEMPTR-RA-DA8C864C
title: "Resource cell aliases miss MemPtr raw identity through aggregates and returns"
area: core
status: open
resolved: false
priority: P0
type: bug
created: 2026-05-13
updated: 2026-05-13
target: nepl-core/src/resource
---

# ISS-20260513T060220120Z-RESOURCE-CELL-ALIASES-MISS-MEMPTR-RA-DA8C864C: Resource cell aliases miss MemPtr raw identity through aggregates and returns

## 概要

tests/compiler/move_effect.n.md の MemPtr alias / aggregate field / Result payload / function return を経由するケースで、non-Copy raw load の二重所有値生成が
resource.cell.moved として検出されずコンパイル成功している。raw address alias は存在するはずだが cell state の canonicalization / summary / match bind / aggregate field propagation のどこかで失われている。

## 対象

- `nepl-core/src/resource`

## 根拠

- 未記入

## 問題

tests/compiler/move_effect.n.md の MemPtr alias / aggregate field / Result payload / function return を経由するケースで、non-Copy raw load の二重所有値生成が
resource.cell.moved として検出されずコンパイル成功している。raw address alias は存在するはずだが cell state の canonicalization / summary / match bind / aggregate field propagation のどこかで失われている。

## 影響

Resource IR の memory safety 検査が、同じ raw cell から non-Copy value を複数回 move するケースを一部見逃す。型安全・メモリ安全の必達条件に直接関わる。

## 修正方針

MemPtr raw field、aggregate field、enum payload、function return summary、branch/match merge の raw address alias 伝播を監査し、cell state が同一 raw cell を同一 place として扱えるように根本修正する。特定テストを列挙して通すのではなく、RawCellAddressAliases と CellTable の責務境界を保った設計にする。

## 検証

tests/compiler/move_effect.n.md の doctest#22,#23,#44-#55 が期待通り resource.cell.moved で失敗し、関連 Resource IR 単体テストを追加する。
