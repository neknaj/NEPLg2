---
id: ISS-20260429T131646897Z-BYTEBUF-EMPTY-NON-EMPTY-CONDITIONAL--34FBA0C2
title: "ByteBuf empty/non-empty ownership invariant lacks explicit structural representation"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-05-18
target: "stdlib/alloc/io/bytebuf.nepl, nodesrc/test_stdlib_io_bytebuf_owner_boundary.js, tests/stdlib/memory_safety.n.md, doc/neplg2/static_check_complexity_reduction_plan.md, doc/neplg2/stdlib_collection_mem_string_static_safety_design.md"
---

# ISS-20260429T131646897Z-BYTEBUF-EMPTY-NON-EMPTY-CONDITIONAL--34FBA0C2: ByteBuf empty/non-empty ownership invariant lacks explicit structural representation

## 概要

After fixing raw address view owner moves, io_bytebuf_from_str_result exposes a deeper stdlib/API modeling gap: the empty branch returns a ByteBuf with no free obligation while the non-empty branch returns a ByteBuf whose ptr owns storage. Resource IR now propagates that conditional obligation as `MaybeFreed` across function summaries, but ByteBuf itself still encodes the invariant as a loose pair of `ptr` and `len` instead of a structural representation the compiler can fully validate.

## 対象

- `stdlib/alloc/io.nepl, stdlib/core/mem.nepl, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `resource_ir_owner_check_keeps_bytebuf_owner_after_raw_address_view` で、元の `ConstructInput out Moved` は解消した一方で、空/非空 ByteBuf の owner が `MaybeFreed` として合流することが分かった。
- 空 branch は `io_bytebuf_empty` により free obligation を持たず、非空 branch は `alloc_ptr<u8>` の owner を `ByteBuf.ptr` に移すため、現在の `OwnerState` merge では同じ struct field に `NoFreeObligation | Live` が合流する。
- 今回の修正で `MaybeFreed` の return boundary と function summary 伝播は追加した。残る問題は、ByteBuf の「空なら owner なし、非空なら owner あり」という値不変条件が stdlib の規約に留まり、型/IR の構造として表現されていない点である。
- これは [静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 の stdlib memory API 責務分離に属する。

## 問題

ByteBuf の以前の表現では、`len == 0` と owner 不在、`len > 0` と owner 存在の対応が型や Resource IR op ではなく関数実装の discipline になっていた。これにより、owner check は保守的な `MaybeFreed` 伝播に頼る必要があり、ByteBuf constructor/free API の正当性を構造的に検証しづらかった。

## 影響

ByteBuf-producing APIs such as io_bytebuf_from_str_result, stdio_finish_read_buffer, fs_read_fd_bytes, and stream input helpers can be made leak-safe by `MaybeFreed` propagation, but the API boundary remains harder to reason about than necessary. Self-host I/O and buffer APIs should not inherit this loose invariant as a long-term design.

## 修正方針

Decide the representation boundary instead of adding local exceptions. Preferred direction is to represent ByteBuf as an enum with an owning non-empty payload and an empty no-owner variant, or introduce an explicit `OwnedBytes`/`Storage` wrapper whose free obligation is structurally separate from non-owning `MemPtr`. Update `io_bytebuf_free` and all ByteBuf constructors to match the chosen model, then simplify any Resource IR special handling that becomes unnecessary.

## 2026-05-18 再確認

この issue は過去に fixed になっていたが、実装は `ByteBuf.region: RegionToken<u8>` と `len` の組み合わせへ移っただけで、空 storage を `io_bytebuf_empty_region` の zero-size `RegionToken` sentinel として残していた。`MemPtr` owner field は消えていたものの、空 buffer と owner payload の有無はまだ型ではなく helper 規約に残っていたため、本 issue の「explicit structural representation」という完了条件は満たしていなかった。

今回の修正では `ByteBufStorage::Empty | Owned(RegionToken<u8>)` を導入し、空 storage は owner payload を持たない enum variant、非空 storage は `Owned` payload の owner tokenとして表す。これにより `io_bytebuf_free` と `io_bytebuf_data_ptr_ref` は `match` の網羅性で状態を分岐し、空 buffer cleanup に sentinel token を渡さない設計へ移った。

## 修正内容

- `ByteBufStorage` enum を追加し、`Empty` と `Owned(RegionToken<u8>)` を構造的に分けた。
- `ByteBuf` は `storage <ByteBufStorage>` と `len` を持つ形に変更し、`region + len` の loose invariant をやめた。
- `io_bytebuf_empty_region` を削除し、`io_bytebuf_empty` は `ByteBufStorage::Empty` を直接使うようにした。
- `io_bytebuf_data_ptr_ref` / `io_bytebuf_ptr_ref` / `io_bytebuf_free` は `ByteBufStorage` を `match` し、`Owned` branch だけで `RegionToken` payload を借用または消費する。
- `io_bytebuf_finish_region` は `byte_len <= 0` で受け取った region を解放して `io_bytebuf_empty` へ正規化し、正の長さだけ `ByteBufStorage::Owned(region)` を返す。
- source policy を更新し、`ByteBufStorage`、`io_bytebuf_empty` の structural empty state、`io_bytebuf_data_ptr_ref` の storage match、empty sentinel helper / zero-size `region_new` の禁止を固定した。
- memory-safety doctest と Stage 6 設計文書を更新し、ByteBuf の empty sentinel が残っていないこと、通常 source から `ByteBuf` direct constructor / `storage` field projection を使えないことを明記した。

## 検証

- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`: pass
- `node nodesrc/test_stdlib_bytebuf_utf8_boundary.js`: pass
- `node nodesrc/tests.js -i stdlib/alloc/io/bytebuf.nepl -i tests/stdlib/bytebuf_result.n.md --no-tree -o tmp/agent1-bytebuf-storage-state.json -j 1 --dist web/dist --assert-io`: pass
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-bytebuf-storage-state-memory-safety.json -j 1 --dist web/dist --assert-io`: total=54, passed=54
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass

## 切り分け済み

- `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/agent1-bytebuf-storage-state-builder-text.json -j 1 --dist web/dist --assert-io`: `text_utf8` 9 件、`byte_builder` doctest#1/#3 は pass。`byte_builder.n.md::doctest#2` は default 60000ms compile timeout。
- 同じ `byte_builder.n.md::doctest#2` timeout は `origin/main` worktree でも再現するため、今回の `ByteBufStorage` 変更による退行ではない。`ISS-20260518T085107445Z-BYTEBUILDER-LEB128-DOCTEST-EXCEEDS-D-3FB2EE7D` として compiler/static-check performance issue に分離した。
