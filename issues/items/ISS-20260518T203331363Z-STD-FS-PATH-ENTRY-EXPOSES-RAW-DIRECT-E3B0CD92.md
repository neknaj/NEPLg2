---
id: ISS-20260518T203331363Z-STD-FS-PATH-ENTRY-EXPOSES-RAW-DIRECT-E3B0CD92
title: "std/fs path entry exposes raw directory byte pointer conversion"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/std/fs/path/entry.nepl, stdlib/std/fs/dir/read_fd.nepl"
---

# ISS-20260518T203331363Z-STD-FS-PATH-ENTRY-EXPOSES-RAW-DIRECT-E3B0CD92: std/fs path entry exposes raw directory byte pointer conversion

## 概要

std/fs/path/entry publicly re-exports fs_string_from_bytes(i32,i32), allowing ordinary imports of std/fs/path to call a raw address and length conversion that wraps an arbitrary i32 as MemPtr before UTF-8 validation. The helper is only needed for fd_readdir buffer processing.

## 対象

- `stdlib/std/fs/path/entry.nepl, stdlib/std/fs/dir/read_fd.nepl`

## 根拠

- `stdlib/std/fs/path.nepl` は safe path facade として `pub #import "std/fs/path/entry" as *` を公開している。
- 修正前の `path/entry.nepl` は `pub fn fs_string_from_bytes <(i32,i32)->Result<str,i32>>` を持ち、内部で `mem_ptr_wrap src` により任意 `i32` を `MemPtr<u8>` へ包んでいた。
- この helper の唯一の使用箇所は `std/fs/dir/read_fd.nepl` の `fd_readdir` buffer 走査であり、caller はすでに `RegionToken<u8>` 由来の `buf_ptr` と `used` 範囲検査結果を持っている。
- Stage 6 方針では `MemPtr<T>` は non-owning view であり、safe facade は raw address から view を構成する public API を持たない。

## 問題

std/fs/path/entry publicly re-exports fs_string_from_bytes(i32,i32), allowing ordinary imports of std/fs/path to call a raw address and length conversion that wraps an arbitrary i32 as MemPtr before UTF-8 validation. The helper is only needed for fd_readdir buffer processing.

## 影響

Directory-entry raw memory conversion remains part of the safe path facade, weakening Stage 6 raw-memory-backed API separation and making caller discipline, not the compiler-visible owner/view boundary, carry the proof that the address comes from the fd_readdir RegionToken buffer.

## 修正方針

Move directory-entry byte conversion into the fd_readdir implementation boundary as a private helper, pass a MemPtr derived from the RegionToken-owned buffer view instead of wrapping a raw i32, and update source policy/doctest regressions so the safe path facade cannot expose the raw conversion again.

## 解決

- `fs_string_from_bytes(i32,i32)` を `stdlib/std/fs/path/entry.nepl` から削除し、`std/fs/path` safe facade から raw directory byte conversion が見えないようにした。
- `stdlib/std/fs/dir/read_fd.nepl` に private `fs_dirent_name_to_string(MemPtr<u8>, i32)` を置き、UTF-8 検証と `str` 複製を fd_readdir raw ABI 境界に閉じた。
- `fs_read_dir_fd` は `mem_ptr_wrap add rec ...` ではなく、`RegionToken` から得た `buf_ptr` に `mem_ptr_add` して directory entry name の `MemPtr<u8>` view を作るようにした。
- `stdlib/std/fs/path.nepl` の facade doctest は、`str_eq`、`eq`、`fs_errno_notcapable` を使う module を明示 import する形に直した。これにより path facade の検証が推移的 import に依存しない。
- `nodesrc/test_stdlib_bytebuf_utf8_boundary.js` と `nodesrc/test_stdlib_fs_no_unsafe_unwraps.js` に、path entry 側へ raw conversion が戻らず、dir/read_fd 側が `RegionToken` 由来 pointer を使うことを固定する policy を追加した。
- `tests/stdlib/fs_path_raw_boundary.n.md` を追加し、`std/fs/path` から `fs_string_from_bytes` が解決できないことを compile-fail regression にした。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- [raw-memory-backed APIs parent issue](./ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md)

## 検証

Run focused fs source policy checks and doctests proving fs_string_from_bytes is no longer exported while fs_read_dir_fd still validates directory entry bytes before string construction.

## 検証結果

- `node nodesrc/test_stdlib_bytebuf_utf8_boundary.js`: passed
- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i tests/stdlib/fs_path_raw_boundary.n.md --no-tree -o tmp/agent1-fs-path-raw-boundary-after-path-doc.json -j 1 --dist web/dist --assert-io`: total=1, passed=1
- `node nodesrc/tests.js -i tests/stdlib/fs.n.md --no-tree -o tmp/agent1-fs-dirent-read-fd-focused.json -j 1 --dist web/dist --assert-io`: total=8, passed=8
- `node nodesrc/tests.js -i stdlib/std/fs/path.nepl -i tests/stdlib/fs.n.md --no-tree -o tmp/agent1-fs-dirent-raw-boundary-focused-after-path-doc.json -j 1 --dist web/dist --assert-io`: total=10, passed=10
