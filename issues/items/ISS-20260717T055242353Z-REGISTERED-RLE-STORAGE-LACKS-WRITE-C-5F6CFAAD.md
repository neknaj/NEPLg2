---
id: ISS-20260717T055242353Z-REGISTERED-RLE-STORAGE-LACKS-WRITE-C-5F6CFAAD
title: "Registered RLE storage lacks write cursor bridge"
area: GUI_FONT
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_write_cursor.nepl
---

# ISS-20260717T055242353Z-REGISTERED-RLE-STORAGE-LACKS-WRITE-C-5F6CFAAD: Registered RLE storage lacks write cursor bridge

## 概要

F5nym produces registered encoded storage authority but the registered stroke path cannot enter existing F5mm write cursor start.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_write_cursor.nepl`

## 根拠

- F5nymはmetadata付きencoded storage ownerを返すが、registered stroke production graphにはそのownerをF5mmへ渡すadapterがなく、write cursor authorityへ到達できなかった。
- start validation、storage recovery、initial written run / byte count 0は既存F5mmが所有する契約であり、registered側で再実装せずpublic F5nym successからexact-onceで再利用する必要がある。

## 問題

F5nym produces registered encoded storage authority but the registered stroke path cannot enter existing F5mm write cursor start.

## 影響

Registered glyph RLE output cannot obtain a metadata-bearing write cursor for encoded runs.

## 修正方針

Add an F5nyn lossless bridge from public F5nym success to existing F5mm, preserving all eight staged errors and stopping before write step or encoded sealing.

## 検証

Focused reachable write cursor success; F5nym and F5mm regressions; source policy; normal compile; trunk/CLI; issues/diff; subagent review.

Focused F5nyn runtimeはmetadata 262/16/3/1、run count 1、capacity 12、cursor 0/16、written run / byte count 0、write cursor freeのevidence 127を通過した。upstream F5nym evidence 31、既存F5mm write cursor start/error recovery回帰、Web source-policy、targeted normal isolation、一時`npm.cmd` shim経由の`trunk build`、trunk後Playground editor JSON 13/13、issues/diff check、subagent差分・全体整合reviewを通過した。初回doctest rootの非canonical metadata/import/main契約を既存stanzaと同じ形式へ根本修正した。
