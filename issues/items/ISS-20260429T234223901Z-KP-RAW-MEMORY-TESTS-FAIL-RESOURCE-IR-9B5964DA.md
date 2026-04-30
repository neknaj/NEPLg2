---
id: ISS-20260429T234223901Z-KP-RAW-MEMORY-TESTS-FAIL-RESOURCE-IR-9B5964DA
title: "kp raw memory tests fail Resource IR initialized checks"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nepl-core/tests/kp.rs, nepl-core/src/resource"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行
---

# ISS-20260429T234223901Z-KP-RAW-MEMORY-TESTS-FAIL-RESOURCE-IR-9B5964DA: kp raw memory tests fail Resource IR initialized checks

## 概要

nepl-core tests/kp.rs failed after the current Resource IR gate because direct WASI fd_read and dynamic raw-memory prefix-sum fixtures read buffers that Resource IR still saw as Uninit.

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

nepl-core tests/kp.rs failed after the current Resource IR gate because direct WASI fd_read and dynamic raw-memory prefix-sum fixtures read buffers that Resource IR still saw as Uninit.

## 影響

Full nepl-core integration tests are blocked, and self-host IO style programs cannot rely on strict initialized-cell verification for WASI out buffers or dynamic raw storage.

## 修正

Resource IR の `RawMemoryLoadCell` strict gate は維持したまま、初期化済み raw cell の情報が正しく伝播するように修正した。

- lowering が WASI `fd_read` の `ExternalIo` effect を `UserCall` に潰していたため、raw helper 用の call effect 生成で外部 IO operation 名を保持するようにした。
- `fd_read` の out pointer effect を Resource IR 初期化検査へ追加し、`nread` と `iovec.buf` が指す buffer の raw cells を initialized として扱うようにした。
- 関数 return summary に、返却 raw address 配下の initialized raw cells と「raw address を保持する cell」を含めるようにした。
- branch/match の output 初期化で output 配下の raw-cell descendants を消さないようにし、両 arm で構築した raw header/buffer の状態を caller へ残すようにした。
- raw address を raw memory cell から load したとき、address alias だけでなく initialized raw cells も load 先へ rekey するようにした。
- `add x 0` のような明示的 raw address view は owner check 側で非所有 view として追跡しつつ、cell table 側では `x` と `x + 0` を同一アドレスとして扱うようにした。
- `kp.rs` fixture は、所有権を raw i32 cell だけへ隠さず、scanner header pointer と owned buffer を aggregate で返す形に直した。dynamic raw array の prefix sum は `fill_i32` で range 初期化を明示した。
- 追加で肥大化した Resource IR checker は `initialized_return`、`initialized_external_io`、`initialized_rekey`、`owner_raw_view` に分割し、既存の responsibility policy を更新した。

この作業中に、所有 raw address と非所有 raw address view、および fallible `realloc_raw` の状態表現がまだ設計として不足していることを確認したため、別 issue `ISS-20260430T004118434Z-RESOURCE-IR-LACKS-EXPLICIT-NON-OWNIN-D546F9CD` を追加した。

## 追加確認

- remote main の `7835b392` で、主要修正は `initialized_return`、`initialized_external_io`、`initialized_rekey`、`owner_raw_view` に分割済みであることを確認した。
- Agent 1 側ではその分割設計を維持し、`initialized_external_io` に `fd_pread` / `fd_write` / `fd_pwrite` / `fd_readdir` / `path_open` / `args_get` / `environ_get` / `random_get` などの out pointer / out buffer effect を追加した。
- `fd_pwrite` は `fd_write` と異なり `nwritten` が第 5 引数なので、scalar offset を out pointer と誤認しないように index を分離し、専用 regression を追加した。
- `fd_read` の iovec buffer effect は iovec cell 自体を buffer と誤認せず、cell に格納された buffer alias 側の unknown-offset raw cells を initialized として扱うようにした。

## 関連残件

- full scanner の loop / realloc / `buf + len` write を「header.buf と header.len によって守られた initialized range」として要約する dependent range model は、今回の bug fix とは別の設計課題として `ISS-20260430T012045983Z-RESOURCE-IR-CANNOT-SUMMARIZE-RETURNE-2FDA4B38` に分離した。
- `fd_write` / `fd_pwrite` が iovec input buffer の initialized state を検査しない問題は、external IO read-effect の不足として `ISS-20260430T012423244Z-RESOURCE-IR-EXTERNAL-IO-WRITE-CALLS--3D90A2FD` に分離した。
- unit-returning in-place helper の引数側 initialized effect が summary 対象外である問題は、return summary ではなく function resource effect として再設計する必要があるため `ISS-20260430T012746721Z-RESOURCE-IR-INITIALIZED-SUMMARIES-SK-727D49FD` に分離した。
- これらの分離は `RawMemoryLoadCell` を弱めるためではなく、Stage 4 Resource check で range summary、external IO read-effect、function resource effect を型付きに設計するための追跡である。

## 検証

- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 130 passed。
- `cargo test -p nepl-core --test kp -- --nocapture`: 14 passed。
- `cargo test -p nepl-core --test effects -- --nocapture`: 21 passed。
- `node nodesrc/test_resource_checker_responsibility.js`: passed。
- 回帰テストとして、関数 return / branch return / raw address cell load / returned header pointer 配下の initialized raw cells を `nepl-core/tests/resource_ir.rs` に追加した。
- `cargo fmt`: passed。
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_external_fd_read_initializes_iovec_buffers -- --nocapture`: passed。
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_fd_pwrite_initializes_nwritten_not_offset -- --nocapture`: passed。
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_returned_raw_header_preserves_initialized_pointee -- --nocapture`: passed。
- `node nodesrc/issues.js check`: passed。
