---
id: ISS-20260507T054543555Z-INITIALIZED-EXTERNAL-IO-EFFECT-EXCEE-5C420730
title: "initialized_external_io_effect exceeds responsibility split limit after fd_read bounded ranges"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_external_io_effect.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260507T054543555Z-INITIALIZED-EXTERNAL-IO-EFFECT-EXCEE-5C420730: initialized_external_io_effect exceeds responsibility split limit after fd_read bounded ranges

## 概要

After fd_read/fd_pread bounded payload range modeling, initialized_external_io_effect.rs has grown past the responsibility split limit. Source policy now reports initialized_external_io_effect.rs has 115 lines while the limit is 90 once earlier raw range file limits are split.

## 対象

- `nepl-core/src/resource/initialized_external_io_effect.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After fd_read/fd_pread bounded payload range modeling, initialized_external_io_effect.rs has grown past the responsibility split limit. Source policy now reports initialized_external_io_effect.rs has 115 lines while the limit is 90 once earlier raw range file limits are split.

## 影響

External I/O initialization is memory-safety critical. Keeping iovec descriptor parsing, bounded payload range creation, and nread exact-cell initialization in one file makes the Resource IR fd_read model harder to audit.

## 修正方針

Split bounded iovec payload range construction from initialized_external_io_effect.rs into a narrower helper module. Keep the line limit instead of raising it.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js after the split, plus fd_read Resource IR regressions.
