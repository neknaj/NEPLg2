---
id: ISS-20260717T061510619Z-REGISTERED-RLE-WRITE-CURSOR-LACKS-WR-8D372E6B
title: "Registered RLE write cursor lacks write step bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_write_step.nepl
---

# ISS-20260717T061510619Z-REGISTERED-RLE-WRITE-CURSOR-LACKS-WR-8D372E6B: Registered RLE write cursor lacks write step bridge

## 概要

F5nyn produces a registered write cursor but the registered stroke path cannot execute existing F5mn write step.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_write_step.nepl`

## 根拠

- F5nynはmetadata付きwrite cursor ownerを返すが、registered stroke production graphにはそのownerをF5mnへ渡すadapterがなく、encoded runをcommitできなかった。
- one-step status、committed written counts、cursor progress、error recoveryは既存F5mnが所有する契約であり、registered側で再実装せずpublic F5nyn successからexact-onceで再利用する必要がある。

## 問題

F5nyn produces a registered write cursor but the registered stroke path cannot execute existing F5mn write step.

## 影響

Registered glyph RLE output cannot commit its first encoded run.

## 修正方針

Add an F5nyo lossless bridge from public F5nyn success to existing F5mn one-step writer, preserving staged errors and stopping before completion probe or sealing.

## 検証

Focused reachable WroteRun success; F5nyn and F5mn regressions; source policy; normal compile; trunk/CLI; issues/diff; subagent review.

Focused F5nyo runtimeは`WroteRun`、metadata 263/16/3/1、run count 1、capacity 12、written 1/12、cursor 16/16、owner freeのevidence 255をtrunk前後で通過した。upstream F5nyn evidence 127、既存F5mn one-step/error recovery回帰、Web source-policy、targeted normal isolation、`trunk build`、trunk後Playground editor JSON 13/13、issues/diff check、subagent差分・全体整合reviewを通過した。review指摘によりsource-policyを九層すべてのErr再包装順序へ強化した。
