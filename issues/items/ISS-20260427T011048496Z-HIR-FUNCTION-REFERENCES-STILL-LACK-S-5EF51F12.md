---
id: ISS-20260427T011048496Z-HIR-FUNCTION-REFERENCES-STILL-LACK-S-5EF51F12
title: "HIR function references still lack stable DefId snapshots after import resolution integration"
area: core
status: open
resolved: false
priority: P2
type: architecture
created: 2026-04-27
updated: 2026-04-27
target: "nepl-core/src/resolve.rs, nepl-core/src/hir.rs, nepl-core/src/typecheck.rs"
---

# ISS-20260427T011048496Z-HIR-FUNCTION-REFERENCES-STILL-LACK-S-5EF51F12: HIR function references still lack stable DefId snapshots after import resolution integration

## 概要

RV-CORE-010 で SourceMap ベースの import visibility は `resolve.rs` に統合し、未使用の `name_resolve.rs` stub も削除した。一方で、現行 flat-loader main pipeline はまだ HIR に `FuncRef::User(symbol, type_args)` と文字列名の local reference を出力しており、resolver 由来の stable DefId を HIR snapshot として保持していない。

## 対象

- `nepl-core/src/resolve.rs, nepl-core/src/hir.rs, nepl-core/src/typecheck.rs`

## 根拠

- `nepl-core/src/hir.rs`: `FuncRef::User` は backend symbol 文字列と type arguments だけを持つ。
- `nepl-core/src/typecheck.rs`: overload 解決後の候補は `Binding` から symbol 文字列へ落とされ、HIR へ declaration id が渡らない。
- `nepl-core/src/resolve.rs`: `ImportResolution` は import visibility を提供するが、HIR node へ DefId を付与する段階はまだ存在しない。

## 問題

SourceMap ベースの import visibility は resolver API に統合済みだが、型検査後の HIR が「どの宣言に解決されたか」を stable id として保持していない。backend symbol は codegen 用の名前であり、module boundary / LSP / snapshot の canonical identity としては不十分。

## 影響

将来の module graph / non-flat loader 統合、LSP reference snapshot、同名 cross-module diagnostic が HIR 単体から target-independent な declaration id を参照できない。

## 修正方針

SourceMap-backed declaration 用の target-independent DefId を導入し、backend symbol 名を保持したまま typecheck `Binding` と HIR function reference に付与する。resolver snapshot test は cross-file import、qualified alias、shadowing、ambiguous open import の id を検証する。

## 検証

cross-file import、qualified alias、shadowing、ambiguous open import の DefId snapshot test を追加し、`nepl-core` の `resolve` / `import_clause` test が引き続き通ることを確認する。
