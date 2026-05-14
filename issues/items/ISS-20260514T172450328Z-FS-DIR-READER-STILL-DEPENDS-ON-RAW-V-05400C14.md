---
id: ISS-20260514T172450328Z-FS-DIR-READER-STILL-DEPENDS-ON-RAW-V-05400C14
title: "fs dir reader still depends on raw Vec.data storage for Vec<str>"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
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

`fs_read_dir_fd` が directory entries を `Vec<str>` に蓄積したあと、旧 `Vec.data` raw pointer と `fs_sort_strings` で直接並べ替えている。これは `Vec` の現行 public API と一致せず、`str` が所有権を持たない Copy view であることも `Vec` API の型制約として表現されないため、Stage 6 の collection boundary と衝突する。

## 影響

Any doctest or program importing std/fs can fail before runtime when the fs dir module is loaded. More importantly, directory reading still depends on old raw Vec storage layout and non-Copy string collection assumptions, conflicting with Stage 6 owner-token / Copy-only collection boundaries.

## 修正方針

Redesign fs_read_dir_fd so directory entries are accumulated and sorted through public safe collection/string APIs, or split the directory-entry buffer into a representation whose ownership and sorting contract is proven by the compiler. Do not restore Vec.data or raw pointer sorting as a compatibility shortcut.

## 検証

Add focused doctests or source policy that std/fs/dir/read_fd does not read Vec.data or raw-sort Vec<str>, then run std/fs dir and bytebuf_result focused suites.

## 解決内容

2026-05-15 Agent 1:

- `fs_sort_strings` を `i32` raw pointer + length から `&Vec<str> -> Result<(), i32>` の mutating helper へ再設計した。
- sort 本体は `v::len<str>` / `v::get<str>` / `v::replace<str>` を使い、`load<str>` / `store<str>` / `mem_ptr_addr` による raw storage projection を廃止した。
- `fs_read_dir_fd` は `fs_sort_strings &entries` を呼び、sort invariant error が返った場合は `Vec<str>` owner を解放して `Err(e)` を返すようにした。
- `tests/stdlib/fs.n.md` の directory listing assertion も `Vec.data` 参照から `v::len` / `v::get` へ移し、host filesystem に依存しない `fs_sort_strings_uses_vec_boundary` regression を追加した。
- `nodesrc/test_stdlib_fs_no_unsafe_unwraps.js` に、fs entry sort が raw `Vec` storage pointer を受け取らず public `Vec` boundary で並べ替えることを検査する source policy を追加した。

## 解決後の検証

- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/fs.n.md --no-tree -o tmp/agent1-fs-dir-vec-data-migration-fs.json -j 1 --dist web/dist --assert-io`: 8/8 pass
- `node nodesrc/tests.js -i tests/stdlib/bytebuf_result.n.md --no-tree -o tmp/agent1-fs-dir-vec-data-migration-bytebuf.json -j 1 --dist web/dist --assert-io`: 6/6 pass
