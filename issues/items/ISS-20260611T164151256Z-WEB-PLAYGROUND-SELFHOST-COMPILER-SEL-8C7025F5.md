---
id: ISS-20260611T164151256Z-WEB-PLAYGROUND-SELFHOST-COMPILER-SEL-8C7025F5
title: "Web Playground selfhost compiler selector needs artifact capability gating"
area: tools
status: open
resolved: false
priority: P1
type: bug
created: 2026-06-11
updated: 2026-06-11
target: "web/index.html; web/src/main.ts; web/src/terminal/shell.ts; web/src/runtime/worker.ts; nepl-web/src/lib.rs"
---

# ISS-20260611T164151256Z-WEB-PLAYGROUND-SELFHOST-COMPILER-SEL-8C7025F5: Web Playground selfhost compiler selector needs artifact capability gating

## 概要

The UI and shell accept selfhost compiler mode even when the current web artifact exports no runnable selfhost compile_outputs_with_vfs API. Failure occurs only after worker compile starts.

## 対象

- `web/index.html; web/src/main.ts; web/src/terminal/shell.ts; web/src/runtime/worker.ts; nepl-web/src/lib.rs`

## 根拠

- 未記入

## 問題

The UI and shell accept selfhost compiler mode even when the current web artifact exports no runnable selfhost compile_outputs_with_vfs API. Failure occurs only after worker compile starts.

## 影響

The Playground advertises a mode that currently fails deterministically, and users cannot distinguish unavailable capability from compile failure.

## 修正方針

Expose compiler capabilities from nepl-web or compiler asset metadata. Disable/hide selfhost in UI until available, and make shell --compiler selfhost fail before spawning compile worker with a stable capability error.

## 検証

Add tests for artifacts with and without selfhost capability, UI disabled/enabled state, and shell --compiler selfhost behavior.
