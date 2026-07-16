---
id: ISS-20260716T031826603Z-REGISTERED-RAW-STROKE-COVERAGE-LACKS-6E469EAF
title: "Registered raw stroke coverage lacks a packed mask owner"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# ISS-20260716T031826603Z-REGISTERED-RAW-STROKE-COVERAGE-LACKS-6E469EAF: Registered raw stroke coverage lacks a packed mask owner

## 概要

F5nxr completes registered raw coverage cells, but no registered authority-preserving packed alpha mask boundary consumes that owner.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl`

## 根拠

- F5nxr completed ownerはexact raw coverage cellsを保持するが、registered module内にそのownerを消費するpacked alpha mask boundaryがない。
- legacy F5lf/F5bfは異なるowner authorityを要求し、直接再利用するとregistered authorityを再構築または分断する。

## 問題

F5nxr completes registered raw coverage cells, but no registered authority-preserving packed alpha mask boundary consumes that owner.

## 影響

The registered glyph path cannot advance from coverage into raster resources or GUI presentation without repackaging legacy authority or leaving the pipeline disconnected.

## 修正方針

Add F5nxs with the completed F5nxq/F5nxr coverage owner as sole nested authority, checked alpha normalization, bounded 0/1-cell polling, owner-bearing recovery, runtime fixtures, source policy, normal compile regression, and coherent docs.

## 検証

Focused runtime tests prove actual F5nxr to F5nxs flow, alpha floor normalization, budget and recovery invariants; source-policy and normal compile gates pass; subagent diff and whole-contract reviews approve.
