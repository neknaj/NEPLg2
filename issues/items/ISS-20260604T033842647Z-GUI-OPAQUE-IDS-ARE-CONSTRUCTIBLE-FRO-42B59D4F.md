---
id: ISS-20260604T033842647Z-GUI-OPAQUE-IDS-ARE-CONSTRUCTIBLE-FRO-42B59D4F
title: "GUI opaque ids are constructible from invalid raw integers without typed validation"
area: stdlib
status: open
resolved: false
priority: P2
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/std/gui/window.nepl, stdlib/std/gui/host.nepl, stdlib/platforms/gui/web/input.nepl"
---

# ISS-20260604T033842647Z-GUI-OPAQUE-IDS-ARE-CONSTRUCTIBLE-FRO-42B59D4F: GUI opaque ids are constructible from invalid raw integers without typed validation

## 概要

stdlib/std/gui/window.nepl exposes window_id, surface_id, and frame_id as raw i32 constructors. WindowId 0, negative ids, or stale host handles are representable as normal values. This conflicts with the Zenn policy of using static data types and Result/Option to make invalid states explicit.

## 対象

- `stdlib/std/gui/window.nepl, stdlib/std/gui/host.nepl, stdlib/platforms/gui/web/input.nepl`

## 根拠

- 未記入

## 問題

stdlib/std/gui/window.nepl exposes window_id, surface_id, and frame_id as raw i32 constructors. WindowId 0, negative ids, or stale host handles are representable as normal values. This conflicts with the Zenn policy of using static data types and Result/Option to make invalid states explicit.

## 影響

Host backends and examples can accidentally carry invalid ids through event/effect/render pipelines, making unsupported or closed-window cases appear as ordinary commands.

## 修正方針

Add checked constructors such as window_id_result and surface_id_result returning Result, model absent default/headless windows with Option, and reserve raw constructors for platform-internal modules or documented test helpers. Add regular tests for 0, negative, valid roundtrip, closed window, and headless host cases.

## 検証

Run GUI source policies, focused std/gui doctests, and future cfg-test-style host id validation tests.
