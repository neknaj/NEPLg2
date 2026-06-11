---
id: ISS-20260611T170007277Z-EDITOR-PANELS-NEED-EXPLICIT-DISPOSE--3406EA52
title: "Editor panels need explicit dispose lifecycle for DOM listeners observers and render loops"
area: tools
status: open
resolved: false
priority: P1
type: bug
created: 2026-06-11
updated: 2026-06-11
target: "web/src/workspace/panel-manager.ts; web/src/editor/editor.ts; web/src/editor/editor-input-handler.ts; web/src/editor/editor-renderer.ts"
---

# ISS-20260611T170007277Z-EDITOR-PANELS-NEED-EXPLICIT-DISPOSE--3406EA52: Editor panels need explicit dispose lifecycle for DOM listeners observers and render loops

## 概要

Workspace panel removal disposes terminal runtimes, but editor runtimes do not expose a matching dispose path for input listeners, resize observers, animation frames, language callbacks, or DOM references.

## 対象

- `web/src/workspace/panel-manager.ts; web/src/editor/editor.ts; web/src/editor/editor-input-handler.ts; web/src/editor/editor-renderer.ts`

## 根拠

- 未記入

## 問題

Workspace panel removal disposes terminal runtimes, but editor runtimes do not expose a matching dispose path for input listeners, resize observers, animation frames, language callbacks, or DOM references.

## 影響

Opening, splitting, and closing editor panels can retain event listeners and render work for panels no longer visible.

## 修正方針

Add CanvasEditor/PlaygroundEditor dispose methods and call them from PanelManager when editor runtimes are removed. Dispose must detach listeners, observers, timers, language callbacks, and renderer loops.

## 検証

Create and close editor panels repeatedly and assert listener/observer counters and render loop state return to baseline.
