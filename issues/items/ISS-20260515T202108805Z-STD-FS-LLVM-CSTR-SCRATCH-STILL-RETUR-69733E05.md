---
id: ISS-20260515T202108805Z-STD-FS-LLVM-CSTR-SCRATCH-STILL-RETUR-69733E05
title: "std/fs llvm cstr scratch still returns MemPtr owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/std/fs/raw/llvm.nepl; nodesrc/test_stdlib_fs_no_unsafe_unwraps.js"
---

# ISS-20260515T202108805Z-STD-FS-LLVM-CSTR-SCRATCH-STILL-RETUR-69733E05: std/fs llvm cstr scratch still returns MemPtr owner

## 概要

std/fs/raw/llvm.nepl copies path bytes for the LLVM path_open fallback with alloc_ptr and returns Result<MemPtr<u8>, i32>, so a temporary C string still exposes MemPtr as a free-obligation owner.

## 対象

- `stdlib/std/fs/raw/llvm.nepl; nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`

## 根拠

- `__fs_copy_to_cstr` は `alloc_ptr<u8>` で一時 C string を確保し、`Result<MemPtr<u8>, i32>` として caller へ返していた。
- `wasi_path_open` は返却された `MemPtr<u8>` を syscall address と deallocation handle の両方として扱い、`dealloc_ptr<u8>` で解放していた。
- Stage 6 の方針では `MemPtr<T>` は non-owning pointer view であり、一時 buffer の free obligation owner は `RegionToken` / storage owner 側に分離する必要がある。

## 問題

std/fs/raw/llvm.nepl copies path bytes for the LLVM path_open fallback with alloc_ptr and returns Result<MemPtr<u8>, i32>, so a temporary C string still exposes MemPtr as a free-obligation owner.

## 影響

The LLVM filesystem fallback remains a raw-backed boundary exception after fd read/write/open/stat/dir moved to RegionToken, keeping PUBLIC-ALLOC-PTR migration open and weakening Stage 6's MemPtr non-owning contract.

## 修正方針

Make __fs_copy_to_cstr return a RegionToken<u8> owner, derive only a non-owning MemPtr view for stores and syscall address passing, deallocate with dealloc_region, and add source policy coverage.

## 検証

Run fs source policy, focused fs consumer doctest where applicable, issues check, and diff whitespace check.

## 解決

2026-05-16 Agent 1 で解決。

- `__fs_copy_to_cstr` の戻り値を `Result<RegionToken<u8>, i32>` に変更した。
- C string scratch は `alloc_region<u8>` で確保し、byte copy と syscall address 取得には `region_ptr` 由来の non-owning `MemPtr<u8>` view だけを使う。
- `wasi_path_open` は `cpath_region` owner を `dealloc_region<u8>` で消費する。
- source policy に `std/fs/raw/llvm.nepl` が `core/mem/pointer/alloc` と `alloc_ptr` / `realloc_ptr` / `dealloc_ptr` を再導入しない検査を追加した。
- LLVM compile-only はローカル PATH に `clang` が無いため `failed to execute clang --version` で実行できなかった。NEPL source-level の回帰検査は fs source policy と fd consumer doctest で固定する。
