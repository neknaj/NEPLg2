---
id: ISS-20260513T082754555Z-VEC-AND-STRING-SUBMODULE-DOCTESTS-RE-E80ED44C
title: "Vec and string submodule doctests rely on stale implicit imports"
area: stdlib
status: open
resolved: false
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

Run node nodesrc/tests.js -i stdlib/alloc/string -i stdlib/alloc/collections/vec --no-tree -o tmp/string-vec-submodule-doctests-after.json -j 1 --dist web/dist, plus the stdlib documentation contract source policy.
