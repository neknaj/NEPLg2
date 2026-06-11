---
id: ISS-20260611T164150659Z-WEB-PLAYGROUND-ANALYSIS-SPANS-MUST-P-CFF77E8E
title: "Web Playground analysis spans must preserve file identity and character offset mapping"
area: tools
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-06-11
updated: 2026-06-12
target: "nepl-web/src/lib.rs; web/src/editor-core/language-analysis.ts; web/src/language/neplg2/neplg2-provider.ts"
---

# ISS-20260611T164150659Z-WEB-PLAYGROUND-ANALYSIS-SPANS-MUST-P-CFF77E8E: Web Playground analysis spans must preserve file identity and character offset mapping

## 概要

The Rust analysis payload can contain byte-based spans and file_path-bearing spans, but the TypeScript payload builder maps semantic expression spans and cross-file diagnostics/definitions as if they belong to the active editor text.

## 対象

- `nepl-web/src/lib.rs; web/src/editor-core/language-analysis.ts; web/src/language/neplg2/neplg2-provider.ts`

## 根拠

- 未記入

## 問題

The Rust analysis payload can contain byte-based spans and file_path-bearing spans, but the TypeScript payload builder maps semantic expression spans and cross-file diagnostics/definitions as if they belong to the active editor text.

## 影響

Non-ASCII source shifts inlay/hover ranges, and diagnostics or definitions from imported files can be underlined or navigated at wrong positions in the active file.

## 修正方針

Normalize all spans through a shared active-file-aware span mapper. Editor-local diagnostics/tokens must be filtered by active path, while definition targets should carry path plus target range.

## 検証

Add non-ASCII and cross-file import playground fixtures. Active editor must not display imported-file diagnostics at local offsets, and definition targets must preserve destination path.

## 修正内容

- `web/src/editor-core/language-analysis.ts` の span mapping を active path aware にし、`file_path` が active editor と異なる diagnostic / token / semantic span / occurrence を editor-local payload から除外した。
- `diagnostics` は Rust payload の `primary` を正規 span として扱うようにし、legacy `span` も互換入力として残した。
- `token_semantics` の `expr_span` / `arg_span`、inlay hint、hover expression は byte offset を直接 UI index として使わず、共有 mapper を通すようにした。
- cross-file definition は `targetPath` と raw `targetSpan` / `targetByteRange` を保持し、active editor の `targetRange` は `null` にして local cursor が byte offset へ誤移動しないようにした。
- `web/src/language/neplg2/neplg2-provider.ts` と `web/src/language/neplg2/neplg2-analysis-worker.ts` は bridge snapshot に `path` / `sourcePath` / `activePath` を渡す。
- provider 内に残っていた active text 専用の旧 span mapper と offset map を削除し、span authority を `editor-core/language-analysis.ts` に一本化した。
- `nepl-web/src/lib.rs` の VFS semantic analysis diagnostics は `SourceMap` 付き helper で `file_path` を保持するようにした。

## 残件

- cross-file definition の target file を実際に workspace tab で開き、target file 本文に対する char range へ cursor を移す UI 経路は `ISS-20260611T170007276Z-WEB-PLAYGROUND-DEFINITION-NAVIGATION-6DDD7F9A` で継続する。

## 確認

- `npm --prefix web run build:ts`
- `node nodesrc/test_playground_analysis_span_identity.js`
- `node nodesrc/test_editor_diagnostic_code_contract.js`
- `node nodesrc/test_editor_current_syntax_highlighting.js`
- `node nodesrc/test_neplg2_language_provider_vfs.js`
- `node nodesrc/test_playground_analysis_freshness.js`
- `node nodesrc/test_playground_editor_performance_policy.js`
- `node nodesrc/test_diagnostic_code_first_boundary.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/agent2-playground-editor-tests.json`
- `git diff --check`
