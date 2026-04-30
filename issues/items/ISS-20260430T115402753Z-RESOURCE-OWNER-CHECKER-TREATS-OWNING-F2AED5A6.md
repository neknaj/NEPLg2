---
id: ISS-20260430T115402753Z-RESOURCE-OWNER-CHECKER-TREATS-OWNING-F2AED5A6
title: "Resource owner checker treats owning raw local reads as non-owning views"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_raw_view.rs, nepl-core/tests/resource_ir.rs, nepl-core/tests/kp.rs"
---

# ISS-20260430T115402753Z-RESOURCE-OWNER-CHECKER-TREATS-OWNING-F2AED5A6: Resource owner checker treats owning raw local reads as non-owning views

## 概要

Resource owner checker inferred non-owning raw address views from alias shape alone. A read temporary aliasing an owning raw local could therefore be classified as a view when it was constructed into a returned aggregate, leaving the original local owner live and reporting a false owner leak.

## 対象

- `nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_raw_view.rs, nepl-core/tests/resource_ir.rs, nepl-core/tests/kp.rs`

## 根拠

- `make_pair` が `alloc_raw` で得た raw owner local を `RawPair left right` に詰めて返す経路で、旧 checker は alias shape だけを見て `left`/`right` を non-owning view と推定していた。
- その結果、aggregate field への owner transfer が起きず、元 local 側に free obligation が残ったまま関数末尾で leak と判定される。
- 一方で `mem_ptr_addr p` のような `MemPtr` projection は non-owning raw address view として扱う必要があるため、raw `i32` alias を一律 owner として扱う修正も不正確である。

## 問題

Resource owner checker inferred non-owning raw address views from alias shape alone. A read temporary aliasing an owning raw local could therefore be classified as a view when it was constructed into a returned aggregate, leaving the original local owner live and reporting a false owner leak.

## 影響

Self-host scanner/header style functions that allocate raw buffers and return an aggregate of owner handles are rejected by Resource IR owner gate. The same heuristic also risks hiding real owner transfer boundaries behind raw alias shape instead of the explicit RawAddressView IR marker.

## 修正方針

Make non-owning view classification depend on the explicit RawAddressView table, propagate view markers through aggregate/value copies, and keep owning raw local reads transferable into aggregate fields.

## 検証

- `cargo fmt --check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_transfers_raw_local_reads_into_returned_aggregate -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_preserves_alloc_ptr_raw_owner_return -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_result_ok_mem_ptr_dealloc_consumption -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_allows_region_ptr -- --nocapture`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test kp local_scanner_new_logic_debug -- --nocapture`: `stdio_write_fd_mem_result` の `iov` / `nwritten` が `std_free` 経由で MaybeLeak になる別問題を検出した。raw owner/view 分離の回帰ではなく、checked `dealloc` の Err branch を握りつぶす `std_free` の設計問題として次 issue で扱う。
