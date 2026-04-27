---
id: ISS-20260427T041147635Z-FS-VEC-PUSH-UNWRAP-TRAPS-4A80E8C1
title: "std/fs の Vec push 失敗が unwrap_ok で trap する"
area: stdlib
status: verified
resolved: true
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
- `fs_normalize_relative` は `str_split` facade を使っており、split allocation failure が空 `Vec` に丸められていた。
- どちらも input size / directory entry count に依存して allocation が増え、失敗時は `Result<_, i32>` の errno 境界で `12` を返せる API である。

## 問題

allocation failure が `Result` として伝播されず、stdlib の通常処理から unsafe helper の `unreachable` が実行され得る。

## 影響

大きい path や directory を処理する self-host CLI / loader が、メモリ不足時に `Err(12)` で診断へ進めず trap する。
`RV-STDLIB-010` の unsafe helper 残存問題のうち、filesystem input-dependent path に該当する。

## 修正方針

`v::push` の結果を `match` し、失敗時は `err = 12` として既存 cleanup 経路へ流す。
source policy regression を追加し、`std/fs.nepl` の実装コードに `unwrap_ok` / `uwok` / `unwrap` 系が再導入されないことを固定する。

## 解決内容

- `fs_normalize_relative` を `str_split` ではなく `str_split_result` に切り替え、component 分割の allocation failure を errno `12` として返すようにした。
- path component stack は `v::push` の結果を `match` し、失敗時は consumed owner を空 `Vec` に置き換えて `err = 12` として既存 cleanup 経路へ流すようにした。
- `fs_read_dir_fd` の directory entry accumulation も `v::push` の結果を `match` し、失敗時は `entries` を空 `Vec` に置き換えて errno `12` を返すようにした。
- `nodesrc/test_stdlib_fs_no_unsafe_unwraps.js` を追加し、`std/fs.nepl` の実装コードから unsafe unwrap helper が戻らないことと、上記の errno 伝播構造を固定した。
- CI/source policy の一覧に `nodesrc/test_stdlib_fs_no_unsafe_unwraps.js` を登録した。

## 検証

- `node nodesrc/test_stdlib_fs_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/std/fs.nepl --no-tree -o tmp/fs-push-error-docs.json -j 1`: 7/7 passed
- `node nodesrc/tests.js -i tests/stdlib/fs.n.md --no-tree -o tmp/fs-push-error-focused.json -j 1`: 7/7 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-fs-push-error.json -j 4`: 305/305 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-fs-push-error.json -j 4`: 418/418 passed
