---
id: ISS-20260506T150445017Z-STRING-INTEGER-SPLIT-LOSES-RAW-MEMOR-36A59D71
title: "String integer split loses raw-memory boundary capability"
area: core
status: fixed
resolved: true
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

## 2026-05-06 修正

`stdlib/alloc/string/integer.nepl` を監査し、`from_u128_radix` が owned string buffer の data pointer に対して `store_u8` を直接行う内部実装であることを確認した。これは safe public API ではなく、Stage 6 で最終的な internal/public 境界へ移すまで compiler-owned raw-memory-boundary capability で扱うべき exact stdlib path である。

`nepl-core/src/loader.rs` の configured stdlib exact path table に `alloc/string/integer.nepl` を追加し、`nepl-core/tests/effects.rs` の loader regression に同じ path を追加した。stdlib 全体や arbitrary suffix を許可せず、configured `stdlib_root` の canonical path と完全一致した file だけが raw memory boundary になる。

同時に `stdlib/alloc/string/float.nepl` も確認したが、float conversion は `StringBuilder` と `alloc/string/integer` に委譲しており、直接の `load_*` / `store_*` / raw allocation 操作を持たない。したがって float module へ raw-memory-boundary capability は追加しない。これは権限を最小化するためではなく、責務分割上 float module が raw memory owner ではないことに基づく設計判断である。

検証:

- `cargo test -p nepl-core --test effects loader_marks_configured_stdlib_byte_and_scanner_boundaries_as_raw_memory_boundary -- --nocapture`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo fmt --check`: passed
- `node nodesrc/test_stdlib_string_integer_boundary.js`: passed
- `node nodesrc/test_stdlib_string_float_boundary.js`: passed
- `node nodesrc/test_stdlib_builder_owner_boundary.js`: passed
- `node nodesrc/issues.js check`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md -o output/kp_after_string_integer_boundary.json --runner wasm --no-tree -j 1 --assert-io`: 240 秒で local command timeout。partial JSON は total=6, passed=4, failed=1, errored=1。`from_u128_radix` の `effect.pure.calls_impure` は top issue から消え、次 blocker は `alloc/string/builder.nepl` の `sb_append_byte_result` / `sb_append_result` / `sb_build_result` raw memory boundary miss になったため、`ISS-20260506T152038161Z-STRING-BUILDER-SPLIT-LOSES-RAW-MEMOR-637613A4` として分離した。
