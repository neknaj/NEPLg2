---
id: ISS-20260506T150445017Z-STRING-INTEGER-SPLIT-LOSES-RAW-MEMOR-36A59D71
title: "String integer split loses raw-memory boundary capability"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/loader.rs, stdlib/alloc/string/integer.nepl, tests/stdlib/kp.n.md"
---

# ISS-20260506T150445017Z-STRING-INTEGER-SPLIT-LOSES-RAW-MEMOR-36A59D71: String integer split loses raw-memory boundary capability

## 概要

Remote main split integer conversion helpers into stdlib/alloc/string/integer.nepl, but the loader exact raw-memory-boundary path table does not include that new internal raw-memory-backed module. KP doctest#1 and #7 now fail with effect.pure.calls_impure for from_u128_radix using store.

## 対象

- `nepl-core/src/loader.rs, stdlib/alloc/string/integer.nepl, tests/stdlib/kp.n.md`

## 根拠

- remote main `5428a314` で `stdlib/alloc/string/integer.nepl` が追加され、integer conversion helper が `alloc/string.nepl` から分離された。
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_after_prefixsum_explicit_init.json --runner wasm --no-tree -j 1 --assert-io` で doctest#1 と doctest#7 が `effect.pure.calls_impure` になった。
- 診断は `pure function 'from_u128_radix__u128_i32__Result_T_E_str_i32__pure' uses unsafe memory operation 'store'` で、既存の string access/scanner boundary と同じ exact path table 追従漏れの形をしている。
- `tests/stdlib/kp.n.md::doctest#3` の prefix buffer 問題は `ISS-20260506T145720311Z-KP-PREFIX-SUM-DOCTEST-RELIES-ON-IMPL-5F1F3821` で解消済みのため、この issue は新しい split module の Stage 5 boundary miss として分離する。

## 問題

Remote main split integer conversion helpers into stdlib/alloc/string/integer.nepl, but the loader exact raw-memory-boundary path table does not include that new internal raw-memory-backed module. KP doctest#1 and #7 now fail with effect.pure.calls_impure for from_u128_radix using store.

## 影響

KP doctests and any stdlib path that formats integers through the split module can fail compilation, hiding remaining runtime/performance regressions behind a Stage5 boundary configuration miss.

## 修正方針

Audit stdlib/alloc/string/integer.nepl. If it is an internal raw-memory-backed string construction module during Stage6 migration, add only that configured exact stdlib path to the loader boundary table and add regression coverage so future splits must be deliberate.

## 検証

Run loader raw-memory-boundary regressions and tests/stdlib/kp.n.md to confirm from_u128_radix no longer fails with effect.pure.calls_impure.
