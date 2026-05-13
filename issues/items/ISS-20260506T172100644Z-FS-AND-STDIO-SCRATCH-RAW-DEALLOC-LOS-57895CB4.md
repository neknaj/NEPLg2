---
id: ISS-20260506T172100644Z-FS-AND-STDIO-SCRATCH-RAW-DEALLOC-LOS-57895CB4
title: "fs and stdio scratch raw dealloc lose free obligation after dynamic range blocker"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-13
target: "nepl-core/src/resource/owner_*.rs, stdlib/std/fs/*.nepl, stdlib/std/stdio/*.nepl, nodesrc/test_stdlib_*boundary.js, nepl-core/tests/resource_ir.rs"
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

## 2026-05-13 再オープン

`cargo test -p nepl-core --test functions function_purity_check_impure_calls_pure -- --nocapture` が、clean main (`275e9bdb`) と import visibility commit 後の main (`941a5249`) の両方で `stdio_write_fd_mem_result__i32_MemPtr_T_u8_i32__Result_T_E_unit_StdErrorKind__imp` の Resource IR owner diagnostics により失敗した。

現在の診断は、`iov` / `nwritten` scratch の `dealloc_raw` に対する `resource.owner.no_free_obligation` と、同じ scratch owner の `resource.owner.maybe_leak` が同時に出ている。2026-05-07 の修正で `kp` / focused Resource IR regression は通っていたが、`functions` 経由の compile pipeline では scratch owner の free obligation がまだ exact dealloc temporary へ接続されていない経路が残っている。

この issue は再オープンし、Resource IR owner checker を弱めずに、`alloc_ptr` で確保された private scratch owner が `MemPtr` wrapper、raw address temporary、loop/match return path を通っても同じ free obligation として追跡されるように根本原因を調査する。必要なら `stdlib/std/stdio/write/fd.nepl` の scratch cleanup shape を再設計するが、compiler 側の owner obligation / MaybeLeak 診断を握りつぶしてはならない。

追加検証対象:

- `cargo test -p nepl-core --test functions function_purity_check_impure_calls_pure -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_stdio_fd_write_scratch_cleanup -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup -- --nocapture`
- `cargo test -p nepl-core --test kp kpread_to_kpwrite_prefixsum_i32 -- --nocapture`

## 2026-05-13 修正

再調査で、`mem_ptr_addr` が Resource IR 上で non-owning raw address view になった後も、fs/stdio の scratch cleanup が raw address へ free obligation を移せる前提を一部残していたことを確認した。これは `NoFreeObligation` を握りつぶすべきではなく、stdlib 側の cleanup contract を「所有 `MemPtr` を `dealloc_ptr` に渡す」形へ揃える必要がある。

同時に、`realloc_ptr` などの summary effect が `MemPtr` 内部の raw view を経由して「明示的に返却 owner へ移る」ケースと、通常の non-owning raw view を誤って owner として扱うケースを分離できていなかった。修正では通常 transfer は raw view を非所有のまま維持し、関数 summary の明示的 owner effect に限って raw view 経由の元 owner 解決を許可した。

ResourceIR 側では、assignment/store overwrite 時の live owner preservation が単純に value root だけを見る実装だったため、`set p grown` のように target 配下の live field と replacement 側 alias が同じ storage を指すケースを leak と誤認する可能性があった。`target` から overwritten place への suffix を取り、replacement 側の同一 field を owner alias 解決して storage id を比較するようにした。

stdlib 側では、`fs_open_with_flags`、`fs_read_fd_bytes`、`fs_finish_read_buffer`、`stdio_read_all_bytes_result`、`stdio_finish_read_buffer`、`stdio_write_fd_mem_result`、`print_byte`、`stdio_read_line_result` の scratch cleanup を raw dealloc から typed `dealloc_ptr` に移行した。error path で owner cleanup と `Result::Err` 構築を同じ ownership boundary に閉じるため、`fs_discard_read_buffer` と `stdio_discard_read_buffer` を追加した。

source policy も、旧 raw cleanup 前提から `MemPtr = non-owning pointer` 方針に合わせて更新した。`fs_finish_read_buffer` / `stdio_write_fd_mem_result` は `dealloc_raw mem_ptr_addr ...` ではなく typed owner cleanup を要求し、fs の `unreachable` は `dealloc_ptr` Err branch の owner-cleanup invariant に限って許可する。

検証:

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_loop_realloc_owner_replacement -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_realloc_owner_replacement_assignment -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_stdio_fd_write_scratch_cleanup -- --nocapture`: passed
- `cargo test -p nepl-core --test functions function_purity_check_impure_calls_pure -- --nocapture`: passed
- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`: passed
- `node nodesrc/test_stdlib_stdio_read_boundary.js`: passed

## 関連

- [ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F](./ISS-20260506T130126516Z-RESOURCE-OWNER-SUMMARIES-REJECT-FS-A-7E58243F.md)
- [ISS-20260506T172012873Z-RESOURCE-IR-DYNAMIC-RAW-ADDRESS-VIEW-77E94B53](./ISS-20260506T172012873Z-RESOURCE-IR-DYNAMIC-RAW-ADDRESS-VIEW-77E94B53.md)
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
