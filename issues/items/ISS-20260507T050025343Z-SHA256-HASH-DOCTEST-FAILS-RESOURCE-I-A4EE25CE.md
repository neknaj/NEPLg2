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

While verifying the hash string access import fix, stdlib/tests/hash.n.md::doctest#1 failed at compile time with resource.cell.uninit in sha256_rounds_loop on local e#0. The failure is independent from the string access import and appears to be a Resource IR / SHA-256 round state tracking mismatch.

## 対象

- `stdlib/tests/hash.n.md, stdlib/alloc/hash/sha256.nepl, nepl-core resource checker`

## 根拠

- 未記入

## 問題

While verifying the hash string access import fix, stdlib/tests/hash.n.md::doctest#1 failed at compile time with resource.cell.uninit in sha256_rounds_loop on local e#0. The failure is independent from the string access import and appears to be a Resource IR / SHA-256 round state tracking mismatch.

## 影響

The canonical SHA-256 known-vector doctest cannot currently be used as a regression for hash stdlib changes, and future changes may avoid this suite even though self-host artifact hashing depends on it.

## 修正方針

Investigate whether sha256_rounds_loop violates the current value initialization contract or whether Resource IR loses initialized state for recursive round locals. Fix the root cause and keep the known-vector doctest enabled.

## 検証

Run node nodesrc/tests.js -i stdlib/tests/hash.n.md -i stdlib/alloc/hash/sha256.nepl --no-tree with current web/dist and confirm the SHA-256 known-vector doctest passes.

## 2026-05-07 Agent 2 fixed

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
