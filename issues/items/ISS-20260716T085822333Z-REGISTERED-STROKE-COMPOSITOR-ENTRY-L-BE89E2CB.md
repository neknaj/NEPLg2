---
id: ISS-20260716T085822333Z-REGISTERED-STROKE-COMPOSITOR-ENTRY-L-BE89E2CB
title: "Registered stroke compositor entry lacks direct batch-range bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260716T085822333Z-REGISTERED-STROKE-COMPOSITOR-ENTRY-L-BE89E2CB: Registered stroke compositor entry lacks direct batch-range bridge

## 概要

F5nya creates a compositor entry but registered stroke has no direct owner-preserving path into existing F5mb batch-range preparation; chaining F5nyb would skip descriptors because F5ma advances the cursor.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- F5ma/F5nybはcursorを進めてdescriptorを保持しないため、そのterminalからF5mbへ進むとbatch 0 payloadを欠落させる。F5nya entryからF5mbへの直接経路が必要だった。

## 問題

F5nya creates a compositor entry but registered stroke has no direct owner-preserving path into existing F5mb batch-range preparation; chaining F5nyb would skip descriptors because F5ma advances the cursor.

## 影響

Registered stroke output cannot enter payload metadata preparation without caller-side plumbing or an incorrect cursor-advancing detour.

## 修正方針

Add a direct F5nya-to-F5mb registered bridge that preserves lower owner-bearing errors and batch-range ownership; explicitly prohibit F5ma/F5nyb delegation and payload transport.

## 検証

Focused first-range success and entry-bridge recovery fixtures; existing F5mb recovery contract delegation; source policy; normal compile; issues/diff checks; subagent review.

Focused first-range successとentry-bridge recovery、Web GUI contract、normal compile isolation、issues/diff check、subagent reviewを通過した。
