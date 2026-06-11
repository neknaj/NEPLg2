---
id: ISS-20260611T164153310Z-WORKSPACE-EDITORS-NEED-SHARED-DOCUME-1D4DCAE1
title: "Workspace editors need shared document ownership and dirty close handling"
area: tools
status: open
resolved: false
priority: P1
type: bug
created: 2026-06-11
updated: 2026-06-11
target: "web/src/workspace/panel-manager.ts; web/src/library/tabs.ts; web/src/runtime/vfs.ts"
---

# ISS-20260611T164153310Z-WORKSPACE-EDITORS-NEED-SHARED-DOCUME-1D4DCAE1: Workspace editors need shared document ownership and dirty close handling

## 概要

Multiple editor panels can open the same path with independent buffers. Closing an active dirty tab can drop unsaved editor text, and run/build syncs only the focused editor.

## 対象

- `web/src/workspace/panel-manager.ts; web/src/library/tabs.ts; web/src/runtime/vfs.ts`

## 根拠

- 未記入

## 問題

Multiple editor panels can open the same path with independent buffers. Closing an active dirty tab can drop unsaved editor text, and run/build syncs only the focused editor.

## 影響

User edits can disappear, compile input can omit dirty buffers in other panels, and same-path edits are last-writer-wins without conflict handling.

## 修正方針

Introduce shared document state by canonical path or prevent duplicate writable opens. Tab close must save/confirm/discard explicitly, and compile/run must use a consistent dirty-document policy.

## 検証

Tests for active dirty tab close, same path in two editors, compile after editing in a non-focused panel, and conflict/dirty status behavior.
