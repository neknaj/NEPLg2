---
id: ISS-20260427T043342720Z-CLIARG-UNWRAPS-CHECKED-MEMORY-5E10A4D2
title: "std/env/cliarg が checked memory helper を unwrap している"
area: stdlib
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/std/env/cliarg.nepl, stdlib/tests/cliarg.n.md, nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js"
source: "ISS-20260425T000000Z-RV-STDLIB-010-BF35FCBB"
---

# ISS-20260427T043342720Z-CLIARG-UNWRAPS-CHECKED-MEMORY-5E10A4D2: std/env/cliarg が checked memory helper を unwrap している

## 概要

`std/env/cliarg.nepl` は argv buffer の読み書きで `load_u8` / `load_i32` / `store_u8` の checked result を `unwrap` し、異常時に `Option` / error code へ戻さず `unreachable` trap へ落ちる経路を持つ。

## 対象

- `stdlib/std/env/cliarg.nepl`
- `stdlib/tests/cliarg.n.md`
- `nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js`

## 根拠

- LLVM 互換 `args_sizes_get` は `/proc/self/cmdline` buffer を数えるときに `unwrap<i32> load_u8 ...` を使う。
- LLVM 互換 `args_get` は argv buffer への copy と NUL scan で `unwrap<i32> load_u8 ...` を使う。
- `cstr_len` / `cstr_to_str` は caller-provided C string pointer を読みながら `unwrap<i32> load_u8 ...` を使う。
- `cliarg_count` / `cliarg_get` は `args_sizes_get` 後の metadata load を `unwrap<i32> load_i32 ...` にしている。

## 問題

argv は host / WASI 境界から入る入力であり、失敗時は `cliarg_count = 0` や `cliarg_get = None`、LLVM shim の errno 相当 `1` として扱うべきである。
checked memory helper の結果を unsafe helper で剥がすと、self-host CLI の起動引数処理が診断へ進めず trap する。

## 影響

self-host compiler CLI の argv parsing が、環境差・buffer 異常・allocation pressure に対して `Option` / error code で回復できず、起動直後に落ちる可能性がある。

## 修正方針

`unwrap` を `match` に置き換え、LLVM shim は `1`、public facade は既存仕様通り `0` / `None` / `""` へ丸める。
source policy regression を追加し、`stdlib/std/env/cliarg.nepl` の実装コードに unsafe unwrap helper が再導入されないことを固定する。

## 解決内容

- `cli_load_u8_result` を追加し、MemPtr 版 `load_u8` の `Option::None` を LLVM shim errno 相当の `1` へ写すようにした。
- LLVM 互換 `args_sizes_get` / `args_get` の argv byte scan と copy loop を `match` 化し、load/store 異常を `1` として返すようにした。
- `cstr_len_result` を追加し、無効 pointer を `Err` として扱ったうえで、既存 `cstr_len` facade は `0`、`cstr_to_str` は `""` へ丸めるようにした。
- `cliarg_count` / `cliarg_get` の `load_i32` metadata 読み取りを `match` 化し、失敗時は既存仕様通り `0` / `None` を返すようにした。
- `nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js` を追加し、`cliarg` 実装コードから unsafe unwrap helper が戻らないことと checked memory failure の伝播構造を固定した。
- CI/source policy と `doc/testing.md` に新しい guard を登録した。

## 検証

- `node nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i stdlib/std/env/cliarg.nepl --no-tree -o tmp/cliarg-checked-memory-docs.json -j 1`: 5/5 passed
- `node nodesrc/tests.js -i stdlib/tests/cliarg.n.md --no-tree -o tmp/cliarg-checked-memory-focused.json -j 1`: 6/6 passed
- source policy regressions: pass
- `node nodesrc/tests.js -i tests/stdlib --no-tree -o tmp/tests-stdlib-cliarg-checked-memory.json -j 4`: 305/305 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-cliarg-checked-memory.json -j 4`: 418/418 passed
