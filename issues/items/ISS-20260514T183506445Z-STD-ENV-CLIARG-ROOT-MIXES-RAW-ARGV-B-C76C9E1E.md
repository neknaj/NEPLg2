---
id: ISS-20260514T183506445Z-STD-ENV-CLIARG-ROOT-MIXES-RAW-ARGV-B-C76C9E1E
title: "std env cliarg root mixes raw argv boundary into public facade"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/std/env/cliarg.nepl, stdlib/std/env/cliarg/raw.nepl, stdlib/std/env/cliarg/cstr.nepl, stdlib/tests/cliarg.n.md, nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js"
---

# ISS-20260514T183506445Z-STD-ENV-CLIARG-ROOT-MIXES-RAW-ARGV-B-C76C9E1E: std env cliarg root mixes raw argv boundary into public facade

## 概要

std/env/cliarg root facade imports core/mem/raw and performs argv out-pointer scratch initialization, raw address conversion, args_get, and raw slot loads directly. This keeps raw-memory-boundary implementation details in the public cliarg API file instead of proving them inside an explicit raw submodule.

## 対象

- `stdlib/std/env/cliarg.nepl, stdlib/std/env/cliarg/raw.nepl, nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js`

## 根拠

- 修正前の `stdlib/std/env/cliarg.nepl` は `core/mem/raw` / `core/mem/internal` / `std/env/cliarg/raw` を wildcard import し、`mem_ptr_addr`、`store_i32`、`args_get`、`load_i32` を public root file 内で直接使っていた。
- `std/env/cliarg/raw.nepl` の冒頭コメントは raw syscall / scratch buffer の詳細を root から分離する方針を既に述べていたが、実装は `cliarg_get` の主要部分を root に残していた。
- Stage 6 の `std/fs` / `std/stdio` root facade 分離と同じく、ordinary `std/env/cliarg` import は public API だけを見せ、raw argv ABI 境界は explicit submodule に閉じる必要がある。

## 問題

std/env/cliarg root facade imports core/mem/raw and performs argv out-pointer scratch initialization, raw address conversion, args_get, and raw slot loads directly. This keeps raw-memory-boundary implementation details in the public cliarg API file instead of proving them inside an explicit raw submodule.

## 影響

Safe std/env/cliarg imports continue to carry source-level raw memory evidence, making Stage 6 public/internal boundary audits weaker and forcing compiler capability proof to trust a broader facade than necessary.

## 修正方針

Move argv scratch allocation, raw address conversion, args_sizes_get/args_get calls, and raw slot load/store orchestration into std/env/cliarg/raw helpers. Keep std/env/cliarg as a thin public facade that delegates to raw helpers and does not import core/mem/raw or core/mem/internal.

## 検証

Run cliarg source policy plus focused stdlib cliarg doctests.

## 解決

2026-05-15 に修正済み。

- `std/env/cliarg` root から `core/mem/raw` / `core/mem/internal` / `std/env/cliarg/cstr` の直接 import を削除した。
- root の `cliarg_count` / `cliarg_get` は `std/env/cliarg/raw` を `cli_raw` qualified namespace で呼ぶ薄い facade にした。
- argv scratch allocation、raw address conversion、out pointer 初期化、`args_get`、raw slot load は `cliarg_count_result` / `cliarg_get_checked` として `std/env/cliarg/raw` に集約した。
- `cstr_len` / `cstr_to_str` は root 経由ではなく `std/env/cliarg/cstr` を明示 import する境界に整理した。
- cstr doctest は `alloc_ptr` owner を直接扱う例から、`RegionToken<u8>` owner と `region_ptr` non-owning view を使う例へ更新した。

検証:

- `node nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js`
- `node nodesrc/tests.js -i stdlib/tests/cliarg.n.md --no-tree -o tmp/agent1-cliarg-root-raw-boundary-cliarg-tests.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/std/env/cliarg.nepl -i stdlib/std/env/cliarg/raw.nepl -i stdlib/std/env/cliarg/cstr.nepl --no-tree -o tmp/agent1-cliarg-root-raw-boundary-module-doctests.json -j 1 --dist web/dist --assert-io`

## 関連

- [ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84](./ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84.md)
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
