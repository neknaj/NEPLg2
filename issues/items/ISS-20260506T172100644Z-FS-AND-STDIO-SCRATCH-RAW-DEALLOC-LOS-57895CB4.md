---
id: ISS-20260506T172100644Z-FS-AND-STDIO-SCRATCH-RAW-DEALLOC-LOS-57895CB4
title: "fs and stdio scratch raw dealloc lose free obligation after dynamic range blocker"
area: core
status: open
resolved: false
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

## 関連

- [ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F](./ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F.md)
- [ISS-20260506T172012873Z-RESOURCE-IR-DYNAMIC-RAW-ADDRESS-VIEW-77E94B53](./ISS-20260506T172012873Z-RESOURCE-IR-DYNAMIC-RAW-ADDRESS-VIEW-77E94B53.md)
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
