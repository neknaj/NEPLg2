---
id: ISS-20260430T023401649Z-SELFHOST-REQ-FAILS-STRICT-OWNER-GATE-F0FF69D6
title: "selfhost_req fails strict owner gate for string, ByteBuf, and Vec ownership"
area: stdlib
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/tests/selfhost_req.rs, stdlib/alloc/string.nepl, stdlib/alloc/io.nepl, stdlib/alloc/collections/vec.nepl, stdlib/std/fs.nepl"
---

# ISS-20260430T023401649Z-SELFHOST-REQ-FAILS-STRICT-OWNER-GATE-F0FF69D6: selfhost_req fails strict owner gate for string, ByteBuf, and Vec ownership

## 概要

With the Resource owner gate active, selfhost_req exposes real maybe-leak diagnostics in byte manipulation, string split utilities, and file-read-to-string code. Vec/ByteBuf observers often require consuming owners or returning owned buffers without a consistent destruction path, while str_split_result can partially store owned str values into Vec storage and only deallocate the backing storage on later failure.

## 対象

- `nepl-core/tests/selfhost_req.rs, stdlib/alloc/string.nepl, stdlib/alloc/io.nepl, stdlib/alloc/collections/vec.nepl, stdlib/std/fs.nepl`

## 根拠

- 未記入

## 問題

With the Resource owner gate active, selfhost_req exposes real maybe-leak diagnostics in byte manipulation, string split utilities, and file-read-to-string code. Vec/ByteBuf observers often require consuming owners or returning owned buffers without a consistent destruction path, while str_split_result can partially store owned str values into Vec storage and only deallocate the backing storage on later failure.

## 影響

Self-host code cannot rely on these stdlib APIs under mandatory memory-safety checking. Tests either leak owned buffers/strings or must bypass public APIs with field-level workarounds, and full cargo test cannot be used as a clean regression gate while these ownership contracts remain unresolved.

## 修正方針

Define ownership-safe destruction and observer contracts for str, ByteBuf, and Vec-backed string collections. str_split_result must clean up partially initialized owned elements on failure or avoid storing owned str in raw Vec storage. selfhost_req fixtures should use borrowed observers and explicit frees once the APIs are fixed.

## 検証

cargo test -p nepl-core --test selfhost_req -- --nocapture should pass with no Resource(Owner) diagnostics; cargo test -p nepl-core should no longer stop in selfhost_req after Resource IR tests pass.

## 対応結果

`nepl-core/tests/selfhost_req.rs` と `tests/stdlib/selfhost_req.n.md` を、現在の owner-safe stdlib contract に合わせて更新した。

- `fs_read_to_string` の `Result::Ok(str)` arm は、予期しない成功時にも `str` owner を `consume_str` で終端するようにした。
- `Vec<u8>` の byte access は `get<u8> &buf 0` の borrowed observer を使い、`Some` / `None` の両 arm で `free<u8> buf` を行うようにした。
- parser 風の「最初の delimiter 位置だけ欲しい」処理は `Vec<str>` を作って非 Copy 要素を取り出すのではなく、新設の `str_find` で byte index を得てから `str_slice` する形にした。
- `str_find` は `stdlib/alloc/string.nepl` に public API として追加し、所有 `Vec<str>` を不要にする delimiter search の標準入口にした。

`Vec<str>` や所有要素を含む collection の element Drop / owned pop API 設計は、この issue の fixture failure から分離し、既存の `ISS-20260425T000000Z-RV-STDLIB-004-91534828` で継続して扱う。

## 検証結果

- `cargo test -p nepl-core --test selfhost_req -- --nocapture`: 6 passed
- `node nodesrc/tests.js -i stdlib/alloc/string.nepl -i tests/stdlib/selfhost_req.n.md --no-tree -o tmp/selfhost-req-str-find.json -j 1 --dist web/dist`: total=15, passed=15
