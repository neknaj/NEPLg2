---
id: ISS-20260512T203408880Z-LANGUAGE-AND-LSP-DIAGNOSTICS-DROP-RE-8AFBCCD3
title: "Language and LSP diagnostics drop registry code messages"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-13
target: "nepl-language/src/lib.rs; nepl-lsp/src/main.rs; nodesrc/test_diagnostic_code_first_boundary.js; doc/neplg2/compiler_diagnostics_redesign_plan.md"
---

# ISS-20260512T203408880Z-LANGUAGE-AND-LSP-DIAGNOSTICS-DROP-RE-8AFBCCD3: Language and LSP diagnostics drop registry code messages

## 概要

Stage D3 requires human-facing display text and machine-facing stable codes to be derived from the same DiagnosticCode value. nepl-web exposes both code and code_message, but nepl-language EditorDiagnostic only exposes code and message, and nepl-lsp forwards only code plus notes/helps. The registry message is therefore lost at the Rust editor/LSP boundary.

## 対象

- `nepl-language/src/lib.rs; nepl-lsp/src/main.rs; nodesrc/test_diagnostic_code_first_boundary.js; doc/neplg2/compiler_diagnostics_redesign_plan.md`

## 根拠

- `nepl-web/src/lib.rs` は diagnostic object に `code` と `code_message` を出しており、web editor bridge も `codeMessage` として保持する。
- `nepl-language/src/lib.rs` の `EditorDiagnostic` は `code` と contextual `message` だけを保持していたため、`DiagnosticCode::message()` 由来の canonical message が editor/LSP 境界で失われていた。
- `nepl-lsp/src/main.rs` の publishDiagnostics 変換は `data.notes` / `data.helps` は転送する一方、`code_message` を転送していなかった。

## 問題

Stage D3 requires human-facing display text and machine-facing stable codes to be derived from the same DiagnosticCode value. nepl-web exposes both code and code_message, but nepl-language EditorDiagnostic only exposes code and message, and nepl-lsp forwards only code plus notes/helps. The registry message is therefore lost at the Rust editor/LSP boundary.

## 影響

Editors using nepl-language or nepl-lsp cannot show the canonical enum-derived diagnostic description separately from contextual message text, and parity with web diagnostics can regress without source-policy coverage.

## 修正方針

Add code_message to EditorDiagnostic, populate it from DiagnosticCode::message(), forward it through LSP diagnostic data, and extend the diagnostic source policy plus focused tests to require this contract.

## 検証

node nodesrc/test_diagnostic_code_first_boundary.js; cargo test -p nepl-language target_directive_diagnostics_keep_loader_codes; cargo check -p nepl-language -p nepl-lsp --tests; node nodesrc/issues.js check --dir issues

## 2026-05-13 修正

`nepl-language` / `nepl-lsp` の diagnostic D3 contract を web 側と揃えた。

- `EditorDiagnostic` に `code_message: &'static str` を追加した。
- `diagnostics_to_editor` は `diagnostic.code.message()` から `code_message` を生成する。
- `nepl-lsp` は publishDiagnostics の `data.code_message` に同じ値を転送する。
- `target_directive_diagnostics_keep_loader_codes` は stable code だけでなく enum-derived message も確認する。
- `nodesrc/test_diagnostic_code_first_boundary.js` は language / LSP 側の `code_message` contract を監視する。

これにより、Rust editor/LSP 境界でも stable code と canonical message が同じ `DiagnosticCode` から生成される。
