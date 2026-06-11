---
id: ISS-20260611T170007276Z-WEB-PLAYGROUND-DEFINITION-NAVIGATION-6DDD7F9A
title: "Web Playground definition navigation must carry target path and range"
area: tools
status: open
resolved: false
priority: P0
type: bug
created: 2026-06-11
updated: 2026-06-11
target: "web/src/editor-core/language-analysis.ts; web/src/editor/editor-input-handler.ts; web/src/workspace/panel-manager.ts"
---

# ISS-20260611T170007276Z-WEB-PLAYGROUND-DEFINITION-NAVIGATION-6DDD7F9A: Web Playground definition navigation must carry target path and range

## 概要

Definition location returns only targetIndex, so cross-file definitions from VFS analysis cannot open the defining file or preserve the destination range.

## 対象

- `web/src/editor-core/language-analysis.ts; web/src/editor/editor-input-handler.ts; web/src/workspace/panel-manager.ts`

## 根拠

- 未記入

## 問題

Definition location returns only targetIndex, so cross-file definitions from VFS analysis cannot open the defining file or preserve the destination range.

## 影響

Go-to-definition on imported or stdlib symbols moves within the active file at a coincidental offset instead of navigating to the real definition.

## 修正方針

Return a typed definition target containing targetPath and targetRange. Route cross-file targets through workspace tab open and atomic document replacement before moving the cursor.

## 検証

Add a cross-file fixture where a symbol resolves to another file and assert the target tab opens and the cursor lands on the definition range.
