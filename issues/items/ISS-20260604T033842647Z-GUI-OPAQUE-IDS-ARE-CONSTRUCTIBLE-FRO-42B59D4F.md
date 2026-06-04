---
id: ISS-20260604T033842647Z-GUI-OPAQUE-IDS-ARE-CONSTRUCTIBLE-FRO-42B59D4F
title: "GUI opaque ids are constructible from invalid raw integers without typed validation"
area: stdlib
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-06-04
updated: 2026-06-05
target: "stdlib/std/gui/window.nepl, stdlib/std/gui/host.nepl, stdlib/platforms/gui/web/input.nepl"
---

# ISS-20260604T033842647Z-GUI-OPAQUE-IDS-ARE-CONSTRUCTIBLE-FRO-42B59D4F: GUI opaque ids are constructible from invalid raw integers without typed validation

## 概要

stdlib/std/gui/window.nepl exposes window_id, surface_id, and frame_id as raw i32 constructors. WindowId 0, negative ids, or stale host handles are representable as normal values. This conflicts with the Zenn policy of using static data types and Result/Option to make invalid states explicit.

## 対象

- `stdlib/std/gui/window.nepl, stdlib/std/gui/host.nepl, stdlib/platforms/gui/web/input.nepl`

## 根拠

- Zenn 記事の方針では、無効状態を文字列・数値・null 的な sentinel に潰さず、`Option` / `Result` と enum により静的検査可能な data contract として表す。
- GUI 文書では、platform boundary の raw host value は `std/gui` / `platforms/gui/*` 側で検証し、application model へ未検証 raw id を漏らさないことを要求している。

## 問題

stdlib/std/gui/window.nepl exposes window_id, surface_id, and frame_id as raw i32 constructors. WindowId 0, negative ids, or stale host handles are representable as normal values. This conflicts with the Zenn policy of using static data types and Result/Option to make invalid states explicit.

## 影響

Host backends and examples can accidentally carry invalid ids through event/effect/render pipelines, making unsupported or closed-window cases appear as ordinary commands.

## 修正方針

Add checked constructors such as window_id_result and surface_id_result returning Result, model absent default/headless windows with Option, and reserve raw constructors for platform-internal modules or documented test helpers. Add regular tests for 0, negative, valid roundtrip, closed window, and headless host cases.

## 検証

- `WindowId` / `SurfaceId` / `FrameId` に module-private proof field を追加し、raw value だけでは外部から構築できない形にした。
- `window_id_result` / `surface_id_result` / `frame_id_result` は 1 以上の raw id だけを `Result::Ok` とし、0 以下を `GuiError::InvalidCommand` として返す。
- `GuiHost.default_window` は `Option WindowId` にし、headless host は `Option::None` を保持する。
- Web input bridge は raw host window id を `window_id_result` で検証してから `GuiWebEvent` に格納する。
- `node nodesrc/test_stdlib_gui_opaque_id_contract.js`
- `node nodesrc/test_stdlib_gui_layering_policy.js`
- `node nodesrc/test_web_gui_shared_event_queue.js`
- `node nodesrc/test_web_gui_input_bridge.js`
- `node nodesrc/tests.js -i stdlib/std/gui.nepl -i stdlib/std/gui/window.nepl -i stdlib/std/gui/host.nepl -i stdlib/std/gui/runtime.nepl -i stdlib/platforms/gui/web/input.nepl -i tests/stdlib/gui_std.n.md -i tests/stdlib/gui_web_input.n.md --no-tree -o tmp/agent2-gui-opaque-id-tests -j 1 --dist web/dist --assert-io`
