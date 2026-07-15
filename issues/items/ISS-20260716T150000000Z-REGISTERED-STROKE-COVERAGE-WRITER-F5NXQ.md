---
id: ISS-20260716T150000000Z-REGISTERED-STROKE-COVERAGE-WRITER-F5NXQ
title: "Registered stroke join geometry lacks a coverage cell writer"
area: GUI_FONT
status: investigating
resolved: false
priority: P1
type: architecture
created: 2026-07-16
updated: 2026-07-16
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour.nepl
---

# Registered stroke coverage writer F5nxq

## Problem

F5nxp completed owner は registered glyph の side-edge、closure、bevel / miter / round join geometry を保持するが、coverage scan が結果を安全に書き込む exact-capacity cell storage owner をまだ持たない。

## Scope

- F5nxp completed owner を sole direct authority とする registered stroke coverage cell writer owner を追加する。
- shared coverage config / shape validation を再利用し、F5nxp source、geometry storage、分類 count を start 前に再検査する。
- cell storage を exact capacity で一度だけ確保し、range-checked push、pre-push owner recovery、exact-full completion、single free を実装する。
- runtime fixture、source-policy、normal compile、仕様・設計・todo・note、trunk / CLI 回帰を同時更新する。

## Out of scope

coverage scan computation、quadratic flattening、packed mask、paint composition、glyph raster/runtime bridge、native/Web GUI 表示は後続 phase とする。本 issue の完了をフォントレンダリングエンジンまたは GUI ライブラリ全体の完成とは扱わない。

## Acceptance

- diff review と全体整合 review に Blocker / Major がない。
- focused runtime、registered module、glyf module、source-policy、normal compile、GUI contract、trunk build、Playground editor CLI JSON が通過する。
- checkpoint は作業 branch に保持し、全 gate 後だけ lease 下で main へ一度統合する。
