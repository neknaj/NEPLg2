---
id: ISS-20260719T113312984Z-F5NZZ-WEB-REGISTERED-BEGINFRAME-RETR-F01A2B02
title: "F5nzz Web registered BeginFrame retry executor"
area: gui-font
status: open
resolved: false
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

- 未記入

## 問題

F5nzy RetryReady is not connected to the existing Web host executor and registered F5nzl completion while preserving spent budget and prior diagnostics.

## 影響

Calling the existing Web step directly would lose registered continuation and could leave a repeated Web failure outside the exhausted retry policy.

## 修正方針

Consume RetryReady once in a platform/Web composition adapter, execute its exact pending action once, rejoin registered completion once, and route every recovered failure to typed abort using the carried exhausted budget.

## 検証

Actual default Web Unsupported host import reaches exhausted typed abort with old and new diagnostics preserved; source policy fixes single-call boundaries; normal compile, release/trunk/CLI gates, and subagent reviews pass.
