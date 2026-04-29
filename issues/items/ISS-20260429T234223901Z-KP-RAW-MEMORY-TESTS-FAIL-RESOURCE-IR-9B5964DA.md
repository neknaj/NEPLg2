---
id: ISS-20260429T234223901Z-KP-RAW-MEMORY-TESTS-FAIL-RESOURCE-IR-9B5964DA
title: "kp raw memory tests fail Resource IR initialized checks"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nepl-core/tests/kp.rs, nepl-core/src/resource"
---

# ISS-20260429T234223901Z-KP-RAW-MEMORY-TESTS-FAIL-RESOURCE-IR-9B5964DA: kp raw memory tests fail Resource IR initialized checks

## 概要

nepl-core tests/kp.rs fails after the current Resource IR gate because direct WASI fd_read and dynamic raw-memory prefix-sum fixtures read buffers that Resource IR still sees as Uninit.

## 対象

- `nepl-core/tests/kp.rs, nepl-core/src/resource`

## 根拠

- `cargo test -p nepl-core --test kp -- --nocapture` は 14 件中 9 passed / 5 failed になった。
- 失敗した test は `wasi_fd_read_raw_iovec_debug`、`wasi_fd_read_raw_iovec_with_dealloc_debug`、`wasi_fd_read_then_alloc_header_debug`、`local_scanner_new_logic_debug`、`kpread_to_kpwrite_prefixsum_i32`。
- `fd_read` 系 fixture は `fd_read 0 iov 1 nread` の後に `buf` / `buf + 1` / `buf + 2` を `load_u8` で読むが、Resource IR は外部 WASI call が `iovec.buf` の指す cell を初期化する効果を持つことを表現できず、`RawMemoryLoadCell ... found Uninit` を報告する。
- `local_scanner_new_logic_debug` は scanner header と buffer byte の raw load が `sc` / temporary raw address 上の `Uninit` として報告された。
- `kpread_to_kpwrite_prefixsum_i32` は `pref + dynamic_offset` の store/load が `StorageOffset(ResourceOffset { bytes: None })` として集約され、直前の store で同じ logical cell が初期化されたことを証明できない。
- Resource IR lowering の深い call tree 修正を一時退避した状態でも `wasi_fd_read_raw_iovec_debug` は同じ `buf` の `Uninit` で失敗したため、今回の lowering 変更による regression ではなく既存の raw initialized-state モデル不足として扱う。

## 問題

nepl-core tests/kp.rs fails after the current Resource IR gate because direct WASI fd_read and dynamic raw-memory prefix-sum fixtures read buffers that Resource IR still sees as Uninit.

## 影響

Full nepl-core integration tests are blocked, and self-host IO style programs cannot rely on strict initialized-cell verification for WASI out buffers or dynamic raw storage.

## 修正方針

Design typed Resource IR summaries for external raw writes and dynamic raw cell ranges. Preserve RawMemoryLoadCell strictness; do not silence the diagnostics.

具体的には、WASI `fd_read` / `args_get` / `path_open` のような外部 call を単なる unsafe call として素通しせず、どの out pointer / iovec range を initialized にするかを Resource IR の effect として明示する。加えて、dynamic offset の store/load は「不明だから常に未初期化」ではなく、range owner と要素幅を持つ initialized range として扱える設計へ拡張する。これにより strict `RawMemoryLoadCell` gate は維持したまま、正しく初期化された IO buffer と raw array fixture を通す。

## 検証

- `cargo test -p nepl-core --test kp -- --nocapture`: 現状 failed。14 件中 9 passed / 5 failed。
- 修正後は同コマンドが pass し、少なくとも `fd_read` out buffer、scanner header/buffer、dynamic offset prefix sum の regression を個別に固定する。
