---
id: ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F
title: "Resource owner summaries reject fs and stdio read scratch owners after scanner boundary"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/owner_*.rs, stdlib/std/fs/fd.nepl, stdlib/std/fs/read.nepl, stdlib/std/stdio/read.nepl, tests/stdlib/kp.n.md"
---

# ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F: Resource owner summaries reject fs and stdio read scratch owners after scanner boundary

## 概要

After the Stage5 raw-memory boundary blockers are removed, tests/stdlib/kp.n.md reaches Resource IR owner checking and fails with resource.owner.maybe_leak for fs_open_with_flags fd_out_buf, fs_read_fd_bytes buf/iov_buf/nread_buf, stdio_read_all_bytes_result buf/iov/nread_ptr, and stdio_read_line_result buf/iov/nread.

## 対象

- `nepl-core/src/resource/owner_*.rs, stdlib/std/fs/fd.nepl, stdlib/std/fs/read.nepl, stdlib/std/stdio/read.nepl, tests/stdlib/kp.n.md`

## 根拠

- `tests/stdlib/kp.n.md` の Stage 5 raw-memory boundary blocker を除去した後、wasm runner は compile phase で `resource.owner.maybe_leak` まで進んだ。
- doctest#1/#4 では `fs_open_with_flags` の `fd_out_buf`、`fs_read_fd_bytes` の `buf` / `iov_buf` / `nread_buf`、`stdio_read_all_bytes_result` の `buf` / `iov` / `nread_ptr` が MaybeLeak になる。
- doctest#2 では `stdio_read_line_result` の `buf` / `iov` / `nread` が MaybeLeak になる。
- これらは private scratch storage または returned `ByteBuf` の owner transfer に関わるため、単に owner check を緩めると memory leak / double free を隠す。
- 既に stdio write scratch cleanup は `ISS-20260505T195142842Z-RESOURCE-OWNER-CHECKER-REPORTS-STDIO-00591700` で修正済みだが、read/fs 側の scratch と returned buffer owner contract は別の関数群で再発している。

## 問題

After the Stage5 raw-memory boundary blockers are removed, tests/stdlib/kp.n.md reaches Resource IR owner checking and fails with resource.owner.maybe_leak for fs_open_with_flags fd_out_buf, fs_read_fd_bytes buf/iov_buf/nread_buf, stdio_read_all_bytes_result buf/iov/nread_ptr, and stdio_read_line_result buf/iov/nread.

## 影響

KP/streamio doctests that use stdio or filesystem input remain blocked at compile time. These failures are memory-safety relevant because weakening owner checks would hide scratch buffer leaks, while the current checker may be unable to prove existing exhaustive cleanup.

## 修正方針

Audit fs/stdin read scratch ownership together with Resource IR owner summaries. Keep scratch owners consumed on every exit path, then either make the checker prove the existing match/loop cleanup or redesign the read boundary so private scratch storage is consumed with exact raw dealloc obligations.

## 検証

- `trunk build`
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_owner_read_scratch.json --runner wasm --no-tree -j 1 --assert-io`
- focused Rust owner regression for fs/stdio read scratch cleanup
