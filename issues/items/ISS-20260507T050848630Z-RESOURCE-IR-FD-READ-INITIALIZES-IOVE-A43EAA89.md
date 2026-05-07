---
id: ISS-20260507T050848630Z-RESOURCE-IR-FD-READ-INITIALIZES-IOVE-A43EAA89
title: "Resource IR fd_read initializes iovec payloads as unbounded unknown cells"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_external_io*.rs, nepl-core/src/resource/cell_state_raw_range*.rs, nepl-core/src/resource/initialized_summary_byte_ranges.rs, nepl-core/tests/resource_ir.rs, nepl-core/tests/kp.rs"
source: "doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行"
---

# ISS-20260507T050848630Z-RESOURCE-IR-FD-READ-INITIALIZES-IOVE-A43EAA89: Resource IR fd_read initializes iovec payloads as unbounded unknown cells

## 概要

fd_read writes at most the reported nread bytes, but Resource IR records the whole iovec payload as initialized. A load guarded only by i < iov_len can pass even when i may be beyond nread.

## 対象

- `nepl-core/src/resource/initialized_external_io*.rs, nepl-core/src/resource/cell_state_raw_range*.rs, nepl-core/src/resource/initialized_summary_byte_ranges.rs, nepl-core/tests/resource_ir.rs, nepl-core/tests/kp.rs`

## 根拠

apply_iov_read_buffers_initialized marks raw_memory_unknown_offset_cell_place for fd_read/fd_pread payload buffers instead of a range bounded by nread.

## 問題

fd_read writes at most the reported nread bytes, but Resource IR records the whole iovec payload as initialized. A load guarded only by i < iov_len can pass even when i may be beyond nread.

## 影響

Scanner and external IO code can accidentally rely on bytes that the host did not initialize. This weakens memory-safety guarantees and conflicts with the typed range model used by fill_u8/fill_i32.

## 修正方針

For single-iov fd_read/fd_pread, mark the payload as InitializedRawByteRange with count equal to the nread raw cell, and require positive/range guard facts before loads. Do not mark an unbounded unknown-offset cell.

## 検証

Add Resource IR regressions accepting nread-guarded payload loads and rejecting unguarded or capacity-guard-only payload loads.

## 2026-05-07 修正

`fd_read` / `fd_pread` の iovec payload 初期化を、unknown-offset の全域 initialized cell ではなく、`nread` raw cell を count に持つ `InitializedRawByteRange` として記録するようにした。

修正内容:

- single-iov の場合だけ payload buffer に `nread` bounded byte range を追加する。
- capacity guard だけでは payload load を通さず、`0 <= i && i < nread` の relation fact がある場合だけ symbolic load を許可する。
- `fd_read` は既存の初期化済み range を破壊せず、host が書いた prefix range を追加する。これにより事前 `memset_u8` などの range fact は維持される。
- direct base load は count が literal positive、または count place に Positive fact がある場合だけ許可する。
- `fd_pread` も同じ `nread` bounded path を使う。

追加した回帰:

- `resource_ir_cell_check_fd_read_accepts_payload_load_guarded_by_nread`
- `resource_ir_cell_check_fd_read_rejects_payload_load_guarded_only_by_capacity`

関連して確認した残件:

- returned struct field projection をまたぐ initialized range summary はまだ完全ではなく、`ISS-20260507T052014018Z-RESOURCE-IR-RETURNED-AGGREGATE-FIELD-F78CD903` に分離した。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_fd_read -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_external_fd_read_initializes_nread_cell -- --nocapture`: passed
- `cargo test -p nepl-core --test kp wasi_fd_read_raw_iovec_debug -- --nocapture`: passed
- `cargo test -p nepl-core --test kp wasi_fd_read_raw_iovec_with_dealloc_debug -- --nocapture`: passed
- `cargo test -p nepl-core --test kp wasi_fd_read_then_alloc_header_debug -- --nocapture`: passed
- `cargo test -p nepl-core --test kp local_scanner_new_logic_debug -- --nocapture`: passed
- `cargo test -p nepl-core --test kp -- --nocapture`: 14 passed
