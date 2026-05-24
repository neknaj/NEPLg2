---
id: ISS-20260524T085928138Z-NEPLG2-1-CORPUS-MIGRATION-NEEDS-SEMA-42A21754
title: "NEPLg2.1 corpus migration needs semantic generic rewrite"
area: stdlib
status: open
resolved: false
priority: P0
type: maintenance
created: 2026-05-24
updated: 2026-05-24
target: "stdlib/**, tests/**, tutorials/**, doc/examples/**"
---

# ISS-20260524T085928138Z-NEPLG2-1-CORPUS-MIGRATION-NEEDS-SEMA-42A21754: NEPLg2.1 corpus migration needs semantic generic rewrite

## 概要

Existing NEPLg2 source uses angle-bracket annotations, type applications, explicit generic postfix calls, and parenthesized lambda arguments throughout the stdlib and test corpus.

## 対象

- `stdlib/**, tests/**, tutorials/**, doc/examples/**`

## 根拠

- 現在の実行対象 corpus は `stdlib/`、`tests/`、`tutorials/`、`doc/examples/` に分散しており、角括弧型注釈、型適用、generic postfix call、parenthesized lambda が広く使われている。
- subagent 調査では、`tests/compiler/typeannot.n.md`、`tests/compiler/functions.n.md`、`tests/compiler/generics.n.md`、`stdlib/core/result.nepl`、`stdlib/alloc/collections/vec/**` が代表的な高密度領域として確認された。
- `tuple_old_syntax.n.md` や compile_fail fixture には、旧構文を失敗例として残すべき箇所がある。
- `stdlib/neplg2/` は selfhost compiler 実装側の構文処理を含むため、利用コードと同じ一括置換対象にはできない。
- 設計計画: [NEPLg2.1 surface syntax migration plan](../../doc/neplg2/neplg21_syntax_migration_plan.md)

## 問題

Existing NEPLg2 source uses angle-bracket annotations, type applications, explicit generic postfix calls, and parenthesized lambda arguments throughout the stdlib and test corpus.

## 影響

A textual rewrite can migrate simple annotations, but explicit generic postfix removal requires expected-type and signature-aware decisions, especially in stdlib callbacks and compile-fail fixtures.

## 修正方針

Build an inventory and migrate executable source to NEPLg2.1 syntax using AST/token-balanced tooling plus LLM review for generic call sites and lambda/function literal boundaries.

### 分類

自動変換しやすいもの:

- balanced token で範囲を取れる `<TypeExpr>` 型注釈。
- `Vec<i32>` / `Result<i32,str>` のような型式内 generic application。
- `fn name <signature> (args):` の外形変換。
- struct field / enum payload の型注釈。

LLM/手動判断が必要なもの:

- `some<i32>` / `unwrap_ok<T,E>` / `Result::Ok<T,E>` などの明示 generic call。
- 期待型が不足しているため `%T` 注釈の追加が必要な call。
- `let f (x):` や `apply 10 (x):` の function literal と旧 tuple fixture の区別。
- 関数を返す関数型を `%fn A (fn B C)` として grouping する必要がある箇所。
- owner-preserving callback signature、borrowed predicate、effect `*` が絡む stdlib API。
- selfhost parser/compiler 実装側の source string fixture。

### 2026-05-24 checkpoint

- branch: `feature/neplg21-syntax-migration-20260524`
- frontend 親 issue: `ISS-20260524T085928069Z-NEPLG2-1-SYNTAX-MIGRATION-NEEDS-FRON-7058CE30`
- doc 親 issue: `ISS-20260524T085928137Z-README-AND-DOCS-MUST-DISTINGUISH-NEP-20719BBC`
- mechanical migration:
  - `nodesrc/neplg21_syntax_migrate.js` を追加し、balanced `<TypeExpr>` 型注釈、function/lambda 引数外形、doc/test corpus の大半を `%` / `\` 構文へ移行した。
  - migrator は string literal 内を変換対象から外し、関数を返す旧関数型は戻り値側を grouping して変換する。
  - `#intrinsic "..." <...>` は compiler-owned directive syntax として保持し、`#extern` signature は `%...` を受理する。
  - `node nodesrc/neplg21_syntax_migrate.js --check` は idempotence check として通過している。
- remaining:
  - executable/comment/doc を合わせた generic postfix 形式はまだ多数残る。これは単純削除ではなく、呼び出し先 signature と expected type を見て `%T` 注釈を補う semantic rewrite として継続する。
  - selfhost source string fixture には旧構文の期待文字列が残る。これは selfhost parser の NEPLg2.1 対応と同じ単位で更新する。

## 検証

Run stdlib/source policy tests, trunk build, and nodesrc CLI JSON tests after migration.
