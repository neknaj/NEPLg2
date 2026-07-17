---
id: ISS-20260717T113857267Z-F5NZC-REGISTERED-TERMINAL-HOST-COMMA-39CC9EBA
title: "F5nzc registered terminal host-command record projection"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_terminal_record.nepl
---

# ISS-20260717T113857267Z-F5NZC-REGISTERED-TERMINAL-HOST-COMMA-39CC9EBA: F5nzc registered terminal host-command record projection

## 概要

F5nzb terminal command step is not yet projected through the existing F5mu typed host-command boundary.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_terminal_record.nepl`

## 根拠

- F5nzbはterminal command-cursor stepとそのdescriptor/owner authorityを保持するが、registered graphは既存F5mu projectionをまだ呼ばない。
- F5muはborrowed command-cursor stepからmetadata-preserving host-command resultを作る唯一の既存境界であり、registered側でresultやrecordを再構築してはならない。
- F5nzcはterminal stepとCopy projection resultを同じmove-only ownerへ保持するまでを対象とし、host execution、virtual drain、schedulerを完成条件に含めない。

## 問題

F5nzb terminal command step is not yet projected through the existing F5mu typed host-command boundary.

## 影響

The registered stroke compositor path stops before a typed terminal record result.

## 修正方針

Pass the opaque F5nzb terminal step by borrow to F5mu exactly once and preserve the step plus Copy projection result in a new move-only twenty-third-layer owner without host or virtual drain.

## 検証

Focused runtime fixture, source policy, normal compile isolation, lower regressions, trunk build, CLI JSON, and subagent reviews.

## 完了

F5nzbの二十二層を保持したままterminal stepをF5muへ一度だけborrowし、元stepとCopy projectionをmove-only ownerへ保持する第二十三層authorityを追加した。深いaggregate Resultの生成Wasmがbyte単位zero/copy展開でvalidator上限を超える根因は、backendを`memory.fill` / `memory.copy`の定数長命令列へ変更して解消した。focused runtime、source policy、normal compile、lower regressions、release trunk build、CLI JSON、subagent reviewを通過した。
