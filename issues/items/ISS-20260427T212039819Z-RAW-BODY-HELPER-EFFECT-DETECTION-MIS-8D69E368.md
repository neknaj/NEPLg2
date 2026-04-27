---
id: ISS-20260427T212039819Z-RAW-BODY-HELPER-EFFECT-DETECTION-MIS-8D69E368
title: "raw body helper effect detection misses mangled raw memory symbols"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/effects.rs, nepl-core/tests/effects.rs"
---

# ISS-20260427T212039819Z-RAW-BODY-HELPER-EFFECT-DETECTION-MIS-8D69E368: raw body helper effect detection misses mangled raw memory symbols

## 概要

raw body effect validation only checks exact raw helper names, so a pure raw body can call a mangled/suffixed raw memory helper symbol such as store_i32__i32_i32__unit__pure without receiving D3025.

## 対象

- `nepl-core/src/effects.rs, nepl-core/tests/effects.rs`

## 根拠

- `nepl-core/src/effects.rs` の `raw_callee_is_raw_memory_effect` は raw body direct callee を marker と完全一致で照合していた。
- `runtime_helpers::helper_base_name` は compiler generated / namespaced symbol の base 名を正規化できるが、effect validation 側では使われていなかった。
- 修正前の `cargo test -p nepl-core --test effects pure_wasm_raw_call_to_suffixed_raw_memory_helper_is_rejected_outside_core_mem` では、期待した `D3025` ではなく raw wasm 後段の parse/codegen diagnostic へ進んだ。

## 問題

raw body effect validation only checks exact raw helper names, so a pure raw body can call a mangled/suffixed raw memory helper symbol such as store_i32__i32_i32__unit__pure without receiving D3025.

## 影響

Generated or user-written raw bodies can bypass the raw memory effect boundary and reach backend/codegen handling instead of being rejected at type/effect validation.

## 修正方針

Normalize direct raw-body callee symbols to their compiler helper base name before comparing them with raw memory helper markers, and add a regression test for suffixed raw memory helper symbols.

## 解決内容

- `raw_callee_is_raw_memory_effect` で direct callee symbol を `helper_base_name` に通してから raw memory helper marker と照合するようにした。
- `store_i32__i32_i32__unit__pure` のような suffixed raw helper symbol を pure raw body から呼ぶ regression test を追加した。
- `fd_write_like` のような marker を含むだけの純粋 extern は既存 test で引き続き許可されることを確認した。

## 検証

- `cargo test -p nepl-core --test effects pure_wasm_raw_call_to_suffixed_raw_memory_helper_is_rejected_outside_core_mem`: pass
- `cargo fmt --check`: pass
- `cargo test -p nepl-core --test effects`: 20/20 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/raw_body_precheck.n.md --no-tree -o tmp/raw-helper-effect-prefix-raw-body-precheck.json -j 1`: 7/7 passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/raw-helper-effect-prefix-move-effect.json -j 1`: 85/85 passed
