---
id: ISS-20260427T011048496Z-HIR-FUNCTION-REFERENCES-STILL-LACK-S-5EF51F12
title: "HIR function references still lack stable DefId snapshots after import resolution integration"
area: core
status: verified
resolved: true
priority: P2
type: architecture
created: 2026-04-27
updated: 2026-04-27
target: "nepl-core/src/resolve.rs, nepl-core/src/hir.rs, nepl-core/src/typecheck.rs, nepl-core/tests/resolve.rs"
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

## 解決

2026-04-27 に HIR の user function call が backend symbol とは別に source declaration identity を保持するようにした。

- `DefId` を host-local 連番ではなく `file_id/start/end` の source-span based id に変更した。
- typecheck の `BindingKind::Func` に `def_id` を追加し、top-level / imported / nested / alias / constructor binding で declaration span から設定するようにした。
- `FuncRef::User` を `User(symbol, type_args, def_id)` に拡張し、overload 解決で選ばれた binding の DefId を HIR に保存するようにした。
- monomorphize / codegen / move_check / drop_insertion など、HIR user call を読む後段 pass は symbol と type args の既存動作を維持したまま新しい field を扱うようにした。
- `nepl-core/tests/resolve.rs` に qualified import の DefId、open import shadowing 時の local DefId、ambiguous open import 内の同名定義が別 DefId になることを固定する回帰テストを追加した。

検証:

- `cargo test -p nepl-core --test resolve -- --nocapture`: 16 passed
- `cargo test -p nepl-core --test import_clause -- --nocapture`: 10 passed
- `cargo test -p nepl-core --target wasm32-unknown-unknown --no-run --all-features`: pass
