---
id: ISS-20260428T202624698Z-SELF-HOST-HIR-ARENA-LACKS-FUNCTION-P-3C47EC35
title: "self-host HIR arena lacks function param range API"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: stdlib/neplg2/core/hir/hir.nepl
---

# ISS-20260428T202624698Z-SELF-HOST-HIR-ARENA-LACKS-FUNCTION-P-3C47EC35: self-host HIR arena lacks function param range API

## 概要

SelfhostHirFunction has first_param / param_count fields and the module owns a params table, but there is no public API to copy parameter records into a contiguous table range or read a parameter by range/index. Function lowering would need to construct raw table offsets by hand.

## 対象

- `stdlib/neplg2/core/hir/hir.nepl`

## 根拠

- `SelfhostHirFunction` は `first_param` / `param_count` を持つ。
- `SelfhostHirModule` は `params` table を所有する。
- 修正前は `params` へ parameter record 列を追加する API と、range から parameter を読む API がなかった。

## 問題

SelfhostHirFunction has first_param / param_count fields and the module owns a params table, but there is no public API to copy parameter records into a contiguous table range or read a parameter by range/index. Function lowering would need to construct raw table offsets by hand.

## 影響

S4 HIR lowering cannot construct functions with typed parameter ranges through the arena boundary. This leaves function signatures only partially modeled and keeps RV-STDLIB-008 blocked for non-trivial function bodies.

## 修正方針

`SelfhostHirParamRange` と `SelfhostHirModuleParamRangeAlloc` を追加し、parameter record 列を module の `params` table へコピーして typed range を返す API を追加しました。

あわせて、parameter range から function record を作る `selfhost_hir_function_with_params`、function record から parameter range を取り出す `selfhost_hir_function_param_range`、range + index を bounds check して parameter record を返す `selfhost_hir_module_get_param` を追加しました。これにより、後続 lowering は raw offset を直接組み立てずに parameter 付き function を作れます。

## 検証

- `node nodesrc\tests.js -i stdlib\neplg2\core\hir\hir.nepl --no-tree -o tmp\selfhost-hir-param-ranges.json -j 1`: total=3 passed=3
