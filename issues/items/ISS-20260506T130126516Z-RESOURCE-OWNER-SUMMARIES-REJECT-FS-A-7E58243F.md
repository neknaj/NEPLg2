---
id: ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F
title: "Resource owner summaries reject fs and stdio read scratch owners after scanner boundary"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/resource/owner_*.rs, stdlib/std/fs/fd.nepl, stdlib/std/fs/read.nepl, stdlib/std/stdio/read.nepl, tests/stdlib/kp.n.md"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行
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

## 2026-05-06 修正

原因は二つに分かれていた。

- `fs_open_with_flags` / `fs_read_fd_bytes` / `stdio_read_all_bytes_result` / `stdio_read_line_result` の private scratch cleanup が checked `dealloc_ptr` の `Err` を握りつぶす形になっており、Resource IR からは free obligation が全経路で閉じたと証明できなかった。
- `Result::Ok(ByteBuf)` のように owner を payload として返す branch / helper return では、call summary の pending owner effect が result value に残ったまま branch / return 境界を越え、source local 側に owner が残存したように見えていた。

修正内容:

- private scratch storage は allocation success 後に compiler-owned raw boundary 内で必ず解放する内部不変条件を持つため、checked `dealloc_ptr` ではなく exact `dealloc_raw` で閉じる形に統一した。
- `set p grown` のような同一 storage handle への owner-preserving replacement は leak ではないため、owner replacement 判定で同じ `StorageId` の live replacement を許可した。
- pending `Result` owner effect を branch value / match arm value / function return value の境界で materialize し、payload へ owner transfer を反映してから外側状態へ渡すようにした。
- unconditional summary で既に消費済みの引数を variant-conditioned consumption が再消費しないようにし、checked cleanup helper の二重 move 診断を防いだ。

検証:

- `cargo fmt --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir "resource_ir_owner_check_" -- --nocapture`: 54 passed
- `trunk build`: passed
- `node nodesrc/tests.js -i stdlib/std/fs/read.nepl -i stdlib/std/fs/raw.nepl -i stdlib/std/fs/fd.nepl -i stdlib/std/stdio/read.nepl -i stdlib/std/stdio/read/buffer.nepl -o output/read_owner_raw_cleanup_targeted.json --runner wasm --no-tree -j 1 --assert-io`: total=8, passed=8
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_owner_read_scratch_after_fix.json --runner wasm --no-tree -j 1 --assert-io`: fs/stdio read scratch owner leak は消滅。残りは `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` の dynamic range summary と `ISS-20260506T134653279Z-RESOURCE-OWNER-SUMMARY-MISSES-RAW-DE-007EB7EA` の `unwrap_ok dealloc` summary、float doctest performance residual。

remote main `232715b8` / `7b6afed3` 取り込み後の再実行では、`alloc/string/access.nepl` 分割に伴う `len__str` / `string_byte_at_unchecked` の raw-memory-boundary capability 追従漏れが先に発火した。この blocker は `ISS-20260506T135746003Z-STRING-ACCESS-SPLIT-LOSES-RAW-MEMORY-8C64A912` として分離した。
