---
id: ISS-20260602T120007116Z-NEPLG2-1-ZERO-ARGUMENT-FUNCTIONS-SHO-DFF804DA
title: "NEPLg2.1 zero-argument functions should use void marker"
area: core
status: fixed
resolved: true
priority: P0
type: architecture
created: 2026-06-02
updated: 2026-06-02
target: "nepl-core/src/lexer.rs; nepl-core/src/parser.rs; nepl-core/src/parser/neplg21_type_expr.rs; nodesrc/neplg21_syntax_migrate.js; stdlib/**; tests/**; tutorials/**; doc/**"
source: "doc/neplg2/zero_arg_void_marker_spec.md"
---

# ISS-20260602T120007116Z-NEPLG2-1-ZERO-ARGUMENT-FUNCTIONS-SHO-DFF804DA: NEPLg2.1 zero-argument functions should use void marker

## 概要

NEPLg2.1 currently uses `unit` as the unit type, the unit value, and the zero-argument function marker. This makes `fn unit T` unable to distinguish a true zero-argument function from a function that accepts one `unit` value.

## 対象

- `nepl-core/src/lexer.rs; nepl-core/src/parser.rs; nepl-core/src/parser/neplg21_type_expr.rs; nodesrc/neplg21_syntax_migrate.js; stdlib/**; tests/**; tutorials/**; doc/**`

## 根拠

- `doc/neplg2/zero_arg_void_marker_spec.md` の仕様に従う。
- Zenn 開発方針は、試作段階でも暫定設計を残さず、仕様変更のたびに不整合を確認し、issue と計画で管理することを要求している。
- 既存の `ISS-20260526T004744719Z-NEPLG2-1-UNIT-LITERAL-SHOULD-USE-UNI-ECE4D70B` は `unit` を 0 引数 marker として扱う旧 checkpoint を固定しているため、新しい破壊的仕様変更として明示的に置き換える必要がある。
- `unit` 型の値を 1 個受け取る関数と、引数を本当に取らない関数を表層構文で区別できると、関数型の意味がより静的に検査しやすくなる。

## 問題

`fn unit T` が 0 引数関数型として正規化される現状では、`unit` 型の引数を 1 個取る関数型を自然に書けない。さらに `\unit` が marker と仮引数名のように見えるため、`unit` 型・unit 値・空引数 marker の責務が混ざっている。

## 影響

Frontend がこの多義性を維持すると、NEPLg2.1 の source syntax は `unit` 引数と 0 引数関数を明確に区別できない。selfhost 設計、高階関数、memo_call、stdlib API の型記述でも、関数 arity の契約が曖昧になる。

## 修正方針

Introduce `void` as a keyword-backed zero-argument function marker. Parse `fn void T` and `impure fn void T` as empty parameter lists, parse `\void` as a zero-argument lambda/function marker, and stop treating `fn unit T` or `\unit` as zero-argument forms.

`void` is not a type and not a value. Do not add `TypeExpr::Void`, HIR void values, Resource IR void values, or runtime void values. `unit` remains the unit type and unit value, and `fn unit T` becomes a one-argument function type whose parameter type is `unit`.

Update the NEPLg2.1 migrator so existing zero-argument marker usage is rewritten as `fn void` and `\void`, while `unit` values and `unit` return types are preserved.

## 検証

Add focused frontend tests for `fn void T`, `\void`, `fn unit T` with a named parameter, and invalid `void` type/value usages. Migrate corpus/docs and run the NEPLg2.1 migrator check, issue checker, focused compiler tests, whitespace checks, and the project-required build/test path appropriate to the touched files.

## 対応

- `void` を lexer の予約 keyword として追加し、frontend で `fn void T` / `\void` を既存の空 parameter list へ正規化した。
- `unit` は unit 型・unit 値として残し、`fn unit T` は `unit` 型の引数を 1 個取る関数型として parse するようにした。
- `void` は型式・値式・返り値型・型引数としては受理しない。`TypeExpr::Void`、HIR void、Resource IR void、runtime void value は追加していない。
- `nodesrc/neplg21_syntax_migrate.js` を更新し、旧 0 引数 marker の `fn unit` / `\unit` を `fn void` / `\void` へ変換し、正当な `unit` 引数関数は変換しないようにした。
- stdlib、examples、tutorials、tests、selfhost token model、source policy、README / doc を新 marker へ移行した。
- source policy の `byte_builder` owner 境界検査は、旧 `()` helper view ではなく現行 NEPLg2.1 構文の `void` / `unit` を直接検査する形へ更新した。

## 完了時検証

- `cargo test -p nepl-core --test functions function_neplg21 -- --nocapture`
- `cargo test -p nepl-core --test typeannot test_neplg21 -- --nocapture`
- `node nodesrc/test_neplg21_syntax_migrate.js`
- `node nodesrc/test_source_policy_nepl_source_view.js`
- `node nodesrc/test_stdlib_builder_owner_boundary.js`
- `node nodesrc/neplg21_syntax_migrate.js --check`
- `node nodesrc/run_source_policy_regressions.js --warn-only`

`run_source_policy_regressions.js --warn-only` は既存の global warning を 7 件出すが、今回触った syntax migration / source view / byte_builder owner boundary の regression は通過している。
