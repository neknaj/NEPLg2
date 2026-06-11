---
id: ISS-20260611T164150357Z-WEB-PLAYGROUND-DOCUMENT-SWITCH-MUST--6790822A
title: "Web Playground document switch must be atomic across path text and analysis state"
area: tools
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-06-11
updated: 2026-06-12
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

## 対応

- `TabManager.replaceEditorDocument` を追加し、active tab / placeholder / tab close の editor 更新を `path`、`text`、`editable` の 1 回の document replacement にまとめた。
- `PlaygroundEditor.replaceDocument` と `CanvasEditor.replaceDocument` を追加し、editor surface と language provider の境界も同じ document replacement に揃えた。
- `NEPLg2LanguageProvider.replaceDocument` を追加し、`path` と `text` を同時に更新してから stale analysis state を破棄し、空 payload publish と delayed analysis scheduling を行うようにした。
- legacy `setPath` は path-only context update として扱い、単独で semantic analysis を schedule しないようにした。

## 対応後の検証

- pass: `npm --prefix web run build:ts`
- pass: `node nodesrc/test_neplg2_language_provider_vfs.js`
- pass: `node nodesrc/test_playground_editor_performance_policy.js`
- pass: `node nodesrc/playground_editability_test_runner.js`
- pass: `node nodesrc/playground_editor_surface_test_runner.js`
- pass: `node nodesrc/playground_tab_transfer_test_runner.js`
- pass: `node nodesrc/playground_shell_worker_test_runner.js`
- pass: `node nodesrc/test_web_gui_input_bridge.js`
- pass: `node nodesrc/test_web_gui_shared_event_queue.js`
- pass: `node nodesrc/test_web_gui_floating_window_source.js`

## 残件

- analysis payload の `documentVersion` / freshness metadata は `ISS-20260611T170007270Z-WEB-PLAYGROUND-SEMANTIC-DERIVED-UI-M-1BD8D563` で扱う。
- cross-file definition navigation の `targetPath` / `targetRange` 化は `ISS-20260611T170007276Z-WEB-PLAYGROUND-DEFINITION-NAVIGATION-6DDD7F9A` で扱う。
