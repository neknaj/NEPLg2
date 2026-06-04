---
id: ISS-20260604T214744868Z-STD-FS-RAW-FD-DIR-STAT-HELPERS-STILL-78981167
title: "std fs raw fd dir stat helpers still expose host errno as primary errors"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-05
target: "stdlib/std/fs/fd.nepl, stdlib/std/fs/read/fd.nepl, stdlib/std/fs/write/fd.nepl, stdlib/std/fs/dir/**, stdlib/std/fs/stat.nepl, stdlib/std/fs/path/normalize.nepl"
---

# ISS-20260604T214744868Z-STD-FS-RAW-FD-DIR-STAT-HELPERS-STILL-78981167: std fs raw fd dir stat helpers still expose host errno as primary errors

## 概要

Subagent review during the std fs typed error fix found that lower fd, dir, stat, and path-normalize helpers still return Result T i32 as their primary public surface. Path read/write/text APIs now wrap these into FsError, but the lower surfaces still require callers to know raw host errno conventions.

Fixed by splitting raw errno boundaries into explicitly named `*_raw_errno` helpers and making the normal public names return `Result T FsError`. Private raw ABI helpers remain `Result T i32` because they are inside the syscall / scratch-buffer boundary.

## 対象

- `stdlib/std/fs/fd.nepl, stdlib/std/fs/read/fd.nepl, stdlib/std/fs/write/fd.nepl, stdlib/std/fs/dir/**, stdlib/std/fs/stat.nepl, stdlib/std/fs/path/normalize.nepl`

## 根拠

- `stdlib/std/fs/fd.nepl` の open / close helper は host errno を `Result T i32` として返す。
- `stdlib/std/fs/read/fd.nepl` と `stdlib/std/fs/write/fd.nepl` は fd read/write loop の failure を raw errno のまま公開する。
- path-level API は `FsError` へ wrap 済みだが、fd / dir / stat / normalize を直接使う caller は raw numeric error に戻れる。
- Zenn 方針では failure kind は enum data として扱い、raw integer や string sentinel を primary API にしないため、下層 helper も raw 境界か typed wrapper かを明示する必要がある。

## 問題

Subagent review during the std fs typed error fix found that lower fd, dir, stat, and path-normalize helpers still return Result T i32 as their primary public surface. Path read/write/text APIs now wrap these into FsError, but the lower surfaces still require callers to know raw host errno conventions.

The fixed API keeps raw errno only in `std/fs/raw/*`, private raw ABI helpers, and explicit `*_raw_errno` / `*_errno` compatibility functions. Normal names such as `fs_open_read`, `fs_read_fd_bytes`, `fs_read_dir`, `fs_path_filetype`, and `fs_normalize_relative` now return `FsError`.

## 影響

Callers that bypass the path read/write facade still cannot match filesystem failures as enum data, and future GUI/self-host tooling may reintroduce errno flattening through fd, dir, stat, or normalize APIs.

## 修正方針

Introduce typed lower-level FsError variants or typed fd/dir/stat wrapper APIs, keep raw errno only inside std/fs/raw or explicitly named legacy errno functions, and document which helpers are raw boundary utilities.

Implemented typed wrappers for fd lifecycle, fd read/write, directory open/read, path filetype, path normalization, range-stack build helpers, and entry sorting. Added `FsOperation` variants for the lower filesystem stages so callers can match operation and kind separately from display.

## 検証

Verified with focused fs doctests and source policies. `nodesrc/test_stdlib_fs_no_unsafe_unwraps.js` now rejects public fs APIs that return raw `i32` errors unless the function name is explicitly raw/errno compatibility, and it checks the typed wrappers delegate to raw errno boundaries through `fs_error_from_errno`.
