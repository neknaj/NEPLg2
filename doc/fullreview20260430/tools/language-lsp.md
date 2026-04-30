# Tools Review: Language Library And LSP

対象 commit: `f108cebd`

## 対象

- `nepl-language/src/lib.rs`
- `nepl-lsp/src/main.rs`

## 概要

`nepl-language` は editor / web / LSP が共有する analysis API を提供し、lex / parse / name resolution / semantics / hover / definition / inlay hint につながる情報を構造化している。`nepl-lsp` は JSON-RPC LSP の薄い server として `nepl-language` を使い、hover / definition / semanticTokens / inlayHint を返す。

diagnostic code は `DiagnosticCode` から stable string を受け取る方向で、source policy `test_diagnostic_code_first_boundary.js` の対象にもなっている。これは Rust compiler diagnostic redesign と整合している。

## Actions 根拠

Actions run `25157230630` では `compile-test` が success なので、`nepl-language` と `nepl-lsp` は compile gate を通っている。editor-specific source policy は build job の `Source policy regressions` で warn-only 実行され、build job 自体は success である。

## 良い点

- analysis output が `TextRange`, `EditorDiagnostic`, `TokenInfo`, `NameResolutionAnalysis`, `SemanticsAnalysis` などの typed struct になっている。
- LSP server は root/stdlib 設定、didOpen/didChange/didSave、hover/definition/semantic tokens/inlay hints を持つ。
- diagnostic code contract は Rust compiler / web / LSP の共通方針に寄せられている。

## 問題

- `nepl-language/src/lib.rs` は約 57KB で、analysis data model と実装が集中している。
- `nepl-lsp/src/main.rs` は JSON-RPC framing、document state、analysis update、LSP response construction が 1 file にまとまっている。
- LSP は compile gate は通るが、CI 上で LSP protocol fixture を直接確認する job は見当たらない。
- semantic output は Rust compiler の AST/HIR 変更に追従し続ける必要があり、今回も過去 issue で removed field 参照が出ている。

## 必要な設計

- `nepl-language` は analysis model、loader adapter、name trace、semantic trace、hover/definition builder に分割する。
- `nepl-lsp` は JSON-RPC transport、document store、capability handlers を分ける。
- LSP fixture を `tests/playground_editor` と同様に JSON request/expected で持つ。
- diagnostic code は code-less diagnostic を作れない source policy を維持する。

## 進捗状況

- `nepl-language`: 実用段階だが分割対象。
- `nepl-lsp`: 基本機能あり、protocol fixture は不足。
- diagnostic code policy: あり。
- CI compile gate: 通過。
