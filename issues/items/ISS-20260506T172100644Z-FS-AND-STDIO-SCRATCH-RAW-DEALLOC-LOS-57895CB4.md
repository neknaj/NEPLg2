---
id: ISS-20260506T172100644Z-FS-AND-STDIO-SCRATCH-RAW-DEALLOC-LOS-57895CB4
title: "fs and stdio scratch raw dealloc lose free obligation after dynamic range blocker"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/owner_*.rs, stdlib/std/fs/*.nepl, stdlib/std/stdio/*.nepl, nepl-core/tests/kp.rs"
---

# ISS-20260506T172100644Z-FS-AND-STDIO-SCRATCH-RAW-DEALLOC-LOS-57895CB4: fs and stdio scratch raw dealloc lose free obligation after dynamic range blocker

## 概要

After the dynamic range CellState blocker is fixed, cargo test -p nepl-core --test kp kpread_to_kpwrite_prefixsum_i32 reaches Resource IR owner checking and fails with resource.owner.no_free_obligation for fs_open_with_flags fd_out_buf, fs_read_fd_bytes iov_buf/nread_buf, stdio_read_all_bytes_result iov/nread_ptr, and stdio_write_fd_mem_result iov/nwritten scratch dealloc.

## 対象

- `nepl-core/src/resource/owner_*.rs, stdlib/std/fs/*.nepl, stdlib/std/stdio/*.nepl, nepl-core/tests/kp.rs`

## 根拠

- `cargo test -p nepl-core --test kp kpread_to_kpwrite_prefixsum_i32 -- --nocapture` で、dynamic range の `resource.cell.uninit` 修正後に本 issue の owner diagnostics へ到達した。
- 既存 `ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F` は `MaybeLeak` を対象に fixed になっているが、現在の失敗は exact scratch dealloc 時の `NoFreeObligation` であり、owner state / storage origin / stdlib cleanup contract の別経路で再発している。
- 対象関数はいずれも WASI out-pointer / iovec scratch を扱うため、Resource IR owner gate を緩めると private scratch leak や lost owner を見逃す。

## 問題

After the dynamic range CellState blocker is fixed, cargo test -p nepl-core --test kp kpread_to_kpwrite_prefixsum_i32 reaches Resource IR owner checking and fails with resource.owner.no_free_obligation for fs_open_with_flags fd_out_buf, fs_read_fd_bytes iov_buf/nread_buf, stdio_read_all_bytes_result iov/nread_ptr, and stdio_write_fd_mem_result iov/nwritten scratch dealloc.

## 影響

KP and scanner-style WASI programs remain blocked at compile time. Treating NoFreeObligation as harmless would hide real double-free/lost-owner bugs, so this must be fixed by preserving exact scratch storage ownership or correcting the stdlib cleanup contract.

## 修正方針

Audit whether the regression is in Resource IR owner alias/release summaries or in stdlib scratch ownership after recent module splits. Preserve private scratch owner consumption on all paths, and add focused owner regressions for fs_open_with_flags, fs_read_fd_bytes, stdio_read_all_bytes_result, and stdio_write_fd_mem_result.

## 検証

cargo test -p nepl-core --test kp kpread_to_kpwrite_prefixsum_i32 -- --nocapture

## 2026-05-07 修正

根本原因は stdlib scratch cleanup ではなく、Resource IR の raw owner alias 不変条件にあった。

`alloc_raw` / `alloc_ptr` の戻り値は一度 temporary に `RawMemory::Alloc` として現れ、その後 `let p ...` や `Result::Ok p` の payload へ owner transfer される。`RawCellAddressAliases::move_owner_aliases` は owner mark を移していたが、raw owner value 自身の alias group を再作成していなかった。そのため後続の `Read %p -> tmp` が「追跡中 raw owner の exact copy」として扱われず、`dealloc_raw tmp` が `%p` の free obligation へ解決できなかった。

修正では、owner alias move 後も moved target と moved marked projection を alias group に戻すようにした。これにより、通常 i32 copy は引き続き raw alias group を seed しない一方で、compiler が owner mark 済みの raw storage root だけは local read / enum payload bind / struct field move 後も exact owner alias として追跡される。

回帰として `resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup` を追加し、以下の関数で private scratch owner diagnostic が出ないことを固定した。

- `fs_open_with_flags__`
- `fs_read_fd_bytes__`
- `stdio_read_all_bytes_result__`
- `stdio_write_fd_mem_result__`

追加確認:

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_deallocated_alloc -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_allows_raw_pointer_read_before_dealloc -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_stdio_fd_write_scratch_cleanup -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_reports_stale_owned_alias_dealloc_after_free -- --nocapture`: passed
- `cargo test -p nepl-core --test kp kpread_to_kpwrite_prefixsum_i32 -- --nocapture`: passed

## 関連

- [ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F](./ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F.md)
- [ISS-20260506T172012873Z-RESOURCE-IR-DYNAMIC-RAW-ADDRESS-VIEW-77E94B53](./ISS-20260506T172012873Z-RESOURCE-IR-DYNAMIC-RAW-ADDRESS-VIEW-77E94B53.md)
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
