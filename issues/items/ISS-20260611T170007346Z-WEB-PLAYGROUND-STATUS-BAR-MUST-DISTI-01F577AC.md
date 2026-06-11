---
id: ISS-20260611T170007346Z-WEB-PLAYGROUND-STATUS-BAR-MUST-DISTI-01F577AC
title: "Web Playground status bar must distinguish non editor focus from stale editor analysis"
area: tools
status: open
resolved: false
priority: P1
type: bug
created: 2026-06-11
updated: 2026-06-11
target: "web/src/workspace/panel-manager.ts; web/src/main.ts"
---

# ISS-20260611T170007346Z-WEB-PLAYGROUND-STATUS-BAR-MUST-DISTI-01F577AC: Web Playground status bar must distinguish non editor focus from stale editor analysis

## 概要

Status sync falls back to an editor runtime even when terminal, explorer, or GUI focus is active, so cursor and token analysis from a previous editor can be displayed as current context.

## 対象

- `web/src/workspace/panel-manager.ts; web/src/main.ts`

## 根拠

- 未記入

## 問題

Status sync falls back to an editor runtime even when terminal, explorer, or GUI focus is active, so cursor and token analysis from a previous editor can be displayed as current context.

## 影響

Users can see stale editor state while interacting with terminal or GUI windows, and delayed analysis insight may update the status bar for the wrong focus owner.

## 修正方針

Track focused surface kind explicitly and clear or replace editor-specific status fields when focus is not an editor. Delayed analysis updates must include the originating editor identity and be dropped after focus changes.

## 検証

Focus editor, terminal, explorer, and GUI window in sequence and assert editor token/cursor status only appears for the active editor.
