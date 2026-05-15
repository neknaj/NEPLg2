---
id: ISS-20260515T203122854Z-STD-ENV-CLIARG-GET-CHECKED-ACCEPTS-N-19FA44EB
title: "std/env cliarg_get_checked accepts negative index before argv slot access"
area: stdlib
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "stdlib/std/env/cliarg/raw.nepl; stdlib/tests/cliarg.n.md; nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js"
---

# ISS-20260515T203122854Z-STD-ENV-CLIARG-GET-CHECKED-ACCEPTS-N-19FA44EB: std/env cliarg_get_checked accepts negative index before argv slot access

## 概要

cliarg_get_checked only rejects idx >= argc and buf_size <= 0 before computing arg_slot_raw = argv_raw + idx * 4, so a negative idx can address before the argv pointer array inside the raw argv boundary.

## 対象

- `stdlib/std/env/cliarg/raw.nepl; stdlib/tests/cliarg.n.md; nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js`

## 根拠

- `cliarg_get_checked` は `idx >= argc` と `buf_size <= 0` だけを拒否し、`idx < 0` を拒否していなかった。
- その直後に `arg_slot_raw = argv_raw + idx * 4` を計算するため、負 index は argv pointer array の前方アドレスを指し得る。
- root facade の `cliarg_get` は負 index を拒否していたが、`std/env/cliarg/raw` は explicit import 可能な raw boundary module なので、raw helper 自体も下限検査を持つ必要がある。

## 問題

cliarg_get_checked only rejects idx >= argc and buf_size <= 0 before computing arg_slot_raw = argv_raw + idx * 4, so a negative idx can address before the argv pointer array inside the raw argv boundary.

## 影響

A safe public cliarg_get negative index currently reaches raw pointer slot arithmetic, which violates the memory-safety requirement even if host args_get later fails or returns no useful string.

## 修正方針

Reject idx < 0 before allocating argv scratch or computing arg_slot_raw, add a regression doctest for cliarg_get negative index, and add source policy coverage for the lower-bound check.

## 検証

Run cliarg source policy and focused cliarg doctests.

## 解決

2026-05-16 Agent 1 で解決。

- `cliarg_get_checked` の raw slot address 計算前に `idx < 0` を拒否する条件を追加した。
- source policy に、`lt idx 0` が `arg_slot_raw` 計算より前にあることを固定した。
- `stdlib/tests/cliarg.n.md` の out-of-range doctest に `cli_raw::cliarg_get_checked -1` の regression assertion を追加した。
