---
id: ISS-20260520T040120122Z-SELF-HOST-HIR-REMAINS-A-FLAT-IMPLEME-C9744663
title: "self-host HIR remains a flat implementation file"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/hir/hir.nepl, stdlib/neplg2/core/hir/**, doc/neplg2/self_host_source_tree_layout_review_20260518.md"
---

# ISS-20260520T040120122Z-SELF-HOST-HIR-REMAINS-A-FLAT-IMPLEME-C9744663: self-host HIR remains a flat implementation file

## 概要

Self-host HIR still keeps id model, range payloads, expression payloads, function records, module arena ownership, allocation result accessors, table copy/get operations, and stage smoke API in one large file. This contradicts the source tree plan and makes later lowering/static-check work easy to append to the flat facade.

## 対象

- `stdlib/neplg2/core/hir/hir.nepl, stdlib/neplg2/core/hir/**, doc/neplg2/self_host_source_tree_layout_review_20260518.md`

## 根拠

- `doc/neplg2/self_host_source_tree_layout_review_20260518.md` は `core/hir/hir.nepl` を P2 の分割対象として挙げ、`hir/` 配下に id / expr / function / module / arena / range / lower を分ける方針を明記していた。
- 変更前の `core/hir/hir.nepl` は HIR id、range payload、expression payload、function model、module arena、allocation result accessor、table copy/get、stage smoke API を 1 file に持っていた。
- この状態で lowering や ResourceIR-facing HIR behavior を追加すると、Rust 側の flat HIR/typecheck 周辺構造を self-host 側へ再導入する危険があった。

## 問題

Self-host HIR still keeps id model, range payloads, expression payloads, function records, module arena ownership, allocation result accessors, table copy/get operations, and stage smoke API in one large file. This contradicts the source tree plan and makes later lowering/static-check work easy to append to the flat facade.

## 影響

Adding lowering or ResourceIR-facing HIR behavior into the same file would copy the Rust compiler flat structure, weaken responsibility review, and make enum/match source policies harder to target precisely.

## 修正方針

Keep hir.nepl as an implementation-free public facade and split HIR into id/range/expr/function/module/arena/stage0 modules. Preserve typed absence, payload enum matches, owner-safe allocation accessors, doctests, and source policy checks.

## 検証

Run focused HIR source policy checks, HIR doctests with --assert-io, issues check, and diff whitespace check.

## 対応結果

- `core/hir/hir.nepl` は doctest を保持する implementation-free facade にした。
- 実装は `core/hir/hir/id.nepl`、`range.nepl`、`expr.nepl`、`function.nepl`、`module.nepl`、`arena.nepl`、`stage0.nepl` へ分割した。
- `SelfhostHirExprPayload` / `SelfhostHirChildRange` / `SelfhostHirParamRange` の enum payload、typed absence、owner-safe allocation accessor は維持した。
- `nodesrc/selfhost_hir_sources.js` を追加し、既存の HIR source policy が facade と split files をまとめて監視できるようにした。
- `nodesrc/test_selfhost_hir_split_contract.js` を追加し、facade への実装再導入、split file の 450 行超過、submodule から facade への曖昧 import を拒否する。

## 検証結果

- `node nodesrc/test_selfhost_hir_split_contract.js`: passed
- `node nodesrc/test_selfhost_hir_report_contract.js`: passed
- `node nodesrc/test_selfhost_hir_range_payload.js`: passed
- `node nodesrc/test_selfhost_hir_expr_payload.js`: passed
- `node nodesrc/test_selfhost_hir_expr_id_absence.js`: passed
- `node nodesrc/test_selfhost_model_no_numeric_kind_tags.js`: passed
- `node nodesrc/tests.js -i stdlib/neplg2/core/hir/hir.nepl --no-tree -o tmp/agent1-hir-split-core.json -j 1 --dist web/dist --assert-io`: 3/3 passed
