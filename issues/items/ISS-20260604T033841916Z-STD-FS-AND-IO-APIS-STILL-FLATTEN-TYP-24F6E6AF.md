---
id: ISS-20260604T033841916Z-STD-FS-AND-IO-APIS-STILL-FLATTEN-TYP-24F6E6AF
title: "std fs and io APIs still flatten typed failures into errno and empty string"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-05
target: "stdlib/std/fs/error.nepl, stdlib/std/fs/bytes.nepl, stdlib/std/fs/read/path.nepl, stdlib/std/fs/write/path.nepl, stdlib/std/io.nepl, stdlib/neplg2/cli/file_io.nepl"
---

# ISS-20260604T033841916Z-STD-FS-AND-IO-APIS-STILL-FLATTEN-TYP-24F6E6AF: std fs and io APIs still flatten typed failures into errno and empty string

## 概要

Subagent audit and file inspection showed fs_bytes_to_string_result returned Result str i32, fs_std_error_to_errno mapped StdErrorKind into raw errno, and fs_bytes_to_string returned an empty string on failure. std/io further mapped distinct file and UTF-8 failures into broad IoError-style results. This conflicted with the Zenn policy that errors should be enum data, display should be separated, and callers should be able to match failure kinds statically.

Fixed by adding `FsOperation`, `FsErrorKind`, and `FsError`, then migrating the primary path/text read/write surfaces to `Result T FsError`. Raw errno projection now goes through explicitly named `*_errno` compatibility wrappers and `fs_error_to_errno`.

## 対象

- `stdlib/std/fs/error.nepl`
- `stdlib/std/fs/bytes.nepl`
- `stdlib/std/fs/read/path.nepl`
- `stdlib/std/fs/write/path.nepl`
- `stdlib/std/io.nepl`
- `stdlib/neplg2/cli/file_io.nepl`

## 根拠

- `stdlib/std/fs/error.nepl` now stores operation, kind, optional errno, and optional `StdErrorKind` as structured payload.
- `stdlib/std/fs/bytes.nepl` returns `Result str FsError` for text conversion and exposes errno only through `fs_bytes_to_*_errno_result`.
- `stdlib/std/fs/read/path.nepl` and `stdlib/std/fs/write/path.nepl` wrap open/read/write/close failures in `FsError`, including close-after-read/write operations.
- `stdlib/std/io.nepl` uses `fs_error_to_std_error_kind` instead of mapping all fs failures to `StdErrorKind::IoError`.

## 問題

The old primary fs text/read/write APIs flattened typed failures into `i32` errno. The remaining `fs_bytes_to_string` empty-string facade is now documented as legacy compatibility, while the primary result API preserves `FsError`.

## 影響

Callers can now match invalid UTF-8, out-of-memory, invalid operation, not-capable, and host errno cases through `FsErrorKind`, and can inspect the failed `FsOperation`. Compatibility callers can still project to errno explicitly.

## 修正方針

Implemented `FsError` and migrated the primary path/text read/write APIs to it. Lower fd/dir/stat/normalize helpers that still expose raw errno have been split into `ISS-20260604T214744868Z-STD-FS-RAW-FD-DIR-STAT-HELPERS-STILL-78981167` so this issue stays scoped to the path/text/std-io API surface.

## 検証

Verified with focused fs/std_io/selfhost doctests and source policies. Tests now match `FsErrorKind::InvalidUtf8` and use `fs_error_to_errno` only as an explicit compatibility projection.
