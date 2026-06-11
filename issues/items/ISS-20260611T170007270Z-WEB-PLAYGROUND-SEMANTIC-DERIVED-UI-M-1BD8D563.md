---
id: ISS-20260611T170007270Z-WEB-PLAYGROUND-SEMANTIC-DERIVED-UI-M-1BD8D563
title: "Web Playground semantic derived UI must require fresh analysis snapshots"
area: tools
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-06-11
updated: 2026-06-12
target: "web/src/language/neplg2/neplg2-provider.ts; web/src/editor-core/language-analysis.ts; web/src/editor/editor.ts"
---

# ISS-20260611T170007270Z-WEB-PLAYGROUND-SEMANTIC-DERIVED-UI-M-1BD8D563: Web Playground semantic derived UI must require fresh analysis snapshots

## 概要

Completion, occurrence, hover, definition and token insight can read remapped or stale semantic payloads after text edits because the payload does not expose freshness or document identity.

## 対象

- `web/src/language/neplg2/neplg2-provider.ts; web/src/editor-core/language-analysis.ts; web/src/editor/editor.ts`

## 根拠

- `NEPLg2LanguageProvider` は編集後の debounce window 中に前回の `resolve` / `semantics` と remap 済み editor payload を保持していた。
- その payload には path、document version、freshness が無く、completion / hover / definition / occurrence / token insight が「現在文書に対して fresh か」を判定できなかった。

## 問題

Completion, occurrence, hover, definition and token insight can read remapped or stale semantic payloads after text edits because the payload does not expose freshness or document identity.

## 影響

Deleted names, old diagnostics, stale definitions, and wrong path context can appear as current editor state during the debounce window.

## 修正方針

Add document/path/version/freshness metadata to analysis payloads and make semantic-derived UI return pending or empty results unless the snapshot is fresh for the active document.

## 検証

Edit text after a semantic payload is published and assert completion, occurrence, definition, hover, and token insight do not use stale semantic data before the fresh analysis response arrives.

## 解決

- `EditorUpdatePayload.analysis` に path、documentVersion、sourcePath、sourceDocumentVersion、analysisVersion、freshness、isFresh を追加した。
- `NEPLg2LanguageProvider` は文書 path/text の変更ごとに `documentVersion` を進め、empty / provisional / fresh の payload を明示的に発行する。
- semantic analysis と structural analysis は開始時の path/text/documentVersion を捕捉し、完了時に現在文書と一致しない結果を破棄する。
- completion の symbol 候補、occurrence、hover、definition、token insight は fresh snapshot でない限り semantic payload を読まない。
- provisional payload は diagnostics、folding ranges、semantic tokens、inlay hints を保持せず、editor / problems API も fresh でない semantic-derived payload を採用しない。
- token coloring は lexical `tokens` と semantic `semanticHighlightTokens` を分離し、editor は fresh payload のときだけ semantic overlay を重ねる。
- 連続編集中の provisional metadata は直前 documentVersion ではなく、元の fresh analysis input の sourceDocumentVersion/sourcePath を引き継ぐ。

## 解決時の検証

- pass: `npm --prefix web run build:ts`
- pass: `node nodesrc/test_playground_analysis_freshness.js`
- pass: `node nodesrc/test_neplg2_language_provider_vfs.js`
- pass: `node nodesrc/test_playground_editor_performance_policy.js`
