---
id: ISS-20260604T033841916Z-STD-FS-AND-IO-APIS-STILL-FLATTEN-TYP-24F6E6AF
title: "std fs and io APIs still flatten typed failures into errno and empty string"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/std/fs/bytes.nepl, stdlib/std/fs/read/path.nepl, stdlib/std/io.nepl"
---

# ISS-20260604T033841916Z-STD-FS-AND-IO-APIS-STILL-FLATTEN-TYP-24F6E6AF: std fs and io APIs still flatten typed failures into errno and empty string

## 概要

Subagent audit and file inspection show fs_bytes_to_string_result returns Result str i32, fs_std_error_to_errno maps StdErrorKind into raw errno, and fs_bytes_to_string returns an empty string on failure. std/io further maps distinct file and UTF-8 failures into broad IoError-style results. This conflicts with the Zenn policy that errors should be enum data, display should be separated, and callers should be able to match failure kinds statically.

## 対象

- `stdlib/std/fs/bytes.nepl, stdlib/std/fs/read/path.nepl, stdlib/std/io.nepl`

## 根拠

- 未記入

## 問題

Subagent audit and file inspection show fs_bytes_to_string_result returns Result str i32, fs_std_error_to_errno maps StdErrorKind into raw errno, and fs_bytes_to_string returns an empty string on failure. std/io further maps distinct file and UTF-8 failures into broad IoError-style results. This conflicts with the Zenn policy that errors should be enum data, display should be separated, and callers should be able to match failure kinds statically.

## 影響

Callers cannot distinguish empty file content from conversion failure in compatibility paths, and higher layers cannot handle missing file, invalid UTF-8, permission/open/read/close failures independently without relying on raw errno conventions.

## 修正方針

Introduce a typed FsError or StdIoError payload that preserves host errno only at the boundary, migrate primary APIs to Result T FsError/StdIoError, and keep empty-string compatibility wrappers clearly documented as legacy. Add future cfg-test-style regular tests for missing file, empty file, invalid UTF-8, permission failure, and close failure.

## 検証

Run focused fs/std_io doctests, source policy for fs report/error contracts, and future regular tests that match individual enum cases rather than errno integers.
