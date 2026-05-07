---
id: ISS-20260507T045028757Z-KP-PREFIX-SUM-REGRESSION-LACKS-EXPLI-B8A2B29A
title: "KP prefix sum regression lacks explicit range guards after typed fill range"
area: test
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/tests/kp.rs, tests/stdlib/kp.n.md"
source: "doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行"
---

# ISS-20260507T045028757Z-KP-PREFIX-SUM-REGRESSION-LACKS-EXPLI-B8A2B29A: KP prefix sum regression lacks explicit range guards after typed fill range

## 概要

KP prefix sum regression uses fill_i32 but still performs dynamic pref loads/stores without explicit 0 <= index < pref_len guards, so the stricter typed element range checker correctly rejects the source.

## 対象

- `nepl-core/tests/kp.rs, tests/stdlib/kp.n.md`

## 根拠

- `cargo test -p nepl-core --test kp kpread_to_kpwrite_prefixsum_i32 -- --nocapture` fails with `RawMemoryLoadCell Uninit` for `pref[+Symbolic(..)].Deref`.
- Existing Resource IR tests require a typed guard for scaled symbolic `fill_i32` loads; relaxing the checker for stdin-derived indices would weaken memory safety.

## 問題

The KP regression fixture relies on loop/input convention for prefix indices instead of making the range proof explicit in source. After fill_i32 moved to typed element ranges, dynamic load/store sites must carry ResourceConditionFact proof for the index used by the scaled offset.

## 影響

Leaving the fixture unguarded either keeps KP regression failing or pressures the compiler toward an unsound dynamic-offset initialization relaxation.

## 修正方針

Add explicit range guards around prefix-buffer dynamic loads/stores and query loads. Keep RawMemoryLoadCell strict: only guarded scaled symbolic offsets may use fill_i32 initialized element ranges.

## 検証

Run the focused KP test and the existing word-fill Resource IR guard regressions.

## 2026-05-07 修正

KP prefix sum fixture の `pref` dynamic access を、typed element range checker が要求する明示 guard 付きに更新した。

- prefix 構築 loop の `prev = load_i32 pref + im1 * 4` は `0 <= im1 && im1 < pref_len` の then branch 内でだけ読む。
- prefix 構築 loop の `store_i32 pref + i * 4` は `0 <= i && i < pref_len` の then branch 内でだけ書く。
- query loop の `left` / `right` は `0 <= l && l < pref_len && 0 <= r1 && r1 < pref_len` の then branch 内でだけ読む。

これは compiler 側の緩和ではない。`fill_i32` が記録する initialized element range は、scaled symbolic offset と range guard が Resource IR state から証明できる場合だけ使われる。

検証:

- `cargo test -p nepl-core --test kp kpread_to_kpwrite_prefixsum_i32 -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_word_fill -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_dynamic_fill -- --nocapture`: passed
- `trunk build`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 3 --dist web/dist`: passed, stdout `6\n14\n15\n`
