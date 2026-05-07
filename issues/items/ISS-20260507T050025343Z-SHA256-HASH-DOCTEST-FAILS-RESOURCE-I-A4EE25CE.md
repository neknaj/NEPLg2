---
id: ISS-20260507T050025343Z-SHA256-HASH-DOCTEST-FAILS-RESOURCE-I-A4EE25CE
title: "sha256 hash doctest fails Resource IR cell state under current checker"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "stdlib/tests/hash.n.md, stdlib/alloc/hash/sha256.nepl, nepl-core resource checker"
---

# ISS-20260507T050025343Z-SHA256-HASH-DOCTEST-FAILS-RESOURCE-I-A4EE25CE: sha256 hash doctest fails Resource IR cell state under current checker

## 概要

While verifying the hash string access import fix, `stdlib/tests/hash.n.md::doctest#1` failed at compile time with `resource.cell.uninit` in `sha256_rounds_loop` on local `e#0`. The failure was independent from the string access import and was a Resource IR match-payload lowering bug.

## 対象

- `stdlib/tests/hash.n.md, stdlib/alloc/hash/sha256.nepl, nepl-core resource checker`

## 根拠

- `sha256_rounds_loop` の `match sha256_k i` で arm payload binding `Result::Err e` が関数引数 `e` を shadow していた。
- Resource IR lowering は arm payload binding の `Place` として `%e` を作った後、body scope には `ctx.declare_local("e", ty)` により `%e#0` を登録していた。
- initialized checker は payload を `%e` に初期化したが、arm body は `%e#0` を読むため、実際には lowering の同一 binding 内で Place identity が分裂していた。
- 関連計画: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 Resource Check。

## 問題

Match payload binding が外側 local / parameter と同名になると、payload initialization の対象 Place と arm body が参照する Place が一致しなかった。これは SHA-256 固有ではなく、enum payload match 全般で initialized / moved state と drop elaboration bridge の入力を壊す可能性がある。

## 影響

The canonical SHA-256 known-vector doctest could not be used as a regression for hash stdlib changes, and future changes could avoid this suite even though self-host artifact hashing depends on it. Non-Copy payload では drop insertion bridge が source binding 名を失うリスクもあった。

## 修正方針

Resource IR lowering で match payload binding は必ず `ctx.declare_local` が返す固有 Place を authority とする。drop elaboration bridge が HIR source binding へ戻れるよう、`ResourceMatchArm` は checked Place と source binding 名を分離して保持する。checker 側の `resource.cell.uninit` 判定は緩めない。

## 検証

### 2026-05-07 Agent 2 stdlib-side mitigation

`sha256_rounds_loop` は working variable として `e` を引数に持つ一方、`sha256_k i` の `Result::Err` arm でも payload を `e` として bind していた。現在の Resource IR ではこの shadowing が `e#0` の initialized state tracking を混乱させ、Err payload の構築と match value で `resource.cell.uninit` を報告していた。

修正内容:

- `sha256_rounds_loop` 内の `Result::Err e` を `Result::Err err` に変更し、working variable `e` と error payload binding を分離した。
- `nodesrc/test_stdlib_sha256_no_unsafe_unwraps.js` に、`sha256_rounds_loop` が `Result::Err e:` を再導入しない source policy を追加した。
- SHA-256 known-vector doctest は skip せず、Resource IR の回帰として通す。

検証:

- `node nodesrc/test_stdlib_sha256_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/tests/hash.n.md -i stdlib/alloc/hash/sha256.nepl --no-tree -o tmp/sha256-hash-resource-fix.json -j 1 --dist web/dist`: total=1, passed=1
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed

### 2026-05-07 Agent 1 compiler root-cause fix

- `cargo fmt --check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_match_payload_bind_shadow -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_scope_auto_drop_keeps_same_type_shadowed_locals_distinct -- --nocapture`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i stdlib/tests/hash.n.md -i stdlib/alloc/hash/sha256.nepl --no-tree --dist web/dist -o tmp/hash_sha256_resource_agent1_after.json -j 1 --assert-io`: total=1, passed=1
