---
id: ISS-20260430T012423244Z-RESOURCE-IR-EXTERNAL-IO-WRITE-CALLS--3D90A2FD
title: "Resource IR external IO write calls do not validate initialized iovec input buffers"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/initialized_external_io.rs, nepl-core/src/resource/initialized.rs, nepl-core/tests/resource_ir.rs"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行
---

# ISS-20260430T012423244Z-RESOURCE-IR-EXTERNAL-IO-WRITE-CALLS--3D90A2FD: Resource IR external IO write calls do not validate initialized iovec input buffers

## 概要

Resource IR external IO summaries currently model fd_write/fd_pwrite as out-parameter writes for nwritten, but they do not require the iovec input buffers to be initialized before the external call reads from them.

## 対象

- `nepl-core/src/resource/initialized_external_io.rs, nepl-core/src/resource/initialized.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `ISS-20260429T234223901Z-KP-RAW-MEMORY-TESTS-FAIL-RESOURCE-IR-9B5964DA` の修正中、外部 IO effect は out pointer 初期化 (`fd_read` buffer、`nread`、`nwritten`) を CellState に反映するようになった。
- その一方で `fd_write` / `fd_pwrite` は外部へ buffer 内容を読む call であるにもかかわらず、iovec が指す raw cells の initialized state を検査していない。
- `fd_pwrite` の `nwritten` index 誤りは今回修正し、scalar offset を out pointer として扱わない regression を追加した。ただし、write input buffer 自体の initialized check は別途 Resource IR read-effect として設計する必要がある。
- この問題は `RawMemoryLoadCell` を弱める問題ではなく、external IO が raw memory を「読む」効果を Resource IR operation として表せていないことが原因である。

## 問題

Resource IR external IO summaries currently model fd_write/fd_pwrite as out-parameter writes for nwritten, but they do not require the iovec input buffers to be initialized before the external call reads from them.

## 影響

A program can pass an uninitialized raw buffer to fd_write/fd_pwrite without Resource IR reporting RawMemoryLoadCell, which weakens initialized-memory safety and can leak uninitialized memory contents to external IO.

## 修正方針

Add a typed external IO read-effect checker that walks iovec entries, requires each written input range to be initialized under the same CellState model used by raw loads, and reports a stable Resource IR diagnostic without weakening fd_read out-buffer summaries.

## 修正内容

- Resource IR の direct call 処理で、通常の引数所有権消費とは別に external IO が raw memory から読む cell を事前検査する段階を追加した。
- `fd_read` / `fd_pread` は iovec descriptor の buffer pointer / length cell を `RawMemoryLoadCell` として検査する。
- `fd_write` / `fd_pwrite` は iovec descriptor に加えて、descriptor が指す payload buffer の initialized state を同じ CellState モデルで検査する。
- external IO input precondition が満たされない場合は call output と out-parameter side effect を初期化済みに進めず、通常の raw load と同じ `CellUnavailable` 診断として報告する。
- `RawCellAddressAliases` に tracked places と raw-address value 判定を寄せ、raw memory checker と external IO checker が同じ alias 判断を使うようにした。
- `raw_memory_unknown_offset_cell_place` を place utility に移し、raw fill と external IO payload read が同じ unknown-offset cell 表現を使うようにした。

## 検証

Add Resource IR regressions for fd_write/fd_pwrite with initialized and uninitialized iovec buffers. The initialized fixture must pass and the uninitialized fixture must report a cell diagnostic for the buffer range.

## 検証結果

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_fd_write -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_fd_read -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_fd_pwrite -- --nocapture`: 1 passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 137 passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/tests.js -i stdlib/std/stdio.nepl --no-tree -o tmp/stdio-external-io-write-check.json -j 2`: 28 passed
