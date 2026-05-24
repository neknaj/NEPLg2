---
id: ISS-20260524T085928069Z-NEPLG2-1-SYNTAX-MIGRATION-NEEDS-FRON-7058CE30
title: "NEPLg2.1 syntax migration needs frontend lowering boundary"
area: core
status: open
resolved: false
priority: P0
type: architecture
created: 2026-05-24
updated: 2026-05-24
target: "nepl-core/src/lexer.rs, nepl-core/src/parser.rs, nepl-core/src/typecheck/**"
---

# ISS-20260524T085928069Z-NEPLG2-1-SYNTAX-MIGRATION-NEEDS-FRON-7058CE30: NEPLg2.1 syntax migration needs frontend lowering boundary

## 概要

NEPLg2.0 angle-bracket type annotations, parenthesized lambdas, and explicit generic call postfixes are still accepted as the main surface syntax. NEPLg2.1 must introduce percent type annotations, prefix type expressions, and backslash lambdas while lowering to the existing typed HIR.

## 対象

- `nepl-core/src/lexer.rs, nepl-core/src/parser.rs, nepl-core/src/typecheck/**`

## 根拠

- 現在の言語対象は NEPLg2 であり、この変更により NEPLg2.1 へ切り替える。NEPLg3 は未着手・未確定であり、`doc/neplg3/` は参考に留める。
- 設計計画: [NEPLg2.1 surface syntax migration plan](../../doc/neplg2/neplg21_syntax_migration_plan.md)
- frontend 調査では、`%` 型注釈と `\` lambda は既存 `PrefixItem::TypeAnnotation` / private `FnDef` desugar へ正規化できると確認した。
- prefix 型式の境界は parser 単独では難しいため、表層構文を後続 phase へ漏らさない typed lowering boundary が必要である。
- 明示 generic postfix `f<T>` の撤廃は単純な lexer/parser 変更ではなく、期待型・引数型・trait bound を使う resolver/typecheck 側の責務を含む。

## 問題

NEPLg2.0 angle-bracket type annotations, parenthesized lambdas, and explicit generic call postfixes are still accepted as the main surface syntax. NEPLg2.1 must introduce percent type annotations, prefix type expressions, and backslash lambdas while lowering to the existing typed HIR.

## 影響

Without a frontend-owned lowering boundary the syntax migration can leak into Resource IR or selfhost planning, and NEPLg2.1 can be confused with unstable NEPLg3 documents.

## 修正方針

Add NEPLg2.1 lexer/parser/type frontend support that normalizes new surface syntax into existing TypeExpr/FnDef/PrefixItem structures, rejects or quarantines explicit generic call postfixes, and keeps Resource IR unchanged.

### 実装境界

- `TokenKind::Percent` と `TokenKind::Backslash` を追加する。
- `%TypeExpr` は既存 `PrefixItem::TypeAnnotation(TypeExpr, Span)` へ落とす。
- `\a\b:` / `\():` は既存の private `FnDef` + value expr block へ desugar する。
- `fn A fn B C` は表層表記として受け取り、NEPLg2.1 では既存の複数引数関数型へ正規化する。部分適用は導入しない。
- 関数を返す関数型は `%fn A (fn B C)` のように戻り値側を grouping し、grouping された関数型は flatten しない。
- `impure fn A B` を NEPLg2.1 の副作用関数型表記として扱う。
- 通常 source の `Ident<...>` 明示 type args は移行診断または拒否へ寄せる。ただし compiler-owned intrinsic/internal path の型引数処理は source syntax と分ける。
- Resource IR / ownership / borrow / codegen には NEPLg2.1 専用 syntax node を追加しない。

### 2026-05-24 checkpoint

- branch: `feature/neplg21-syntax-migration-20260524`
- doc: `doc/neplg2/neplg21_syntax_migration_plan.md`
- frontend implementation:
  - Rust lexer/parser は `%` 型注釈、prefix 型式、`\` lambda 引数、`#extern` の `%...` signature を受け付ける。
  - `fn A fn B C` / `impure fn A B` は既存 `TypeKind::Function` へ正規化し、部分適用は導入していない。
  - `%fn A (fn B C)` は関数を返す関数型として扱い、複数引数関数型への flatten から除外する。
  - selfhost lexer/token model にも `Percent` / `Backslash` token を追加した。
  - overload candidate pruning は `Option<.T>: Copy` のような generic impl pattern を候補段階で認識し、`Stack<.T>.items` 経由の `vec::free` を落とさない。
- remaining:
  - prefix 型適用境界はまだ parser-local arity hints に依存している。kind resolver 化は `ISS-20260524T193635695Z-NEPLG2-1-PREFIX-TYPE-APPS-NEED-KIND-RESOLVER-A13F0C92` で追跡する。
  - 通常 source の explicit generic postfix は互換受理が残っており、撤廃は corpus semantic rewrite issue 側で継続する。
- 関連 issue:
  - `ISS-20260524T085928138Z-NEPLG2-1-CORPUS-MIGRATION-NEEDS-SEMA-42A21754`
  - `ISS-20260524T085928137Z-README-AND-DOCS-MUST-DISTINGUISH-NEP-20719BBC`
  - `ISS-20260524T193635695Z-NEPLG2-1-PREFIX-TYPE-APPS-NEED-KIND-RESOLVER-A13F0C92`

## 検証

Focused parser/typecheck tests for percent annotations, prefix function types, lambda syntax, no partial application, and generic inference without call postfixes.
