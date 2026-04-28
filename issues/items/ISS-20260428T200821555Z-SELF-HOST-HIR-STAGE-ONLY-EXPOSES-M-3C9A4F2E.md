---
id: ISS-20260428T200821555Z-SELF-HOST-HIR-STAGE-ONLY-EXPOSES-M-3C9A4F2E
title: "self-host HIR stage only exposes marker API"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/neplg2/core/hir/hir.nepl"
---

# ISS-20260428T200821555Z-SELF-HOST-HIR-STAGE-ONLY-EXPOSES-M-3C9A4F2E: self-host HIR stage only exposes marker API

## 概要

`stdlib/neplg2/core/hir/hir.nepl` は Stage 0 marker API だけを公開しており、S3 type/check から S4 HIR lowering へ値を渡すための function / expression table がありません。

## 対象

- `stdlib/neplg2/core/hir/hir.nepl`

## 根拠

- `selfhost_hir_stage0` は `0` を返す marker のみ。
- `doc/neplg2/self_host_plan.md` は S4 で HIR、move / borrow / drop、monomorphize を現行 Rust 実装と同じ判定へ進める計画だが、HIR module root と stable id がまだない。

## 問題

後続の checker / lowering / backend が HIR を共有する境界を持てないため、実装を始めるたびに AST や type arena を直接覗く形になりやすい。これは self-host compiler の pass 境界を曖昧にし、deep traversal を explicit stack へ移す計画にもつながらない。

## 影響

S3 の型 stage で作った `SelfhostTypeId` を、S4 の HIR function / expression root と結びつけられません。self-host compiler の進捗が marker API から先へ進まず、`RV-STDLIB-008` の umbrella issue が解消できません。

## 修正方針

最小の HIR arena model を追加しました。`SelfhostHirModule` が function / param / expr / expr child table を所有し、`SelfhostHirFunctionId` と `SelfhostHirExprId` を stable id として返します。初期実装では unit / literal / var / call / block / if などの式種別と function root を保持し、後続 lowering が table を拡張できる境界を作りました。

## 検証

- `node nodesrc\tests.js -i stdlib\neplg2\core\hir\hir.nepl --no-tree -o tmp\selfhost-hir-minimal-model.json -j 1`: total=1 passed=1
