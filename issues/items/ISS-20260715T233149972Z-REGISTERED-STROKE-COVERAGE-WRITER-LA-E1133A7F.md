---
id: ISS-20260715T233149972Z-REGISTERED-STROKE-COVERAGE-WRITER-LA-E1133A7F
title: "Registered stroke coverage writer lacks a scan converter"
area: GUI_FONT
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-15
updated: 2026-07-15
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260715T233149972Z-REGISTERED-STROKE-COVERAGE-WRITER-LA-E1133A7F: Registered stroke coverage writer lacks a scan converter

## 概要

F5nxq completed writer storage cannot yet compute coverage from registered side-edge and join geometry authority.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- F5nxqはexact-capacity writerとowner recoveryを提供するが、nested registered side-edge/join geometryからcoverageを計算するconsumerを持たない。
- legacy F5lbはprivate legacy owner chainへ固定されており、registered authorityを詰め替えて再利用できない。

## 問題

F5nxq completed writer storage cannot yet compute coverage from registered side-edge and join geometry authority.

## 影響

Registered glyph stroke rasterization stops before raw coverage cells are populated.

## 修正方針

Add an F5nxr scan owner that consumes the F5nxq writer, borrows registered geometry, computes one cell per step, preserves owner-bearing recovery, and stops before packed mask.

- quadratic subdivisionをboundedにし、Right source reversalとendpoint normal interpolationを維持する。
- crossingをparityで畳み、sample座標とprogressをoverflow-safeに検査する。
- terminal-before-budget、1 step 1 push、exact completion、single freeを固定する。

## 検証

Focused runtime, module, source-policy, normal compile, docs, trunk, CLI, and subagent reviews.

## Out of scope

packed mask、paint composition、raster output、runtime bridge、native/Web GUI表示は後続issueとする。本issueをフォントレンダリングエンジンまたはGUIライブラリ全体の完成とは扱わない。
