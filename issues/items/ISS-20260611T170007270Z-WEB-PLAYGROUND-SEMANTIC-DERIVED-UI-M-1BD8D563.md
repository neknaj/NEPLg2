---
id: ISS-20260611T170007270Z-WEB-PLAYGROUND-SEMANTIC-DERIVED-UI-M-1BD8D563
title: "Web Playground semantic derived UI must require fresh analysis snapshots"
area: tools
status: open
resolved: false
priority: P0
type: bug
created: 2026-06-11
updated: 2026-06-11
target: "web/src/language/neplg2/neplg2-provider.ts; web/src/editor-core/language-analysis.ts; web/src/editor/editor.ts"
---

# ISS-20260611T170007270Z-WEB-PLAYGROUND-SEMANTIC-DERIVED-UI-M-1BD8D563: Web Playground semantic derived UI must require fresh analysis snapshots

## 概要

Completion, occurrence, hover, definition and token insight can read remapped or stale semantic payloads after text edits because the payload does not expose freshness or document identity.

## 対象

- `web/src/language/neplg2/neplg2-provider.ts; web/src/editor-core/language-analysis.ts; web/src/editor/editor.ts`

## 根拠

- 未記入

## 問題

Completion, occurrence, hover, definition and token insight can read remapped or stale semantic payloads after text edits because the payload does not expose freshness or document identity.

## 影響

Deleted names, old diagnostics, stale definitions, and wrong path context can appear as current editor state during the debounce window.

## 修正方針

Add document/path/version/freshness metadata to analysis payloads and make semantic-derived UI return pending or empty results unless the snapshot is fresh for the active document.

## 検証

Edit text after a semantic payload is published and assert completion, occurrence, definition, hover, and token insight do not use stale semantic data before the fresh analysis response arrives.
