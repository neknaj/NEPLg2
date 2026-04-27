---
id: ISS-20260427T041147635Z-FS-VEC-PUSH-UNWRAP-TRAPS-4A80E8C1
title: "std/fs の Vec push 失敗が unwrap_ok で trap する"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/std/fs.nepl, tests/stdlib/fs.n.md, nodesrc/test_stdlib_fs_no_unsafe_unwraps.js"
source: "ISS-20260425T000000Z-RV-STDLIB-010-BF35FCBB"
---

# ISS-20260427T041147635Z-FS-VEC-PUSH-UNWRAP-TRAPS-4A80E8C1: std/fs の Vec push 失敗が unwrap_ok で trap する

## 概要

`fs_normalize_relative` と `fs_read_dir_fd` は、`Vec<str>` に component / directory entry を追加するときに `unwrap_ok v::push<str>` を使っている。
`v::push` は容量拡張で allocation failure を返し得るため、filesystem public API が errno を返す代わりに `unreachable` trap へ落ちる。

## 対象

- `stdlib/std/fs.nepl`
- `tests/stdlib/fs.n.md`
- `nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`

## 根拠

- `fs_normalize_relative` は path component stack の拡張で `unwrap_ok v::push<str> stack part` を呼ぶ。
- `fs_read_dir_fd` は directory entry name の蓄積で `unwrap_ok v::push<str> entries name` を呼ぶ。
- どちらも input size / directory entry count に依存して allocation が増え、失敗時は `Result<_, i32>` の errno 境界で `12` を返せる API である。

## 問題

allocation failure が `Result` として伝播されず、stdlib の通常処理から unsafe helper の `unreachable` が実行され得る。

## 影響

大きい path や directory を処理する self-host CLI / loader が、メモリ不足時に `Err(12)` で診断へ進めず trap する。
`RV-STDLIB-010` の unsafe helper 残存問題のうち、filesystem input-dependent path に該当する。

## 修正方針

`v::push` の結果を `match` し、失敗時は `err = 12` として既存 cleanup 経路へ流す。
source policy regression を追加し、`std/fs.nepl` の実装コードに `unwrap_ok` / `uwok` / `unwrap` 系が再導入されないことを固定する。

## 検証

- `fs_normalize_relative` と `fs_read_dir` の既存 doctest を通す。
- `nodesrc/test_stdlib_fs_no_unsafe_unwraps.js` を追加して CI/source policy に登録する。
