---
id: ISS-20260507T094645212Z-DIAGNOSTIC-VALUE-STILL-PERMITS-CODEL-3654AAD9
title: "Diagnostic value still permits codeless diagnostics"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/diagnostic.rs, nepl-core/src/diagnostic_codes.rs, nodesrc/test_diagnostic_code_first_boundary.js"
---

# ISS-20260507T094645212Z-DIAGNOSTIC-VALUE-STILL-PERMITS-CODEL-3654AAD9: Diagnostic value still permits codeless diagnostics

## 概要

Diagnostic stores its stable code as Option<DiagnosticCode> and still exposes code-less error/warning constructors. Even though source policy keeps active call sites code-first today, the type itself still permits diagnostics without compiler-owned enum codes.

## 対象

- `nepl-core/src/diagnostic.rs, nepl-core/src/diagnostic_codes.rs, nodesrc/test_diagnostic_code_first_boundary.js`

## 根拠

- `nepl-core/src/diagnostic.rs` の `Diagnostic.code` が `Option<DiagnosticCode>` だったため、diagnostic value が code を持たない状態を型として許していた。
- 同 file には `Diagnostic::error(...)` / `Diagnostic::warning(...)` が残り、active call site policy をすり抜ければ code-less diagnostic を再導入できる API になっていた。
- `doc/neplg2/compiler_diagnostics_redesign_plan.md` は内部識別子を階層 enum にする方針だが、`Diagnostic` value 自体はまだ必須 enum を要求していなかった。

## 問題

Diagnostic stores its stable code as Option<DiagnosticCode> and still exposes code-less error/warning constructors. Even though source policy keeps active call sites code-first today, the type itself still permits diagnostics without compiler-owned enum codes.

## 影響

A future compiler pass, language boundary, or web boundary can reintroduce code-less diagnostics without a Rust type error. This weakens the diagnostic redesign goal and makes Resource IR/static-check regressions depend on policy text instead of enum-first API design.

## 修正方針

Make Diagnostic.code mandatory, remove public code-less constructors, build all diagnostics through DiagnosticSpec or code-first constructors, and update external serialization to emit stable code unconditionally.

## 検証

cargo test -p nepl-core diagnostic -- --nocapture; cargo check -p nepl-core -p nepl-language -p nepl-lsp --tests; cargo check --manifest-path nepl-web/Cargo.toml; node nodesrc/test_diagnostic_code_first_boundary.js; node nodesrc/issues.js check

## 2026-05-07 対応結果

`Diagnostic.code` を `DiagnosticCode` 必須 field に変更し、`Option<DiagnosticCode>` を削除した。code-less な `Diagnostic::error(...)` / `Diagnostic::warning(...)` constructor も削除し、`DiagnosticSpec` / `error_with_code` / `warning_with_code` など code-first constructor 経由でしか診断を構築できない形にした。

`nepl-language` の `EditorDiagnostic.code` も必須 stable string に変更し、`nepl-web` の wasm analysis serialization は `code` / `code_message` を常に出すようにした。既存 Rust tests は optional code 前提の比較をやめ、`DiagnosticCode` enum を直接比較する。

`nodesrc/test_diagnostic_code_first_boundary.js` は `Diagnostic.code` が必須 `DiagnosticCode` であること、`Option<DiagnosticCode>` が戻らないこと、code-less constructor 呼び出しが unit test 内を含めて戻らないことを確認する。

検証:

- `cargo test -p nepl-core diagnostic -- --nocapture`: passed
- `node nodesrc/test_diagnostic_code_first_boundary.js`: passed
- `cargo check -p nepl-core -p nepl-language -p nepl-lsp --tests`: passed
- `cargo check -p nepl-language -p nepl-lsp --tests`: passed
- `cargo check --manifest-path nepl-web/Cargo.toml`: passed
- `cargo fmt --check`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
