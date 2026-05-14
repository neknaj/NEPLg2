---
id: ISS-20260514T223113919Z-STD-TEXT-ROOT-RE-EXPORTS-RAW-UTF-8-M-7F3A2723
title: "std/text root re-exports raw UTF-8 memory helpers"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/std/text.nepl, stdlib/std/text/validate.nepl, stdlib/std/text/decode.nepl, stdlib/std/text/convert.nepl, tests/stdlib/text_utf8.n.md, nodesrc/test_stdlib_text_boundary.js"
---

# ISS-20260514T223113919Z-STD-TEXT-ROOT-RE-EXPORTS-RAW-UTF-8-M-7F3A2723: std/text root re-exports raw UTF-8 memory helpers

## 概要

std/text is the ordinary text facade but currently re-exports text/validate and text/decode with public @merge imports. This exposes MemPtr-based UTF-8 validation and raw decode helpers through the normal std/text import path. The related text_utf8 doctests also construct invalid byte sequences with raw i32 store operations outside a raw-memory boundary.

## 対象

- `stdlib/std/text.nepl, stdlib/std/text/validate.nepl, stdlib/std/text/decode.nepl, stdlib/std/text/convert.nepl, tests/stdlib/text_utf8.n.md, nodesrc/test_stdlib_text_boundary.js`

## 根拠

- `stdlib/std/text.nepl` が `./text/validate` と `./text/decode` を public `@merge` しており、ordinary `#import "std/text" as *` から `MemPtr<u8>` based UTF-8 helper に到達できた。
- `tests/stdlib/text_utf8.n.md` の invalid byte fixture は `mem_ptr_addr` と raw address `store_u8` を使っており、現在の raw-memory-boundary discipline では通常 doctest が raw storage identity を持つ形になっていた。

## 問題

std/text is the ordinary text facade but currently re-exports text/validate and text/decode with public @merge imports. This exposes MemPtr-based UTF-8 validation and raw decode helpers through the normal std/text import path. The related text_utf8 doctests also construct invalid byte sequences with raw i32 store operations outside a raw-memory boundary.

## 影響

Ordinary text users can depend on raw memory identity instead of ByteBuf/str conversion APIs, weakening Stage 6 public/raw boundary separation. The stale doctests also fail under the current raw memory boundary checker, obscuring real UTF-8 conversion regressions.

## 修正方針

Stop re-exporting raw validate/decode helpers from std/text root. Keep checked ByteBuf-to-str conversion as the root surface, move raw decode tests to explicit std/text/decode imports, and update invalid byte fixtures to use checked MemPtr store/dealloc APIs instead of raw address store/dealloc.

## 検証

Run std/text source policy, tests/stdlib/text_utf8.n.md, affected std/io/streamio/fs/stdio focused tests, issue validation, and whitespace checks.

## 結果

- `std/text` root は checked `ByteBuf -> str` conversion のみを再公開し、`text/validate` / `text/decode` の raw `MemPtr` helper は明示 submodule import 境界へ閉じた。
- `tests/stdlib/text_utf8.n.md` の raw decode / encode cases は `std/text/decode` を明示 import する形にした。
- invalid UTF-8 fixture は raw `i32` address store ではなく、checked `MemPtr` `store_u8` と `dealloc_ptr` cleanup で構成するようにした。
- focused consumer run で `stdlib/std/io.nepl::doctest#1` の既存 import drift を確認し、`ISS-20260514T223843320Z-STD-IO-DOCTEST-OMITS-EXPLICIT-IOTARG-12D221C3` として分離した。

## 検証結果

- `node nodesrc/test_stdlib_text_boundary.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/agent1-std-text-root-raw-facade.json -j 1 --dist web/dist --assert-io`: 9/9 pass
- `node nodesrc/tests.js -i stdlib/std/io.nepl -i stdlib/std/streamio/input.nepl -i stdlib/std/stdio/read/text.nepl -i stdlib/std/fs/bytes.nepl --no-tree -o tmp/agent1-std-text-root-consumers.json -j 1 --dist web/dist --assert-io`: 4/5 pass。失敗は `std/io` doctest の `WriteStream` import drift で、別 issue に分離済み。
