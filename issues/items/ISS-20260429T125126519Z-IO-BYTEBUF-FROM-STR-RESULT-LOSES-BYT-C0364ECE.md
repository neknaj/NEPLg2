---
id: ISS-20260429T125126519Z-IO-BYTEBUF-FROM-STR-RESULT-LOSES-BYT-C0364ECE
title: "io_bytebuf_from_str_result loses ByteBuf owner under Resource IR"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_flow.rs, nepl-core/src/resource/owner_state.rs, nepl-core/src/resource/summary.rs, nepl-core/tests/resource_ir.rs, stdlib/alloc/io.nepl, tests/stdlib/streamio.n.md"
---

# ISS-20260429T125126519Z-IO-BYTEBUF-FROM-STR-RESULT-LOSES-BYT-C0364ECE: io_bytebuf_from_str_result loses ByteBuf owner under Resource IR

## 概要

After origin/main 78f310e, tests/stdlib/streamio.n.md reports io_bytebuf_from_str_result ConstructInput on ByteBuf out found Moved and out_raw owner may leak. The conversion allocates/copies bytes but the owner transfer into ByteBuf is not ResourceIR-clean.

## 対象

- `stdlib/alloc/io.nepl, tests/stdlib/streamio.n.md`

## 根拠

- `tests/stdlib/streamio.n.md` の text/binary stream roundtrip が `io_bytebuf_from_str_result` の `Result::Ok(ByteBuf).ptr` owner を戻り値へ移せず、`ConstructInput` / `ReturnValue` の owner violation で止まっていた。
- `io_bytebuf_from_str_result` は `alloc_ptr<u8>` で得た `MemPtr` を `out_raw` / `data_raw` の raw address に分解してから `ByteBuf out byte_len` へ戻しており、Resource IR から見ると owner が raw address local と aggregate field の間で分断されていた。
- 空 `ByteBuf` も `MemPtr(0)` と `len=0` の同じ `ptr` field で表していたため、所有領域がある buffer とない buffer の区別が型に現れていなかった。
- `fs_read_fd_bytes` は 0 byte 読み取り成功時にも scratch buffer owner を確保したまま `ByteBuf len=0` へ丸めており、空 buffer と所有領域の区別がない設計に依存していた。
- `mem_ptr_addr out` の戻り値を `let out_raw` に束縛すると、Resource IR owner checker が raw address alias 経由で `out.raw` の free obligation を `out_raw` へ move していた。
- その後の `ByteBuf out byte_len` では `out.raw` が `Moved` になっており、`ConstructInput` が失敗していた。
- `mem_ptr_addr` は owner を返す関数ではなく、既存 `MemPtr` owner への非所有 address view であるため、call return summary と `let`/assign の owner transfer 対象にしてはいけない。

## 問題

After origin/main 78f310e, tests/stdlib/streamio.n.md reports io_bytebuf_from_str_result ConstructInput on ByteBuf out found Moved and out_raw owner may leak. The conversion allocates/copies bytes but the owner transfer into ByteBuf is not ResourceIR-clean.

## 影響

String-to-byte output helpers used by streamio and self-host IO cannot pass strict memory-safety checking. This blocks buffered output tests independently of StreamWriter header raw loads.

## 修正方針

Resource IR owner checker 側で、非所有 raw address view と owner-bearing raw value を分離した。

- `mem_ptr_addr` の call return summary は即時 owner transfer に使わず、raw address alias として扱う。
- `let out_raw = mem_ptr_addr out` のように、直接 owner を持たず alias だけを持つ `i32` initializer は、宣言/代入時に owner を move せず alias と storage origin だけを引き継ぐ。
- `alloc_raw` など直接 owner を持つ raw value の束縛は従来通り owner transfer する。
- `MaybeFreed` owner が関数戻り値として外へ出る場合は、関数内 leak/unavailable ではなく caller 側へ obligation を移す return boundary として扱い、function summary に conditional owner として伝播する。
- 回帰テストとして、`mem_ptr_addr` の結果を copy 用 raw address に束縛しても、後続の `ByteBuf out len` が allocation owner を保持していることを固定した。
- 負の回帰テストとして、条件付き owner を返す関数の戻り値を caller が捨てると `OwnerMaybeLeaked` が報告されることを固定した。

補足: remote main では空 ByteBuf と非空 ByteBuf の owner invariant を stdlib/API の構造として明示する設計課題が [ISS-20260429T131646897Z-BYTEBUF-EMPTY-NON-EMPTY-CONDITIONAL--34FBA0C2](./ISS-20260429T131646897Z-BYTEBUF-EMPTY-NON-EMPTY-CONDITIONAL--34FBA0C2.md) として分離されていた。今回の stdlib 側修正で `Option<MemPtr<u8>>` による構造的表現へ移行したため、同 issue も別途 fixed に更新する。

## 修正内容

- `ByteBuf.ptr` を `Option<MemPtr<u8>>` に変更し、空 buffer は `None`、所有領域を持つ buffer は `Some(ptr)` として表すようにした。null pointer を所有 field として混ぜる設計をやめた。
- `io_bytebuf_alloc_region` / `io_bytebuf_region_ptr` / `io_bytebuf_finish_region` を追加し、`io_bytebuf_from_str_result` は `RegionToken<u8>` で確保領域 owner を保持したまま copy し、最後に `ByteBuf` へ一度だけ移す形へ変更した。
- `io_bytebuf_from_owned_ptr`、`io_bytebuf_len_ref`、`io_bytebuf_ptr_ref`、`io_bytebuf_byte_at` を追加し、`std/stdio` / `std/fs` / `std/text` / `std/streamio` の direct field read と direct `ByteBuf ptr len` construction を置き換えた。
- `fs_read_fd_bytes` の 0 byte 成功読み取りは scratch buffer を解放してから `io_bytebuf_empty` を返すようにし、空 `ByteBuf` が隠れた owner を持たないことを保証した。
- `nodesrc/test_stdlib_io_bytebuf_owner_boundary.js` を追加し、`io_bytebuf_from_str_result` が raw pointer intermediate や直接 `alloc_ptr` に戻らないこと、`ByteBuf` が `Option<MemPtr<u8>>` で空/所有領域を区別することを固定した。
- 既存 `.n.md` の direct `ByteBuf data len` / `get bytes "ptr"` は public helper 経由に更新した。

## 検証

- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`: passed
- `node nodesrc/test_stdlib_streamio_writer_boundary.js`: passed
- `node nodesrc/test_stdlib_streamio_scanner_boundary.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/io.nepl --no-tree -o tmp/alloc-io-bytebuf-owner-after.json -j 1 --dist web/dist`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/bytebuf_result.n.md --no-tree -o tmp/bytebuf-result-owner-after.json -j 1 --dist web/dist`: total=6, passed=5, failed=1。`io_bytebuf_from_str_result` failure は解消済み。残件は既存 `ISS-20260429T125010191Z-STD-FS-WASI-OUT-POINTER-READS-FAIL-R-7FEF289D` の `fs_open_with_flags` out pointer ResourceIR failure。
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/streamio-after-alloc-io-bytebuf.json -j 1 --dist web/dist`: total=14, passed=8, failed=6。doctest #2/#3 の `io_bytebuf_from_str_result` failure は解消済み。残件は既存 `stdio_finish_read_buffer` / `std fs WASI out pointer` issues。
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_keeps_bytebuf_owner_after_raw_address_view -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_ -- --nocapture`
- `node nodesrc/tests.js -i stdlib/alloc/io.nepl --no-tree -o tmp/alloc-io-bytebuf-owner-after.json -j 1 --dist web/dist`
- remote main の StreamWriter 修正取り込み後、`node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/alloc-io-bytebuf-streamio-after-rebase.json -j 1 --dist web/dist` は `total=14`, `passed=7`, `failed=7`。失敗上位は std/test assertion value の未集約、stdio/fs ByteBuf conditional owner、fs raw load であり、`mem_ptr_addr out` による `ConstructInput out Moved` は再発していない。
