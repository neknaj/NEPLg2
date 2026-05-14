---
id: ISS-20260505T021408593Z-FS-PATH-FILETYPE-LEAKS-NORMALIZED-ST-2B0962CF
title: "fs_path_filetype leaks normalized str owner under strict Resource IR"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-14
target: "stdlib/std/fs.nepl, stdlib/std/fs/path.nepl, tests/stdlib/fs.n.md"
---

# ISS-20260505T021408593Z-FS-PATH-FILETYPE-LEAKS-NORMALIZED-ST-2B0962CF: fs_path_filetype leaks normalized str owner under strict Resource IR

## 概要

fs_path_filetype normalized a path through fs_normalize_relative and then only extracted a raw pointer and length for wasi_path_filestat_get. The returned str owner was not structurally consumed or freed, and the filestat out-buffer filetype byte was read after an external call without an initialized local cell visible to Resource IR.

## 対象

- `stdlib/std/fs.nepl, stdlib/std/fs/path.nepl, tests/stdlib/fs.n.md`

## 根拠

- 未記入

## 問題

fs_path_filetype normalized a path through fs_normalize_relative and then only extracted a raw pointer and length for wasi_path_filestat_get. The returned str owner was not structurally consumed or freed, and the filestat out-buffer filetype byte was read after an external call without an initialized local cell visible to Resource IR.

## 影響

Strict static checking rejects filesystem kind helpers such as fs_exists/fs_is_file/fs_is_dir. This blocks using std/fs as a selfhost file discovery layer under mandatory memory-safety checking.

## 修正方針

Keep normalized syscall paths as StringBuilder owners through fs_normalize_relative_builder, pass the builder buffer to wasi_path_filestat_get, initialize the filetype byte before the external call, then close the builder with string_builder_free after the syscall. Update fs tests so unexpected success arms consume returned str owners through assertion helpers.

## 検証

node nodesrc/tests.js -i tests/stdlib/fs.n.md --no-tree -o tmp/fs-suite-after-path-split.json -j 1 passes 7/7; node nodesrc/tests.js -i stdlib/std/fs.nepl --no-tree -o tmp/fs-facade-after-path-split.json -j 1 passes 7/7.

## 2026-05-14 再発観測

`Vec.push` の owner-preserving failure payload 化後に current `web/dist` で `tests/stdlib/fs.n.md` を再確認したところ、total=7, passed=5, failed=2 になった。

- `doctest#4` は ordinary doctest から `store_u8` を直接呼び、`resource.raw.memory_outside_boundary` で拒否された。これは静的検査を緩めるべきではなく、fixture を public safe API または明示的な raw boundary fixture へ移す必要がある。
- `doctest#5` は `fs_path_filetype` 内で `normalized` を borrow / read / free する経路が `Moved` / `Uninit` と判定されている。`StringBuilder` owner を syscall pointer 参照と cleanup の両方に使う構造が ResourceIR に正しく証明されていないため、normalized path の owner flow を再設計する必要がある。

再検証:

- `node nodesrc/tests.js -i tests/stdlib/fs.n.md --no-tree -o tmp/agent1-vec-push-owner-error-fs-final.json -j 1 --dist web/dist --assert-io`: total=7, passed=5, failed=2
