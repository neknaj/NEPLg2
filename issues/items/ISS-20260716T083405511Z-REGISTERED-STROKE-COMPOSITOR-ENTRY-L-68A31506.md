---
id: ISS-20260716T083405511Z-REGISTERED-STROKE-COMPOSITOR-ENTRY-L-68A31506
title: "Registered stroke compositor entry lacks bounded batch drain bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260716T083405511Z-REGISTERED-STROKE-COMPOSITOR-ENTRY-L-68A31506: Registered stroke compositor entry lacks bounded batch drain bridge

## 概要

F5nya creates the existing compositor frame-entry owner, but the registered stroke path cannot invoke the existing bounded compositor batch drain without caller-side owner-stage plumbing.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- F5nyaはregistered completed ownerをcompositor frame entryへ接続するが、bounded F5ma drainのterminal/error continuationをregistered lifetime段階へ結ぶAPIは存在しなかった。

## 問題

F5nya creates the existing compositor frame-entry owner, but the registered stroke path cannot invoke the existing bounded compositor batch drain without caller-side owner-stage plumbing.

## 影響

Registered stroke rendering reaches compositor metadata but cannot participate in bounded scheduler progress or recover its entry authority through a registered API.

## 修正方針

Add a registered composite bridge that calls F5nya then existing F5ma exactly once, retaining stage-specific owner-bearing errors and terminal continuation without entering row range, payload, transport, or presentation.

## 検証

Focused zero/positive/invalid budget runtime fixtures; Web GUI source policy; normal compile isolation; issue/diff checks; subagent review.

Focused budget 0 continuation、budget 1 completion、negative budget entry recovery、Web GUI source-policy、normal compile isolation、issues/diff check、subagent reviewを通過した。
