---
id: ISS-20260426T010005Z-BYTEBUF-BUILDER-WASM-D63A7C11
title: "wasm emitter needs owned byte builder APIs"
area: selfhost
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: "stdlib/alloc/io.nepl, tests/stdlib/byte_builder.n.md"
source: doc/neplg2/self_host_plan.md
---

# ISS-20260426T010005Z-BYTEBUF-BUILDER-WASM-D63A7C11: wasm emitter needs owned byte builder APIs

## 概要

self-host wasm backend は section を順に組み立てるため、append、reserve、LEB128 encoding、slice copy を持つ owned byte builder を必要とする。

## 対象

- `stdlib/alloc/io.nepl`
- `stdlib/alloc/collections/vec.nepl`
- `stdlib/neplg2/core/codegen/wasm/`

## 根拠

- `ByteBuf` は pointer + len の所有 handle として存在する。
- 現行 public API は read/write の媒体が中心で、incremental binary emitter に必要な growable builder 面が不足している。

## 問題

wasm binary を生成するたびに raw memory や `Vec` 内部表現へ依存すると、self-host compiler 自体が unsafe helper と private layout に縛られる。
これは stdlib の public API 中心で書く方針に反する。

## 影響

codegen/wasm が最初から低レベル memory 操作だらけになり、検証しにくい。
また、binary output が部分的に壊れたときに builder invariant をテストで固定できない。

## 修正方針

`ByteBuilder` を追加し、`new`、`with_capacity`、`push_u8`、`push_bytes_ref`、`push_leb_u32`、`finish` を提供する。
`finish` は `ByteBuf` を返し、`fs_write_to_bytes` / `stdio_write_bytes_result` へ渡せるようにする。

## 検証

- LEB128 known vector test。
- wasm header + empty module section を builder で生成して byte列一致を確認する doctest。
- builder growth 後も既存 bytes が保持される property-style test。

## 対応結果

`alloc/io.nepl` に owned `ByteBuilder` を追加し、`byte_builder_new`、`byte_builder_with_capacity`、`byte_builder_reserve`、`byte_builder_push_u8`、`byte_builder_push_bytes_ref`、`byte_builder_push_bytebuf`、`byte_builder_push_leb_u32`、`byte_builder_finish`、`byte_builder_free` を公開した。
builder は `ptr/len/cap` を保持し、append 時は 2 倍 growth で capacity を伸ばす。

`byte_builder_finish` は builder の余剰 capacity を exact-size に詰め直してから `ByteBuf` を返すため、既存の `io_bytebuf_free` が `len` byte で解放する invariant と衝突しない。
`push_*` / `reserve` / `finish` は builder を消費し、失敗時は内部 buffer を解放して `StdErrorKind` を返す。

`byte_builder_push_leb_u32` は unsigned 32-bit bit pattern として値を扱い、WASM の section size / index で使う unsigned LEB128 を出力する。
実装中、owned builder accumulator を loop 内で fallible consuming call に渡すと move checker が D3065/D3054 を出す問題を確認したため、`ISS-20260426T135905659Z-MOVE-CHECKER-REJECTS-OWNED-ACCUMULAT-2EB9BB98` として別 issue を追加した。
この issue では compiler bug を隠さないよう追跡しつつ、現行 compiler で安全に通る recursion / direct copy の形で ByteBuilder API を実装した。

## 検証結果

- `node nodesrc/tests.js -i stdlib/alloc/io.nepl -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/byte-builder-focused-final.json -j 1`: 4/4 passed
- `node nodesrc/tests.js -i tests/stdlib/bytebuf_result.n.md --no-tree -o tmp/byte-builder-bytebuf-result-final.json -j 1`: 6/6 passed
- `trunk build`: pass, warnings なし
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-byte-builder.json`: 13/13 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/byte-builder-stdlib-full.json -j 4`: 406/406 passed
- `cargo fmt --all --check`: pass
