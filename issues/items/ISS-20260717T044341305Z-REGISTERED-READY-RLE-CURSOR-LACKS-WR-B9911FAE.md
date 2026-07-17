---
id: ISS-20260717T044341305Z-REGISTERED-READY-RLE-CURSOR-LACKS-WR-B9911FAE
title: "Registered ready RLE cursor lacks writer plan bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_writer_plan.nepl
---

# ISS-20260717T044341305Z-REGISTERED-READY-RLE-CURSOR-LACKS-WR-B9911FAE: Registered ready RLE cursor lacks writer plan bridge

## 概要

F5nyk produces registered ready cursor authority but the registered stroke path cannot enter existing F5mk checked writer capacity planning.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_writer_plan.nepl`

## 根拠

- F5nykはmetadata付きready cursor ownerを返すが、registered stroke production graphにはそのownerをF5mkへ渡すadapterがなく、exact encoded byte capacity evidenceへ到達できなかった。
- checked `total_run_count * 12`、overflow classification、ready-owner recoveryは既存F5mkが所有する契約であり、registered側で再実装せずpublic F5nyk successからexact-onceで再利用する必要がある。

## 問題

F5nyk produces registered ready cursor authority but the registered stroke path cannot enter existing F5mk checked writer capacity planning.

## 影響

Registered glyph RLE output cannot advance from a ready cursor to exact encoded byte capacity evidence.

## 修正方針

Add an F5nyl lossless bridge from public F5nyk success to existing F5mk, preserving all six staged errors and stopping before storage or writes.

## 検証

Focused reachable writer-plan success; F5nyk and F5mk regressions; source policy; normal compile; trunk/CLI; issues/diff; subagent review.

Focused F5nyl runtimeはmetadata、production-derived run count 1、checked capacity 12、cursor 0/16、payload 64/freeを通過した。旧F5nyi/F5nyj/F5nyk fixtureの誤期待3も実値1へ訂正した。既存F5mk回帰、Web source-policy、targeted normal isolation、trunk build、Playground editor JSON 13/13、issues/diff check、subagent差分・全体整合reviewを通過した。
