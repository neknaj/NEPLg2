---
id: ISS-20260719T113312984Z-F5NZZ-WEB-REGISTERED-BEGINFRAME-RETR-F01A2B02
title: "F5nzz Web registered BeginFrame retry executor"
area: gui-font
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-07-19
updated: 2026-07-19
target: stdlib/platforms/gui/web/font_registered_begin_frame_retry_executor.nepl
---

# ISS-20260719T113312984Z-F5NZZ-WEB-REGISTERED-BEGINFRAME-RETR-F01A2B02: F5nzz Web registered BeginFrame retry executor

## 概要

F5nzy RetryReady is not connected to the existing Web host executor and registered F5nzl completion while preserving spent budget and prior diagnostics.

## 対象

- `stdlib/platforms/gui/web/font_registered_begin_frame_retry_executor.nepl`

## 根拠

- F5nzy `RetryReady` carried the exact pending action, registered dispatch continuation, selected support, prior failure context, and spent budget, but no platform adapter consumed that complete authority.
- The existing Web executor already owns raw imports and status mapping, while F5nzl owns the registered completion contract; bypassing either boundary would duplicate host policy or lose continuation.
- Actual Web `Unsupported` is an accepted action's failed outcome, so F5nzl/F5nzw correctly classify it as `DriverCompletionFailed` recovered state rather than a second support-preflight rejection.

## 問題

F5nzy RetryReady is not connected to the existing Web host executor and registered F5nzl completion while preserving spent budget and prior diagnostics.

## 影響

Calling the existing Web step directly would lose registered continuation and could leave a repeated Web failure outside the exhausted retry policy.

## 修正方針

Consume RetryReady once in a platform/Web composition adapter, execute its exact pending action once, rejoin registered completion once, and route every recovered failure to typed abort using the carried exhausted budget.

## 検証

Actual default Web Unsupported host import reaches the typed RecoveredState abort with old SinkRejected and new DriverCompletionFailed diagnostics preserved. A repeated RetryPending is separately forced through the carried exhausted budget to BudgetExhausted. Source policy fixes single-call boundaries; normal compile, release/trunk/CLI gates, and subagent reviews pass.

## 検証結果

- WASI/WASM actual Web import fixture: evidence 63, failed 0, exit 0.
- Web GUI/font source-policy and dedicated normal-mode test-only isolation passed.
- `cargo check --workspace`, wasm32 workspace check, release CLI build, release trunk build passed.
- Post-trunk Playground editor CLI JSON passed 13/13; issues check and diff check passed.
- Subagent diff re-review reported no blocker, major, minor, or cleanup leak.
