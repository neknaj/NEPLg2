---
id: ISS-20260717T103144338Z-REGISTERED-COMMAND-CURSOR-LACKS-FIRS-1ACFCE81
title: "Registered command cursor lacks first BeginFrame step bridge"
area: GUI_FONT
status: open
resolved: false
priority: P1
type: architecture
created: 2026-07-17
updated: 2026-07-17
target: stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_command_cursor_step.nepl
---

# ISS-20260717T103144338Z-REGISTERED-COMMAND-CURSOR-LACKS-FIRS-1ACFCE81: Registered command cursor lacks first BeginFrame step bridge

## 概要

The registered compositor graph stops at the F5nyx BeginPending owner and cannot expose the first typed BeginFrame command.

## 対象

- `stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_command_cursor_step.nepl`

## 根拠

- F5nyxはF5mtの`BeginPending` command-cursor ownerを返すが、registered graphにはそのownerから最初のtyped outputを得るadapterがない。
- `BeginPending`から`BeginFrame`と`RunPending` continuationを作るauthorityは既存F5mt stepにあるため、registered側でphaseやdescriptorを再構築してはならない。
- このissueは最初の`BeginFrame`と継続ownerまでを対象とし、次のRun、EndFrame、terminal Completed、F5mu record、host実行を完成条件に含めない。

- 未記入

## 問題

The registered compositor graph stops at the F5nyx BeginPending owner and cannot expose the first typed BeginFrame command.

## 影響

Registered rendering cannot advance its established F5mt command cursor without bypassing the owner-bearing step contract.

## 修正方針

Add F5nyy lossless bridge from F5nyx success to exactly one F5mt command-cursor step, preserving layer19 step errors and BeginFrame continuation while stopping before RunPending is stepped.

## 検証

Production-derived runtime fixture must prove BeginFrame descriptor metadata and continuation owner cleanup, with source policy, normal compile, trunk, CLI, regression, and subagent review gates.
