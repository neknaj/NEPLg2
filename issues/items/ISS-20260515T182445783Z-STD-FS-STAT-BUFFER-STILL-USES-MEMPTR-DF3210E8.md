---
id: ISS-20260515T182445783Z-STD-FS-STAT-BUFFER-STILL-USES-MEMPTR-DF3210E8
title: "std fs stat buffer still uses MemPtr owner API"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-16
target: "stdlib/std/fs/stat.nepl; nodesrc/test_stdlib_fs_no_unsafe_unwraps.js"
---

# ISS-20260515T182445783Z-STD-FS-STAT-BUFFER-STILL-USES-MEMPTR-DF3210E8: std fs stat buffer still uses MemPtr owner API

## 概要

std/fs/stat.nepl still allocates the path_filestat_get out buffer with alloc_ptr/dealloc_ptr, keeping the 64-byte filestat scratch free obligation in MemPtr even though MemPtr is now non-owning.

## 対象

- `stdlib/std/fs/stat.nepl; nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`

## 根拠

- `stdlib/std/fs/stat.nepl` の `fs_path_filetype` は `path_filestat_get` の 64 byte out buffer を `alloc_ptr<u8>` で確保し、終了時に `dealloc_ptr<u8>` へ渡していた。
- Stage 6 では `MemPtr<T>` を non-owning pointer view として扱うため、filestat scratch の free obligation は `RegionToken` / storage token に持たせる必要がある。
- `fs_path_filetype` は `std/fs` facade から利用される existence / filetype 判定の基盤であり、ここに `MemPtr` owner API が残ると safe surface に近い filesystem helper へ古い owner model が残る。

## 問題

std/fs/stat.nepl still allocates the path_filestat_get out buffer with alloc_ptr/dealloc_ptr, keeping the 64-byte filestat scratch free obligation in MemPtr even though MemPtr is now non-owning.

## 影響

Filesystem stat/existence checks remain dependent on the old MemPtr owner model and keep Resource IR owner-summary special cases near the std/fs safe facade.

## 修正方針

Move fs_path_filetype stat scratch allocation to RegionToken<u8>, keep raw filestat layout local to the stat boundary, and update source policy to reject direct alloc_ptr/dealloc_ptr in std/fs/stat.nepl.

## 検証

- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`
- `node nodesrc/tests.js -i stdlib/std/fs/stat.nepl --no-tree -o tmp/agent1-fs-stat-region.json -j 1 --dist web/dist --assert-io`: dependency `fs_normalize_range_push` の `resource.raw.identity_escape` で compile fail。stat buffer owner API の再導入ではなく、`Result<Vec<i32>, i32>` owner return を raw identity escape と誤診断する core issue として [ISS-20260515T182630578Z-VEC-OWNER-RESULT-RETURN-TRIPS-RAW-ID-421E44DD](./ISS-20260515T182630578Z-VEC-OWNER-RESULT-RETURN-TRIPS-RAW-ID-421E44DD.md) に分離した。

## 関連

- 親 issue: [ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686](./ISS-20260515T163252091Z-PUBLIC-ALLOC-PTR-APIS-STILL-ENCODE-F-EECDD686.md)
