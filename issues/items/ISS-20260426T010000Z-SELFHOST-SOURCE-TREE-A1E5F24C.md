---
id: ISS-20260426T010000Z-SELFHOST-SOURCE-TREE-A1E5F24C
title: "NEPLg2.0 self-host compiler source tree is missing"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: stdlib/neplg2
source: doc/neplg2/self_host_plan.md
---

# ISS-20260426T010000Z-SELFHOST-SOURCE-TREE-A1E5F24C: NEPLg2.0 self-host compiler source tree is missing

## 概要

NEPLg2.0 のセルフホスト用ソースツリーが存在しない。
NEPLg3 用 placeholder は `stdlib/neplg3/` にあるが、現行 NEPLg2.0 構文で現行コンパイラを再実装する場所は別に必要である。

## 対象

- `stdlib/neplg2/`
- `doc/neplg2/self_host_plan.md`

## 根拠

- `doc/neplg3/impl/compiler_structure.md` は NEPLg3 の並行実装構成であり、NEPLg2.0 の syntax / type annotation / import 仕様とは異なる。
- 現行セルフホスト作業では NEPLg2.0 の `#import`、angle bracket 型注釈、現行 `typecheck.rs` 相当の処理を NEPLg2.0 コードで段階的に写す必要がある。

## 問題

`stdlib/neplg3/` を NEPLg2.0 セルフホストに流用すると、NEPLg3 設計と NEPLg2.0 実装目標が混ざる。
セルフホストの検証対象、doctest、bootstrap artifact、将来の NEPLg3 作業範囲を分離できない。

## 影響

NEPLg2.0 self-host と NEPLg3 compiler design の責務が混ざり、どちらの仕様に従うべきか判断できないファイルが発生する。
また、テスト失敗時に current compiler の回帰なのか NEPLg3 計画の未実装なのか切り分けられない。

## 修正方針

`stdlib/neplg2/` を NEPLg2.0 self-host compiler の正規ソースツリーとして再作成する。
構成は `doc/neplg3/impl/compiler_structure.md` の stage 分割を参考にしつつ、NEPLg2.0 の現行構文と現行 pipeline に合わせて `infra/`、`syntax/`、`module/`、`resolve/`、`ty/`、`check/`、`hir/`、`resource/`、`mono/`、`codegen/`、`builtins/`、`pipeline.nepl`、`cli/` に分ける。

## 検証

- `node nodesrc/tests.js -i stdlib/neplg2 --no-tree -o tmp/neplg2-selfhost-placeholder.json -j 2`
- `stdlib/neplg2/core/syntax/lexer.nepl` などの最初の実行可能 doctest を追加して green を確認する。
