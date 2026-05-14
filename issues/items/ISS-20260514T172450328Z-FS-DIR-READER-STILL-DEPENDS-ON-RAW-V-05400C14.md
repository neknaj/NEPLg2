---
id: ISS-20260514T172450328Z-FS-DIR-READER-STILL-DEPENDS-ON-RAW-V-05400C14
title: "fs dir reader still depends on raw Vec.data storage for Vec<str>"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/std/fs/dir/read_fd.nepl, tests/stdlib/bytebuf_result.n.md, tests/stdlib/fs.n.md"
---

# ISS-20260514T172450328Z-FS-DIR-READER-STILL-DEPENDS-ON-RAW-V-05400C14: fs dir reader still depends on raw Vec.data storage for Vec<str>

## 概要

Focused ByteBuf doctests that import `std/fs` fail in `std/fs/dir/read_fd.nepl` with `type.field.invalid_access` because `fs_read_dir_fd` still reads `get entries "data"` and sorts through a raw `i32` pointer. `Vec` no longer exposes a `data` field, and `Vec<str>` is non-Copy payload storage that must not be handled by raw storage projection.

## 対象

- `stdlib/std/fs/dir/read_fd.nepl, tests/stdlib/bytebuf_result.n.md, tests/stdlib/fs.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib/alloc/io/bytebuilder -i stdlib/alloc/io/bytebuf.nepl -i tests/stdlib/byte_builder.n.md -i tests/stdlib/bytebuf_result.n.md --no-tree -o tmp/agent1-io-empty-region-private-focused.json -j 1 --dist web/dist --assert-io` で `tests/stdlib/bytebuf_result.n.md::doctest#4/#5/#6` が compile failure になった。
- diagnostic は `/stdlib/std/fs/dir/read_fd.nepl:138` の `let entries_data <i32> mem_ptr_addr get entries "data"` に対する `type.field.invalid_access` である。
- `Vec<T>` は Stage 6 で `data: MemPtr<T>` field を削除し、storage owner を `RegionToken<T>` に移しているため、旧 field へ戻す修正は不可である。

## 問題

`fs_read_dir_fd` が directory entries を `Vec<str>` に蓄積したあと、旧 `Vec.data` raw pointer と `fs_sort_strings` で直接並べ替えている。これは `Vec` の現行 public API と一致せず、さらに non-Copy `str` payload を raw storage として扱うため、Stage 6 の Copy-only collection boundary とも衝突する。

## 影響

Any doctest or program importing std/fs can fail before runtime when the fs dir module is loaded. More importantly, directory reading still depends on old raw Vec storage layout and non-Copy string collection assumptions, conflicting with Stage 6 owner-token / Copy-only collection boundaries.

## 修正方針

Redesign fs_read_dir_fd so directory entries are accumulated and sorted through public safe collection/string APIs, or split the directory-entry buffer into a representation whose ownership and sorting contract is proven by the compiler. Do not restore Vec.data or raw pointer sorting as a compatibility shortcut.

## 検証

Add focused doctests or source policy that std/fs/dir/read_fd does not read Vec.data or raw-sort Vec<str>, then run std/fs dir and bytebuf_result focused suites.
