---
id: ISS-20260505T094232444Z-MEM-FILL-DOCTESTS-STILL-CALL-RAW-FIL-C8F2D305
title: "mem_fill doctests still call raw fill helpers from pure main"
area: TEST
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-05
updated: 2026-05-05
target: tests/stdlib/mem_fill.n.md
---

# ISS-20260505T094232444Z-MEM-FILL-DOCTESTS-STILL-CALL-RAW-FIL-C8F2D305: mem_fill doctests still call raw fill helpers from pure main

## 概要

After resource.raw.unsafe_memory_boundary became an enforced diagnostic, tests/stdlib/mem_fill.n.md still declares raw fill positive doctest main functions as pure while calling memset_u8, fill_i32, and fill_u8.

## 対象

- `tests/stdlib/mem_fill.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/mem_fill.n.md --no-tree -o tmp/mem-fill-agent1-before.json -j 1 --dist web/dist` が 3 total / 0 passed / 3 failed になった。
- 3 件とも `resource.raw.unsafe_memory_boundary` で、`memset_u8` / `fill_i32` / `fill_u8` を pure `main__unit__i32__pure` から呼んでいた。

## 問題

After resource.raw.unsafe_memory_boundary became an enforced diagnostic, tests/stdlib/mem_fill.n.md still declares raw fill positive doctest main functions as pure while calling memset_u8, fill_i32, and fill_u8.

## 影響

Focused mem_fill doctests fail at the effect boundary before they can verify fill behavior. This is a test fixture issue separate from Resource IR fill range correctness.

## 修正方針

Mark only the positive raw fill doctest main functions as impure, keeping future compile_fail boundary tests pure when added.

## 対応

- `tests/stdlib/mem_fill.n.md` の positive raw fill doctest 3 件だけを `fn main <()*>i32>` に変更した。
- raw memory helper 自体や compiler の unsafe memory boundary は緩めていない。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/mem_fill.n.md --no-tree -o tmp/mem-fill-agent1-after.json -j 1 --dist web/dist`: 3 total / 3 passed
- `node nodesrc/issues.js check`: pass
