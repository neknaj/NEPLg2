---
id: ISS-20260717T053248243Z-REGISTERED-RLE-WRITER-PLAN-LACKS-STO-E6D26A76
title: "Registered RLE writer plan lacks storage bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_storage.nepl
---

# ISS-20260717T053248243Z-REGISTERED-RLE-WRITER-PLAN-LACKS-STO-E6D26A76: Registered RLE writer plan lacks storage bridge

## 概要

F5nyl produces registered checked writer-plan authority but the registered stroke path cannot enter existing F5ml encoded storage allocation.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_storage.nepl`

## 根拠

- F5nylはchecked writer-plan ownerを返すが、registered stroke production graphにはそのownerをF5mlへ渡すadapterがなく、exact encoded storage authorityへ到達できなかった。
- allocation、metadata保持、plan recovery、storage freeは既存F5mlが所有する契約であり、registered側で再実装せずpublic F5nyl successからexact-onceで再利用する必要がある。

## 問題

F5nyl produces registered checked writer-plan authority but the registered stroke path cannot enter existing F5ml encoded storage allocation.

## 影響

Registered glyph RLE output cannot allocate the exact encoded byte storage required by the write cursor.

## 修正方針

Add an F5nym lossless bridge from public F5nyl success to existing F5ml, preserving all seven staged errors and stopping before write cursor or encoded sealing.

## 検証

Focused reachable storage success; F5nyl and F5ml regressions; source policy; normal compile; trunk/CLI; issues/diff; subagent review.

Focused F5nym runtimeはmetadata 261/16/3/1、production-derived run count 1、exact capacity 12、cursor 0/16、storage freeのevidence 31を通過した。upstream F5nyl evidence 63、既存F5ml storage/error recovery回帰、Web source-policy、targeted normal isolation、`trunk build`、trunk後Playground editor JSON 13/13、issues/diff check、subagent差分・全体整合reviewを通過した。全体整合reviewの指摘により、自然到達不能なallocation failureをfixtureで偽造せず既存F5ml回帰へ委譲する契約をspecとsource-policyへ追加した。
