---
id: ISS-20260518T012520895Z-COMPILER-INTRINSIC-DOCTESTS-STILL-AS-4AD0DA0D
title: "compiler intrinsic doctests still assume user raw memory boundary access"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: tests/compiler/intrinsic.n.md
---

# ISS-20260518T012520895Z-COMPILER-INTRINSIC-DOCTESTS-STILL-AS-4AD0DA0D: compiler intrinsic doctests still assume user raw memory boundary access

## 概要

tests/compiler/intrinsic.n.md keeps runtime success doctests that call alloc_raw/load/store from ordinary doctest entry source. Stage 6 source capability proof correctly rejects those use sites with resource.raw.memory_outside_boundary because raw operation authority must be proven from compiler-owned source, not granted to user fixtures.

## 対象

- `tests/compiler/intrinsic.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/compiler/intrinsic.n.md --no-tree -o tmp/agent1-intrinsic-current.json -j 1 --dist web/dist` で、`doctest#2/#3/#5/#6` が `resource.raw.memory_outside_boundary` により compile phase で失敗した。
- 失敗箇所は `alloc_raw` / `load` / `store` / `dealloc_raw` / `load_i32` / `store_i32` の ordinary doctest entry source からの直接利用だった。
- `nepl-core/tests/intrinsic.rs` には `run_main_wasi_i32_raw_memory_boundary` があり、raw load/store runtime codegen は compiler-owned source provenance と source evidence を与えた harness で既に検証できる。

## 問題

tests/compiler/intrinsic.n.md keeps runtime success doctests that call alloc_raw/load/store from ordinary doctest entry source. Stage 6 source capability proof correctly rejects those use sites with resource.raw.memory_outside_boundary because raw operation authority must be proven from compiler-owned source, not granted to user fixtures.

## 影響

The compiler doctest suite stays red and the stale fixture pressures future changes to weaken the raw-memory boundary. It also hides the intended split: raw codegen behavior belongs in compiler-owned harness tests, while ordinary doctests should prove that direct raw operation access is rejected.

## 修正方針

Move ordinary doctest responsibility to raw-boundary rejection fixtures with stable diag_code metadata. Keep runtime raw load/store behavior covered by nepl-core/tests/intrinsic.rs through run_main_wasi_i32_raw_memory_boundary, which supplies compiler-owned source provenance and exact source evidence.

## 検証

Run focused intrinsic doctests, Rust intrinsic tests, issues check, and diff checks.

## 2026-05-18 修正

`tests/compiler/intrinsic.n.md` の責務を、通常 source から raw memory boundary へ入れないことを検証する doctest に整理した。

- `intrinsic_load_store_i64` / `intrinsic_load_store_f64` / enum payload storage / zero-sized struct raw probe の 4 件を `neplg2:test[compile_fail]` に変更し、`diag_code: resource.raw.memory_outside_boundary` を固定した。
- ファイル冒頭に、raw load/store runtime codegen は `nepl-core/tests/intrinsic.rs` の compiler-owned raw boundary harness で検証することを明記した。
- compiler 側の `SourceCapabilities` / Resource IR gate は緩めていない。ordinary doctest source は raw operation authority を持たず、compiler-owned source proof がある場合だけ raw operation が許可される。

検証:

- `node nodesrc/run_doctest.js -i tests/compiler/intrinsic.n.md -n 2 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/intrinsic.n.md -n 3 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/intrinsic.n.md -n 5 --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/intrinsic.n.md -n 6 --dist web/dist`: passed
- `node nodesrc/tests.js -i tests/compiler/intrinsic.n.md --no-tree -o tmp/agent1-intrinsic-raw-boundary-fixtures.json -j 1 --dist web/dist`: total=8, passed=8
- `cargo test -p nepl-core --test intrinsic -- --nocapture`: 4 passed
