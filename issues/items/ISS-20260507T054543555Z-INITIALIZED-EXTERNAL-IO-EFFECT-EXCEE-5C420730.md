---
id: ISS-20260507T054543555Z-INITIALIZED-EXTERNAL-IO-EFFECT-EXCEE-5C420730
title: "initialized_external_io_effect exceeds responsibility split limit after fd_read bounded ranges"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_external_io_effect.rs, nepl-core/src/resource/initialized_external_io_payload.rs, nodesrc/test_resource_checker_responsibility.js"
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

## 2026-05-07 修正結果

`fd_read` / `fd_pread` の external I/O effect handling から、single-iov payload range を `nread` 境界付き initialized byte range として構築する責務を `initialized_external_io_payload.rs` へ分離した。

`initialized_external_io_effect.rs` は nread out cell の exact initialization と external I/O effect の entry point に戻し、iovec descriptor cell の探索、single-iov 判定、payload alias filtering、`InitializedRawRangeUnit::Bytes` の range 登録は新 module 側に閉じた。line limit は緩めていない。

確認:

- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_fd_read -- --nocapture`: 3 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_aggregate -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test kp local_scanner_new_logic_debug -- --nocapture`: passed
