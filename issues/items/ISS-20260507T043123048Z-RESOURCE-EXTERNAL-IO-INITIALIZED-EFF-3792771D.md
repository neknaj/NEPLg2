---
id: ISS-20260507T043123048Z-RESOURCE-EXTERNAL-IO-INITIALIZED-EFF-3792771D
title: "Resource external IO initialized effect module escapes responsibility policy"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/initialized_external_io*.rs; nodesrc/test_resource_checker_responsibility.js"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行
---

# ISS-20260507T043123048Z-RESOURCE-EXTERNAL-IO-INITIALIZED-EFF-3792771D: Resource external IO initialized effect module escapes responsibility policy

## 概要

initialized_external_io_effect.rs is declared in resource/mod.rs but is not registered in the resource checker responsibility policy, and it mixes iovec descriptor availability, iovec layout discovery, fd_read initialized effects, and generic raw-cell initialization helpers in one 258-line module.

## 対象

- `nepl-core/src/resource/initialized_external_io*.rs; nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

initialized_external_io_effect.rs is declared in resource/mod.rs but is not registered in the resource checker responsibility policy, and it mixes iovec descriptor availability, iovec layout discovery, fd_read initialized effects, and generic raw-cell initialization helpers in one 258-line module.

## 影響

External I/O initialized-cell proof is memory-safety-critical for fd_read/fd_write buffers. If this module remains unmonitored and monolithic, iovec proof changes can bypass source policy and hide future Resource IR range/initialization regressions. The split also exposed that `fd_write` still looked for the old unknown-offset initialized Copy cell and did not accept the newer typed initialized range facts produced by `fill_i32`.

## 修正方針

Register every initialized_external_io module in the responsibility policy and split iovec descriptor/layout proof from fd_read initialized-effect application without changing Resource IR semantics.

## 解決内容

- `initialized_external_io_effect.rs` を fd_read/fd_pread/fd_write などの initialized-effect application に絞った。
- iovec descriptor/payload input check を `initialized_external_io_iov.rs` へ分離した。
- iovec の buffer pointer cell / length cell / descriptor address layout helper を `initialized_external_io_iov_layout.rs` へ分離した。
- `cell_state_raw_range_count.rs` を追加し、fd_write payload check が unknown-offset cell だけでなく、base address と iovec length に対応する typed initialized raw range も受け取れるようにした。
- `nodesrc/test_resource_checker_responsibility.js` に `initialized_external_io_effect.rs`、`initialized_external_io_iov.rs`、`initialized_external_io_iov_layout.rs`、`cell_state_raw_range_count.rs` の存在、`mod` 宣言、行数上限を追加した。

## 検証

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir fd_read -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir fd_write -- --nocapture`: passed
