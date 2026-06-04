---
id: ISS-20260604T214744868Z-STD-FS-RAW-FD-DIR-STAT-HELPERS-STILL-78981167
title: "std fs raw fd dir stat helpers still expose host errno as primary errors"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-05
target: "stdlib/std/fs/fd.nepl, stdlib/std/fs/read/fd.nepl, stdlib/std/fs/write/fd.nepl, stdlib/std/fs/dir/**, stdlib/std/fs/stat.nepl, stdlib/std/fs/path/normalize.nepl"
---

# ISS-20260604T214744868Z-STD-FS-RAW-FD-DIR-STAT-HELPERS-STILL-78981167: std fs raw fd dir stat helpers still expose host errno as primary errors

## 概要

Subagent review during the std fs typed error fix found that lower fd, dir, stat, and path-normalize helpers still return Result T i32 as their primary public surface. Path read/write/text APIs now wrap these into FsError, but the lower surfaces still require callers to know raw host errno conventions.

## 対象

- `stdlib/std/fs/fd.nepl, stdlib/std/fs/read/fd.nepl, stdlib/std/fs/write/fd.nepl, stdlib/std/fs/dir/**, stdlib/std/fs/stat.nepl, stdlib/std/fs/path/normalize.nepl`

## 根拠

- `stdlib/std/fs/fd.nepl` の open / close helper は host errno を `Result T i32` として返す。
- `stdlib/std/fs/read/fd.nepl` と `stdlib/std/fs/write/fd.nepl` は fd read/write loop の failure を raw errno のまま公開する。
- path-level API は `FsError` へ wrap 済みだが、fd / dir / stat / normalize を直接使う caller は raw numeric error に戻れる。
- Zenn 方針では failure kind は enum data として扱い、raw integer や string sentinel を primary API にしないため、下層 helper も raw 境界か typed wrapper かを明示する必要がある。

## 問題

Subagent review during the std fs typed error fix found that lower fd, dir, stat, and path-normalize helpers still return Result T i32 as their primary public surface. Path read/write/text APIs now wrap these into FsError, but the lower surfaces still require callers to know raw host errno conventions.

## 影響

Callers that bypass the path read/write facade still cannot match filesystem failures as enum data, and future GUI/self-host tooling may reintroduce errno flattening through fd, dir, stat, or normalize APIs.

## 修正方針

Introduce typed lower-level FsError variants or typed fd/dir/stat wrapper APIs, keep raw errno only inside std/fs/raw or explicitly named legacy errno functions, and document which helpers are raw boundary utilities.

## 検証

Add source policy and doctests that require lower fs public helpers to return FsError or to be explicitly marked as raw errno boundary wrappers.
