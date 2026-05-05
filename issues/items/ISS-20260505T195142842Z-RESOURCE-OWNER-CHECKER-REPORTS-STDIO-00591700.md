---
id: ISS-20260505T195142842Z-RESOURCE-OWNER-CHECKER-REPORTS-STDIO-00591700
title: "Resource owner checker reports stdio fd_write scratch MaybeLeak after duplicate import cleanup"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-06
target: "stdlib/std/stdio/raw.nepl, stdlib/std/stdio/write.nepl, nepl-core/tests/resource_ir.rs, nodesrc/test_stdlib_stdio_read_boundary.js, nepl-core/tests/kp.rs"
---

# ISS-20260505T195142842Z-RESOURCE-OWNER-CHECKER-REPORTS-STDIO-00591700: Resource owner checker reports stdio fd_write scratch MaybeLeak after duplicate import cleanup

## 概要

After identical imported definitions are no longer reprocessed, local_scanner_new_logic_debug reaches Resource IR owner checking and fails in stdio_write_fd_mem_result with resource.owner.maybe_leak for the iov and nwritten scratch owners on the fd_write loop boundary.

## 対象

- `nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_control.rs, stdlib/std/stdio/write.nepl, nepl-core/tests/kp.rs`

## 根拠

- `resource_ir_owner_check_accepts_stdio_fd_write_scratch_cleanup` を追加して、`stdio_write_fd_mem_result` の実呼び出し時に `iov` / `nwritten` scratch owner が `resource.owner.maybe_leak` になることを再現した。
- Resource IR dump では、旧実装の解放が `std_free(i32,i32)->()` wrapper 呼び出しとして現れていた。`std_free` は checked `dealloc` の `Err` arm を握りつぶして unit を返すため、owner checker は「解放に失敗した path」を安全に消せず、caller の scratch owner を MaybeLeak として保持していた。
- `stdio_write_fd_mem_result` の scratch は同一関数内で `alloc_ptr<u8>` 成功により確保され、サイズも固定であるため、private scratch owner invariant に基づき `dealloc_raw mem_ptr_addr ...` で終了前に必ず消費する設計が適切だった。

## 問題

After identical imported definitions are no longer reprocessed, local_scanner_new_logic_debug reaches Resource IR owner checking and fails in stdio_write_fd_mem_result with resource.owner.maybe_leak for the iov and nwritten scratch owners on the fd_write loop boundary.

## 影響

KP/scanner-style programs remain blocked before runtime. The failure suggests variant-gated owner cleanup or loop/match owner state around stdio fd_write scratch storage is still too conservative or the stdio write boundary needs an explicit Resource IR-safe ownership shape.

## 修正方針

Audit stdio_write_fd_mem_result together with Resource IR owner loop/match merge. Preserve the rule that scratch owners must be freed on all paths, then either make the checker prove the existing cleanup or redesign the stdio write boundary without weakening owner obligations.

## 対応

- `std/stdio/raw` から旧 `std_alloc` / `std_free` wrapper を削除した。checked dealloc の失敗を握りつぶす unit wrapper は owner obligation を静的に表現できず、stdio write boundary には不適切だった。
- `stdio_fd_write_mem` を追加し、`fd_write` の iovec / nwritten ABI 境界を `MemPtr<u8>` wrapper で受ける形に揃えた。
- `stdio_write_fd_mem_result` は `alloc_ptr<u8>` で `iov` / `nwritten` owner を確保し、raw store/load 用に `iov_raw` / `nwritten_raw` を分離した。
- `iov` / `nwritten` はこの関数だけが所有する scratch 領域として、すべての終了 path 前に `dealloc_raw` で消費するようにした。compiler 側の owner checker は弱めていない。
- `print_byte` も同じ private scratch 設計に揃え、`alloc_ptr` 成功後の 1 byte buffer を raw store / raw dealloc で扱うようにした。これにより stdio write doctest 側の `out` MaybeLeak も解消した。
- source policy は旧 `std_alloc` / `std_free` の存在を要求しない形へ更新し、代わりに `stdio_fd_write_mem` と private scratch owner cleanup を監視するようにした。

## 検証

Re-run cargo test -p nepl-core --test kp local_scanner_new_logic_debug -- --nocapture and focused owner Resource IR regressions. Add a regression that stdio_write_fd_mem_result does not report MaybeLeak for iov/nwritten when cleanup paths are exhaustive.

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_stdio_fd_write_scratch_cleanup -- --nocapture`: passed
- `cargo test -p nepl-core --test kp local_scanner_new_logic_debug -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check -- --nocapture`: passed
- `cargo fmt --check -p nepl-core`: passed
- `node nodesrc/tests.js -i stdlib/std/stdio/write.nepl --no-tree -o tmp/stdio-write-owner-cleanup.json -j 1`: `1 total / 1 passed`
- `node nodesrc/test_stdlib_stdio_read_boundary.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
