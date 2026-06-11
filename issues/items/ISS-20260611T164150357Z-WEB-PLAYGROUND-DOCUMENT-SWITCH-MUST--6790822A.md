---
id: ISS-20260611T164150357Z-WEB-PLAYGROUND-DOCUMENT-SWITCH-MUST--6790822A
title: "Web Playground document switch must be atomic across path text and analysis state"
area: tools
status: open
resolved: false
priority: P0
type: bug
created: 2026-06-11
updated: 2026-06-11
target: "web/src/library/tabs.ts; web/src/editor-core/browser-adapter.ts; web/src/language/neplg2/neplg2-provider.ts"
---

# ISS-20260611T164150357Z-WEB-PLAYGROUND-DOCUMENT-SWITCH-MUST--6790822A: Web Playground document switch must be atomic across path text and analysis state

## 概要

Tab open currently updates path and text through separate notifications. A path-only switch can schedule analysis with stale text, and same-content different-path documents can keep old payload or path context.

## 対象

- `web/src/library/tabs.ts; web/src/editor-core/browser-adapter.ts; web/src/language/neplg2/neplg2-provider.ts`

## 根拠

- 未記入

## 問題

Tab open currently updates path and text through separate notifications. A path-only switch can schedule analysis with stale text, and same-content different-path documents can keep old payload or path context.

## 影響

Imports, diagnostics, definitions, hover, and occurrences can be shown for the wrong file context. This becomes worse with VFS-backed analysis and multi-file projects.

## 修正方針

Introduce an atomic replaceDocument contract carrying path, text, editable state, and VFS/document revision. Provider state clearing, version issuance, and analysis scheduling must happen from that single operation.

## 検証

Add fixtures opening two files with identical text but different paths/import context and confirm analysis is rerun for the new path and stale payload is cleared.
