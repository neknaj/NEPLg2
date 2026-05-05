---
id: ISS-20260505T234127273Z-INITIALIZED-EXTERNAL-IO-MIXES-INPUT--B1A15983
title: "Initialized external IO mixes input checks and effects"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-06
target: "nepl-core/src/resource/initialized_external_io.rs, nepl-core/src/resource/initialized_external_io_input.rs, nepl-core/src/resource/mod.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260505T234127273Z-INITIALIZED-EXTERNAL-IO-MIXES-INPUT--B1A15983: Initialized external IO mixes input checks and effects

## 概要

After typed host operations were added, initialized_external_io.rs contains both external IO input availability checks and initialized-state effect application. The file is 156 lines while the resource responsibility policy limit is 140.

## 対象

- `nepl-core/src/resource/initialized_external_io.rs, nepl-core/src/resource/initialized_external_io_input.rs, nepl-core/src/resource/mod.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `ISS-20260505T233519789Z` の effect count 分割後、`node nodesrc/test_resource_checker_responsibility.js` が `initialized_external_io.rs has 156 lines; responsibility split limit is 140` を報告した。
- `initialized_external_io.rs` は `ensure_external_io_initialized_inputs` による読み取り前提条件チェックと、`apply_external_io_initialized_effect` による initialized-state effect application を同じ file に持っていた。
- fd_read/fd_write/fd_pread/fd_pwrite の iovec precondition は、出力 pointer/buffer の initialized marking とは独立した責務である。

## 問題

After typed host operations were added, initialized_external_io.rs contains both external IO input availability checks and initialized-state effect application. The file is 156 lines while the resource responsibility policy limit is 140.

## 影響

The source policy reports initialized_external_io.rs has 156 lines; responsibility split limit is 140. Keeping input validation and state mutation in one file makes initialized external IO safety harder to audit as more host operations are added.

## 修正方針

Move external IO input precondition checks into initialized_external_io_input.rs, keep initialized_external_io.rs focused on applying initialized-state effects, and add the new module to responsibility policy checks.

## 検証

- `cargo fmt -p nepl-core`: 実行済み
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_fd -- --nocapture`: `4 passed`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_external_fd_read_initializes_iovec_buffers -- --nocapture`: passed

## 対応

- `ensure_external_io_initialized_inputs` を `initialized_external_io_input.rs` に分離した。
- `initialized_external_io.rs` は typed host operation の initialized-state effect application に集中させた。
- `resource/mod.rs` と `nodesrc/test_resource_checker_responsibility.js` に新 module を追加し、input precondition check が再び effect application file に戻らないようにした。
