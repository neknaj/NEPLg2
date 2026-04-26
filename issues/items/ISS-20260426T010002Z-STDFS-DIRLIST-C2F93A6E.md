---
id: ISS-20260426T010002Z-STDFS-DIRLIST-C2F93A6E
title: "stdlib discovery needs directory and path interfaces"
area: selfhost
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-26
updated: 2026-04-26
target: stdlib/std/fs.nepl
source: doc/neplg2/self_host_plan.md
---

# ISS-20260426T010002Z-STDFS-DIRLIST-C2F93A6E: stdlib discovery needs directory and path interfaces

## 概要

self-host compiler は import 解決と stdlib discovery のために、ファイル単体の read だけでなく directory traversal と path normalization を必要とする。

## 対象

- `stdlib/std/fs.nepl`
- `stdlib/neplg2/core/module/`

## 根拠

- 現行 Rust loader は host filesystem と bundled stdlib VFS を使って import を解決している。
- NEPLg2.0 self-host CLI が同等のことを行うには、root 配下の `.nepl` / `.n.md` の存在確認、相対 path の正規化、ディレクトリ列挙が必要になる。

## 問題

`fs_read_to_string path` だけでは、stdlib root の探索、module anchor の検出、複数 input root のスキャン、cache invalidation のための file metadata 取得ができない。

## 影響

コンパイラ本体は in-memory VFS だけなら動かせても、CLI としてプロジェクト全体をコンパイルできない。
import 解決が caller 側の手作り path list に依存し、Rust 実装との parity test も固定 fixture に閉じる。

## 修正方針

`std/fs` に `fs_exists`、`fs_is_file`、`fs_is_dir`、`fs_read_dir`、`fs_normalize_relative` を追加する。
WASI では preopen root から外へ出ないことを API 内で保証する。
self-host core は host fs を直接持たず、`cli/file_io.nepl` が `FileSystemSnapshot` または callback で core module loader へ渡す。

## 検証

- temporary directory に複数 `.nepl` を置き、`fs_read_dir` が安定順で返すことを確認する。
- `..` を含む path が preopen root 外へ出ないことを確認する。
- `stdlib` root discovery の fixture を self-host CLI の focused test に追加する。
