---
id: ISS-20260514T041207146Z-STREAMWRITER-CLOSE-IS-NOT-EXPORTED-T-ED15EF36
title: "StreamWriter close is not exported through streamio writer facade"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/std/streamio/writer.nepl, stdlib/std/streamio/writer/state.nepl, tests/stdlib/{streamio,kp,kp_i64}.n.md"
---

# ISS-20260514T041207146Z-STREAMWRITER-CLOSE-IS-NOT-EXPORTED-T-ED15EF36: StreamWriter close is not exported through streamio writer facade

## 概要

StreamWriter close is defined in std/streamio/writer/state, but std/streamio/writer does not expose a root public close overload. Users importing std/streamio or std/streamio/writer cannot resolve |> close for StreamWriter even though write/flush are public facade APIs.

## 対象

- `stdlib/std/streamio/writer.nepl, stdlib/std/streamio/writer/state.nepl, tests/stdlib/{streamio,kp,kp_i64}.n.md`

## 根拠

- 未記入

## 問題

StreamWriter close is defined in std/streamio/writer/state, but std/streamio/writer does not expose a root public close overload. Users importing std/streamio or std/streamio/writer cannot resolve |> close for StreamWriter even though write/flush are public facade APIs.

## 影響

streamio and kp doctests that use buffered writer cannot compile. The public writer API is incomplete and ownership cleanup for StreamWriter is not available through the facade boundary.

## 修正方針

Move/rename the internal state cleanup to an explicit implementation helper and expose a root pub close overload in std/streamio/writer that consumes StreamWriter. Add source/doctest coverage that open/write/flush/close all resolve through the facade.

## 検証

Run writer root doctest and focused streamio/kp writer doctests.
