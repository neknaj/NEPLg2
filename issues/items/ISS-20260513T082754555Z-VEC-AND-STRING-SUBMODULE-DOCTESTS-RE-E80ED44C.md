---
id: ISS-20260513T082754555Z-VEC-AND-STRING-SUBMODULE-DOCTESTS-RE-E80ED44C
title: "Vec and string submodule doctests rely on stale implicit imports"
area: stdlib
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-13
updated: 2026-05-13
target: "stdlib/alloc/collections/vec/**/*.nepl, stdlib/alloc/string/**/*.nepl"
---

# ISS-20260513T082754555Z-VEC-AND-STRING-SUBMODULE-DOCTESTS-RE-E80ED44C: Vec and string submodule doctests rely on stale implicit imports

## 概要

Focused doctest verification after the raw-memory-boundary source capability refactor still fails in stdlib/alloc/collections/vec and part of stdlib/alloc/string. The failures are compile-time undefined identifier errors such as unwrap_ok, eq, gt, get, mem_ptr_addr, data_ptr, len, free, and push in examples embedded in submodule docs. This indicates the doctest snippets still rely on implicit/transitive imports or older fixture style instead of declaring the APIs they use directly.

## 対象

- `stdlib/alloc/collections/vec/**/*.nepl, stdlib/alloc/string/**/*.nepl`

## 根拠

- `node nodesrc/tests.js -i stdlib/alloc/string -i stdlib/alloc/collections/vec --no-tree -o tmp/agent1-source-capability-proof-string-vec.json -j 1 --dist web/dist`: total=46, passed=6, failed=40。
- 代表例として `stdlib/alloc/collections/vec/access/data.nepl::doctest#1` は `unwrap_ok`、`gt`、`data_ptr`、`free` が未定義になり、以降の型注釈 mismatch / overload failure へ連鎖した。
- 同じ run の failure は raw-memory-boundary capability 不足の `effect.pure.calls_impure` ではなく、doc snippet の import / fixture drift に集中している。

## 問題

Focused doctest verification after the raw-memory-boundary source capability refactor still fails in stdlib/alloc/collections/vec and part of stdlib/alloc/string. The failures are compile-time undefined identifier errors such as unwrap_ok, eq, gt, get, mem_ptr_addr, data_ptr, len, free, and push in examples embedded in submodule docs. This indicates the doctest snippets still rely on implicit/transitive imports or older fixture style instead of declaring the APIs they use directly.

## 影響

The raw boundary capability implementation can be verified by Rust and source-policy regressions, but broad stdlib doctest verification is blocked by stale documentation examples. It also violates the stdlib documentation contract that examples should be executable and current.

## 修正方針

Audit the affected Vec and string submodule doctests, add explicit imports for every symbol used, and where examples still use unsafe helper style such as unwrap_ok without explaining the failure path, rewrite them to current Result/match or std/test assertion style. Keep documentation comments; do not delete examples to reduce line count.

## 検証

- `node nodesrc/tests.js -i stdlib/alloc/string --no-tree -o tmp/string-submodule-doctests-imports-after.json -j 1 --dist web/dist`: total=14, passed=14。
- `node nodesrc/tests.js -i stdlib/alloc/string -i stdlib/alloc/collections/vec --no-tree -o tmp/string-vec-submodule-doctests-imports-after.json -j 4 --dist web/dist`: total=46, passed=15, failed=31。失敗は全て `resource.owner.no_free_obligation` で、`resolve.identifier.undefined` は 0 件。

## 解決

- Vec submodule doctest に `core/result`、`core/math`、`core/field`、`core/mem/internal` など使用 symbol の明示 import を追加した。
- String submodule doctest に `core/result` と `core/cast` を追加し、`Result<(),str>::Err` や `cast` を implicit / transitive import に依存しない形へ直した。
- 機械的な import 追加時に doctest code fence の外へ入ると parser が読めないため、全て ` ```neplg2` 内の既存 `#entry` / `#target` / `#import` 群の後ろへ配置した。
- 残る Vec 側 31 件の失敗は stale import ではなく、`vec_free_storage` / `push` / merge sort buffer cleanup の `resource.owner.no_free_obligation` であり、Stage 6 の raw-memory-backed collection owner model 残件として `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` 側で継続する。
