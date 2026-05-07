---
id: ISS-20260507T175704848Z-CLIARG-RAW-ARGV-BOUNDARY-LACKS-RAW-M-C25E93E9
title: "cliarg raw argv boundary lacks raw memory source capability"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-08
target: "stdlib/std/env/cliarg/raw.nepl, nepl-core/src/loader.rs, tests/stdlib/cliarg.n.md, examples/nm.nepl"
---

# ISS-20260507T175704848Z-CLIARG-RAW-ARGV-BOUNDARY-LACKS-RAW-M-C25E93E9: cliarg raw argv boundary lacks raw memory source capability

## 概要

examples/nm.nepl doctest fails because std/env/cliarg/raw.nepl uses store_i32/store_u8 raw memory operations from functions treated as pure after SourceCapabilities became typed.

## 対象

- `stdlib/std/env/cliarg/raw.nepl, nepl-core/src/loader.rs, tests/stdlib/cliarg.n.md, examples/nm.nepl`

## 根拠

- `node nodesrc/tests.js -i examples -o tmp/examples-current-before-ci.json -j 4 --dist web/dist` で `examples/nm.nepl::doctest#1` が失敗した。
- failure は `stdlib/std/env/cliarg/raw.nepl` の `cli_zero_i32_slots_result` / `cli_zero_u8_buffer_result` が `store` を呼ぶが、pure function として扱われ `effect.pure.calls_impure` により拒否される内容だった。
- `cliarg/raw.nepl` は WASI `args_sizes_get` / `args_get` の out pointer と scratch buffer を初期化する raw argv 境界であり、public facade や cstr conversion ではなく、この module だけに raw-memory boundary capability を付与するのが境界として最小で正しい。

## 問題

examples/nm.nepl doctest fails because std/env/cliarg/raw.nepl uses store_i32/store_u8 raw memory operations from functions treated as pure after SourceCapabilities became typed.

## 影響

examples doctests cannot be added to CI without failing, and cliarg argv scratch initialization depends on an implicit raw-memory boundary that the compiler no longer grants.

## 修正方針

Grant raw_memory_boundary capability to the exact cliarg raw argv implementation module and keep public cliarg facade raw-memory-free.

## 検証

Run examples doctests and cliarg source policy; add regression so cliarg/raw remains an exact raw-memory boundary.

## 2026-05-08 Agent 2 修正

根本原因:

- typed SourceCapabilities 化により raw memory operation は source module capability で明示的に許可されるようになった。
- `stdlib/std/env/cliarg/raw.nepl` は WASI argv API のために `store_i32` / `store_u8` で out pointer と scratch buffer を初期化する raw-memory boundary だが、`nepl-core/src/loader.rs` の exact whitelist に含まれていなかった。
- その結果、stdlib module 自体は正しい境界責務を持っているのに、effect checker 上は pure function から unsafe memory operation を呼んだ扱いになっていた。

修正内容:

- `nepl-core/src/loader.rs` の `RAW_MEMORY_BOUNDARY_STDLIB_PATHS` に `["std", "env", "cliarg", "raw.nepl"]` を追加した。
- public facade の `stdlib/std/env/cliarg.nepl` と conversion module の `stdlib/std/env/cliarg/cstr.nepl` には raw-memory boundary capability を与えない設計を明示した。
- `nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js` に source-policy regression を追加し、cliarg raw argv implementation だけが exact raw-memory boundary になることを固定した。

検証:

- `node nodesrc/test_stdlib_cliarg_no_unsafe_unwraps.js`: passed
- `cargo test -p nepl-core raw_memory_boundary_`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i examples -o tmp/examples-cliarg-raw-boundary.json -j 4 --dist web/dist`: total=32, passed=32
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check`: passed
- `cargo fmt -p nepl-core --check`: passed
- `git diff --check`: passed
