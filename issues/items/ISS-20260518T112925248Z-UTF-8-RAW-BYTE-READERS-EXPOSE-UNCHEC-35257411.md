---
id: ISS-20260518T112925248Z-UTF-8-RAW-BYTE-READERS-EXPOSE-UNCHEC-35257411
title: "UTF-8 raw byte readers expose unchecked public index boundary"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/string/utf8.nepl, stdlib/std/text/validate.nepl, stdlib/std/text/decode.nepl"
---

# ISS-20260518T112925248Z-UTF-8-RAW-BYTE-READERS-EXPOSE-UNCHEC-35257411: UTF-8 raw byte readers expose unchecked public index boundary

## 概要

alloc/string/utf8 and std/text/validate expose raw MemPtr byte readers and sequence validators as public helpers, so explicit submodule imports can call raw byte reads without the validate_mem range discipline.

## 対象

- `stdlib/alloc/string/utf8.nepl, stdlib/std/text/validate.nepl, stdlib/std/text/decode.nepl`

## 根拠

- `stdlib/alloc/string/utf8.nepl` は `string_utf8_byte_at(MemPtr<u8>, i32)` と `string_utf8_validate_two/three/four` を public にしていた。
- `stdlib/std/text/validate.nepl` は `text_utf8_byte_at(MemPtr<u8>, i32)` と sequence validator を public にし、`std/text/decode.nepl` もその unchecked reader 名に依存していた。
- これらは root facade からは再公開されていないが、explicit submodule import で到達でき、`validate_mem` / `decode_next` の byte_len discipline を通らない raw byte read API surface になっていた。

## 問題

alloc/string/utf8 and std/text/validate expose raw MemPtr byte readers and sequence validators as public helpers, so explicit submodule imports can call raw byte reads without the validate_mem range discipline.

## 影響

UTF-8 validation internals keep an unchecked public surface around raw MemPtr indexing, which conflicts with the Stage 6 policy that raw-memory-backed helpers must keep proof boundaries internal and expose checked conversion contracts.

## 修正方針

Keep raw byte reads private where possible, replace cross-module use with checked byte_len-carrying helpers, and update source policies/doctests so public APIs expose validation/decode contracts rather than unchecked per-byte raw access.

## 検証

Run focused stdlib text/string source policies and memory safety doctests.

## 2026-05-18 Agent 1 修正

`alloc/string/utf8` は単一 byte raw reader と sequence validator を private implementation detail にし、内部 reader を `string_utf8_byte_at_checked(data, byte_len, idx)` へ変更した。leading byte と continuation byte の読み出しはすべて byte_len を伴う checked helper を通り、負 index / `idx >= byte_len` では raw pointer 計算へ進まない。

`std/text/validate` は cross-module decode 用に `text_utf8_byte_at_checked(data, byte_len, idx)` だけを public に残し、旧 `text_utf8_byte_at` 名と sequence validator の public surface を削除した。`std/text/decode` は decode 前に保持している `byte_len` を checked reader へ渡すため、単一 byte reader だけを直接呼ぶ API へ戻らない。

source policy は旧 unchecked helper 名、public sequence validator、byte_len を持たない reader の再導入を拒否する。`tests/stdlib/memory_safety.n.md` には direct import でも `string_utf8_byte_at` / `string_utf8_validate_two` / `text_utf8_byte_at` が見えない compile-fail regression を追加した。

検証:

- `node nodesrc/test_stdlib_string_utf8_boundary.js`
- `node nodesrc/test_stdlib_text_boundary.js`
- `node nodesrc/test_stdlib_documentation_contract.js`
- `node nodesrc/tests.js -i stdlib\std\text\validate.nepl -i stdlib\alloc\string\utf8.nepl --no-tree -o tmp\agent1-utf8-raw-byte-reader-boundary-doc-tests.json -j 1 --dist web\dist --assert-io`: total=6, passed=6
- `node nodesrc/tests.js -i tests\stdlib\memory_safety.n.md --no-tree -o tmp\agent1-utf8-raw-byte-reader-boundary-memory-safety-final.json -j 1 --dist web\dist --assert-io`: total=57, passed=57
- `node nodesrc/tests.js -i stdlib\std\text -i tests\stdlib\text_utf8.n.md --no-tree -o tmp\agent1-utf8-raw-byte-reader-boundary-text-focused.json -j 1 --dist web\dist --assert-io`: total=13, passed=13
- `node nodesrc/tests.js -i stdlib\alloc\string\utf8.nepl -i stdlib\alloc\string\storage.nepl -i stdlib\alloc\io\bytebuf.nepl --no-tree -o tmp\agent1-utf8-raw-byte-reader-boundary-string-focused.json -j 1 --dist web\dist --assert-io`: total=3, passed=3
