---
id: ISS-20260426T161201078Z-NEPLG2-SELFHOST-LACKS-EXECUTABLE-SPA-52B20497
title: "neplg2 selfhost lacks executable span token lexer foundation"
area: stdlib
status: verified
resolved: true
priority: P2
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: "stdlib/neplg2/core/infra/span.nepl, stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib/neplg2_lexer.n.md"
---

# ISS-20260426T161201078Z-NEPLG2-SELFHOST-LACKS-EXECUTABLE-SPA-52B20497: neplg2 selfhost lacks executable span token lexer foundation

## 概要

doc/neplg2/self_host_plan.md defines S1 as SourceMap / lexer / parser parity, but stdlib/neplg2 currently keeps span and token as Stage 0 marker APIs and has no executable lexer foundation. The broad legacy RV-STDLIB-008 target still references neplg3 and is not a concrete NEPLg2 S1 work unit.

## 対象

- `stdlib/neplg2/core/infra/span.nepl, stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/lexer.nepl, tests/stdlib/neplg2_lexer.n.md`

## 根拠

- 未記入

## 問題

doc/neplg2/self_host_plan.md defines S1 as SourceMap / lexer / parser parity, but stdlib/neplg2 currently keeps span and token as Stage 0 marker APIs and has no executable lexer foundation. The broad legacy RV-STDLIB-008 target still references neplg3 and is not a concrete NEPLg2 S1 work unit.

## 影響

Self-host parser work cannot compare token streams with the Rust compiler, and later source-map diagnostics would be built on ad hoc byte offsets instead of a shared stdlib/neplg2 core model.

## 修正方針

Implement a small copyable SourceSpan and Token/TokenKind model, add a NEPLg2-oriented byte lexer for whitespace, comments, identifiers, integer literals, string literals, punctuation, EOF, and lexical diagnostics, and add focused doctests.

## 検証

Run focused NEPLg2 lexer tests, stdlib/neplg2 tests, issue check, and diff check.

## 解決

- `stdlib/neplg2/core/infra/span.nepl` の Stage 0 marker を、byte offset ベースの `SelfhostSourceSpan` と O(1) helper に置き換えた。
- `stdlib/neplg2/core/syntax/token.nepl` に `TokenKind` / `SelfhostToken` と EOF / error / expression-start 判定を実装した。
- `stdlib/neplg2/core/syntax/lexer.nepl` を追加し、horizontal whitespace、`//` comment、newline、identifier、integer literal、string literal、NEPLg2.0 の主要 punctuation、EOF、lexical diagnostic を扱う byte lexer を実装した。
- `tests/stdlib/neplg2_lexer.n.md` に directive/function signature/int literal、comment skip + unexpected char、unterminated string の回帰テストを追加した。
- `stdlib/neplg2/README.md` と `todo.md` を現状に合わせて更新した。

## 検証結果

- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-lexer-foundation-focused.json -j 1`: `total=25`, `passed=25`
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-neplg2-lexer-foundation-full.json -j 4`: `total=411`, `passed=411`
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-neplg2-lexer-foundation-full.json -j 4`: `total=277`, `passed=277`
- `trunk build`: pass
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-neplg2-lexer-foundation.json`: `13/13 passed`
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
