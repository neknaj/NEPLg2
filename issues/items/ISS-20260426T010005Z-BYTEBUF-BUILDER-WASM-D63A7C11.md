---
id: ISS-20260426T010005Z-BYTEBUF-BUILDER-WASM-D63A7C11
title: "wasm emitter needs owned byte builder APIs"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: stdlib/alloc/io.nepl
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
