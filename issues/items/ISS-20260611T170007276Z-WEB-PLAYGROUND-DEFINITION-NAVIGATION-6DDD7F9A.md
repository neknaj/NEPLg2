---
id: ISS-20260611T170007276Z-WEB-PLAYGROUND-DEFINITION-NAVIGATION-6DDD7F9A
title: "Web Playground definition navigation must carry target path and range"
area: tools
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-06-11
updated: 2026-06-12
target: "web/src/editor-core/language-analysis.ts; web/src/editor/editor-input-handler.ts; web/src/workspace/panel-manager.ts"
---

# ISS-20260611T170007276Z-WEB-PLAYGROUND-DEFINITION-NAVIGATION-6DDD7F9A: Web Playground definition navigation must carry target path and range

## 概要

Definition location returns only targetIndex, so cross-file definitions from VFS analysis cannot open the defining file or preserve the destination range.

## 対象

- `web/src/editor-core/language-analysis.ts; web/src/editor/editor-input-handler.ts; web/src/workspace/panel-manager.ts`

## 根拠

- `DefinitionLocation.targetIndex` は active editor の UTF-16 index であり、cross-file target では意味を持たない。
- compiler 由来の `targetSpan` / `targetByteRange` は target file 本文に対して再解決する必要がある。
- editor input layer が workspace/tab を直接 import すると、入力処理と file/tab lifecycle の責務が混ざる。

## 問題

Definition location returns only targetIndex, so cross-file definitions from VFS analysis cannot open the defining file or preserve the destination range.

## 影響

Go-to-definition on imported or stdlib symbols moves within the active file at a coincidental offset instead of navigating to the real definition.

## 修正方針

Return a typed definition target containing targetPath and targetRange. Route cross-file targets through workspace tab open and atomic document replacement before moving the cursor.

## 実装

- `language-analysis.ts` に `mapAnalysisSpanToTextRange` を公開し、target file text に対して line/col または byte span を UTF-16 editor range へ変換できるようにした。
- `CanvasEditor` / `PlaygroundEditor` に definition navigation callback と cursor range movement adapter を追加した。
- `editor-input-handler.ts` は F12 の same-file definition だけを直接 cursor 移動し、cross-file definition は `onDefinitionNavigation` に委譲する。
- `PanelManager.openDefinitionTarget` が target path を VFS で fail-closed に確認し、target tab を開いてから target file text 上で range を解決し cursor を移す。

## 検証

Add a cross-file fixture where a symbol resolves to another file and assert the target tab opens and the cursor lands on the definition range.

- `npm --prefix web run build:ts`
- `node nodesrc/test_playground_definition_navigation_contract.js`
- `node nodesrc/test_playground_analysis_span_identity.js`
- `node nodesrc/test_playground_analysis_freshness.js`
- `node nodesrc/test_neplg2_language_provider_vfs.js`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests-definition-navigation.json`
