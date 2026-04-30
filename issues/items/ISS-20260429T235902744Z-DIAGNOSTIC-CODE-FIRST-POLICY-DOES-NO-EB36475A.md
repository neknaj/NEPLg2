---
id: ISS-20260429T235902744Z-DIAGNOSTIC-CODE-FIRST-POLICY-DOES-NO-EB36475A
title: "Diagnostic code-first policy does not scan active Rust sources"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-29
updated: 2026-04-30
target: "nodesrc/test_diagnostic_code_first_boundary.js, nepl-core/src/**/*.rs, nepl-language/src/**/*.rs, nepl-lsp/src/**/*.rs, nepl-web/src/**/*.rs"
---

# ISS-20260429T235902744Z-DIAGNOSTIC-CODE-FIRST-POLICY-DOES-NO-EB36475A: Diagnostic code-first policy does not scan active Rust sources

## 概要

The diagnostic source policy only checked a small boundary file list for post-construction .with_code usage. It did not scan all active Rust sources for code-less Diagnostic::error / Diagnostic::warning calls, so compiler pass regressions could reintroduce unclassified diagnostics without failing CI.

## 対象

- `nodesrc/test_diagnostic_code_first_boundary.js, nepl-core/src/**/*.rs, nepl-language/src/**/*.rs, nepl-lsp/src/**/*.rs, nepl-web/src/**/*.rs`

## 根拠

- `nodesrc/test_diagnostic_code_first_boundary.js` は `nepl-core/src/diagnostic.rs`、`nepl-language/src/lib.rs`、`nepl-lsp/src/main.rs`、`nepl-web/src/lib.rs` だけを読み、`.with_code(...)` と `fn with_code` を中心に検査していた。
- `rg -n "Diagnostic::error\(|Diagnostic::warning\(|\.with_code\(|fn\s+with_code\b" nepl-core nepl-language nepl-lsp nepl-web -g "*.rs"` では現時点の残件は `nepl-core/src/diagnostic.rs` の unit test だけだったが、CI policy は他の active Rust source に code-less diagnostic call が再導入されても検出できなかった。
- 診断再設計 Stage D1 は、active compiler pass の診断生成を `DiagnosticCode` enum first にすることを完了条件にしているため、監視対象も active Rust source tree 全体でなければならない。

## 問題

The diagnostic source policy only checked a small boundary file list for post-construction .with_code usage. It did not scan all active Rust sources for code-less Diagnostic::error / Diagnostic::warning calls, so compiler pass regressions could reintroduce unclassified diagnostics without failing CI.

## 影響

The diagnostics redesign requires enum code-first construction for type, effect, owner, lifetime, borrow and backend diagnostics. A narrow policy allows future regressions to bypass Rust enum exhaustiveness and stable diagnostic code contracts.

## 修正方針

Expand the source policy to recursively scan active Rust src trees, reject .with_code and fn with_code everywhere, and reject code-less Diagnostic::error / Diagnostic::warning outside the diagnostic module unit-test area.

## 検証

Run the updated source policy, targeted diagnostic tests, issue validation and diff checks.

## 対応結果

2026-04-30 に source policy を再設計し、固定ファイルリストではなく次の active Rust source root を再帰走査するようにした。

- `nepl-core/src`
- `nepl-language/src`
- `nepl-lsp/src`
- `nepl-web/src`

この policy は全対象ファイルで `.with_code(...)` と `fn with_code` を禁止する。さらに `Diagnostic::error(...)` / `Diagnostic::warning(...)` の code-less constructor 呼び出しを禁止し、`nepl-core/src/diagnostic.rs` の `#[cfg(test)]` unit test 内だけを例外にした。

これにより、診断 code を後付けする回帰だけでなく、active compiler pass が enum code を持たない診断を作る回帰も CI の source policy で検出できる。

検証:

- `node nodesrc/test_diagnostic_code_first_boundary.js`
