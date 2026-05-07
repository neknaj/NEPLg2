---
id: ISS-20260507T050025343Z-SHA256-HASH-DOCTEST-FAILS-RESOURCE-I-A4EE25CE
title: "sha256 hash doctest fails Resource IR cell state under current checker"
area: TEST
status: open
resolved: false
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
